use axum::{
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
    http::StatusCode,
};
use tokio::sync::broadcast;
use log::{info, warn};
use crate::AppState;

/// Client identity flags — matches Node.js consts in common.js:
/// `const ahkClientFlag = 'ahkClient'` / `const ahkKeyShowFlag = 'ahkKeyShow'`
/// 标记识别为客户端, 接受按键消息
pub const AHK_CLIENT_FLAG: &str = "ahkClient";
/// 标记识别显示按键消息
pub const AHK_KEY_SHOW_FLAG: &str = "ahkKeyShow";
/// Web client — default clientType (matches Node.js: ws.clientType = '')
pub const WEB_CLIENT_FLAG: &str = "";

/// WebSocket upgrade handler — matches Node.js startWS().
/// Clients connect and send their type as first message:
///   - "" (empty/other) → web client, receives preData + real-time data broadcasts
///   - AHK_CLIENT_FLAG  → AHK key collector, receives "IniMonitor" on config change
///   - AHK_KEY_SHOW_FLAG → AHK key display, receives forwarded key messages from ahkClient
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
                        if !handle_client_payload(
                            &mut socket,
                            &mut client_type,
                            &ws_tx,
                            &state,
                            text.to_string(),
                        ).await {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        // Node.js 收到的是 Buffer，二进制帧与文本帧处理方式完全一致
                        let payload = String::from_utf8_lossy(&data).to_string();
                        info!("WS binary frame ({} bytes) from client {:?}", data.len(), client_type);
                        if !handle_client_payload(
                            &mut socket,
                            &mut client_type,
                            &ws_tx,
                            &state,
                            payload,
                        ).await {
                            break;
                        }
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

/// Handle one client payload — matches Node.js `ws.on('message')`.
///
/// Node.js always receives the frame as a Buffer, so **text and binary frames are
/// handled identically** (AHK WebSocket clients often send binary frames).
/// Returns false when the connection should be closed.
async fn handle_client_payload(
    socket: &mut WebSocket,
    client_type: &mut String,
    ws_tx: &broadcast::Sender<(String, String)>,
    state: &AppState,
    payload: String,
) -> bool {
    // 身份识别：Node.js 对每条消息都做判断（不限于首次），这里同样处理
    let token = payload.trim();
    if token == AHK_CLIENT_FLAG || token == AHK_KEY_SHOW_FLAG {
        *client_type = token.to_string();
        info!("ahkFlag:{}", client_type);
        return true; // 身份识别完毕
    }

    if *client_type == WEB_CLIENT_FLAG {
        // Web 端的链接，发送消息给客户端用于刷新界面（Node.js: ws.send(JSON.stringify(preData))）
        let pre_data_str = {
            let s = state.read().await;
            serde_json::to_string(&s.pre_data).unwrap_or_default()
        };
        if socket.send(Message::Text(pre_data_str.into())).await.is_err() {
            return false;
        }
    } else if *client_type == AHK_CLIENT_FLAG {
        // 按键消息需要发送给全部的显示接收端（node: 遍历 clients, clientType == ahkKeyShow）
        let _ = ws_tx.send((AHK_KEY_SHOW_FLAG.to_string(), payload));
    }
    // ahkKeyShow 只接收，不处理其发来的消息
    true
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
