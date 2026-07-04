use axum::{
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
    http::StatusCode,
};
use tokio::sync::broadcast;
use log::{info, warn};
use crate::AppState;

/// WebSocket upgrade handler — matches Node.js startWS().
/// Clients connect and send their type as first message:
///   - "" (empty/other) → web client, receives preData + real-time data broadcasts
///   - "ahkClient"      → AHK key collector, receives "IniMonitor" on config change
///   - "ahkKeyShow"     → AHK key display, receives forwarded key messages from ahkClient
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("WebSocket upgrade request");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // "" = web client (matches Node.js: ws.clientType = '')
    let mut client_type = String::new();

    let ws_tx = {
        let s = state.read().await;
        s.ws_tx.clone()
    };
    let mut rx = ws_tx.subscribe();

    info!("WebSocket connection established");

    loop {
        tokio::select! {
            // Incoming WebSocket message from client
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        let text_str = text.to_string();

                        // Client identity identification (Node.js: ahkClientFlag / ahkKeyShowFlag)
                        if client_type.is_empty() {
                            if text_str == "ahkClient" || text_str == "ahkKeyShow" {
                                client_type = text_str.clone();
                                info!("WS client identified as: {}", client_type);
                                continue;
                            }
                        }

                        if client_type.is_empty() {
                            // Web client (clientType == ''): any message → reply with preData
                            let s = state.read().await;
                            let pre_data_str = serde_json::to_string(&s.pre_data).unwrap_or_default();
                            drop(s);
                            if socket.send(Message::Text(pre_data_str.into())).await.is_err() {
                                break;
                            }
                        } else if client_type == "ahkClient" {
                            // AHK client sends key data → forward to all ahkKeyShow clients
                            let _ = ws_tx.send(("ahkKeyShow".to_string(), text_str));
                        }
                        // ahkKeyShow doesn't send messages (passive receiver)
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WS client disconnected: {}", client_type);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Binary(data))) => {
                        info!("WS binary data: {} bytes", data.len());
                    }
                    Some(Err(e)) => {
                        warn!("WS recv error: {}", e);
                        break;
                    }
                    None => {
                        info!("WS stream ended");
                        break;
                    }
                }
            }

            // Server broadcast → filter by target_type, send to matching clients
            recv_result = rx.recv() => {
                match recv_result {
                    Ok((target_type, message)) => {
                        if target_type == client_type {
                            if socket.send(Message::Text(message.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WS broadcast lagged by {} messages, client={}", n, client_type);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket connection closed: {}", client_type);
}

/// Root path handler: handles BOTH WebSocket upgrades and normal HTTP requests.
/// - WebSocket upgrade → delegates to ws_handler
/// - Normal HTTP → serves index.html from ui_path
/// This matches Node.js where WebSocket shares the same port with no specific path.
pub async fn root_handler(
    ws: Option<WebSocketUpgrade>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(ws) = ws {
        info!("WebSocket upgrade at root path");
        return ws.on_upgrade(|socket| handle_socket(socket, state)).into_response();
    }

    // Normal HTTP request: serve index.html
    let ui_path = {
        let sg = state.read().await;
        sg.ui_path.clone()
    };

    let index_path = std::path::PathBuf::from(&ui_path).join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(html) => {
            axum::response::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .status(StatusCode::OK)
                .body(axum::body::Body::from(html))
                .unwrap()
        }
        Err(_) => {
            axum::response::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain; charset=utf-8")
                .body(axum::body::Body::from("index.html not found"))
                .unwrap()
        }
    }
}
