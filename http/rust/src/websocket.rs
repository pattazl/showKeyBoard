use axum::{
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn};
use crate::AppState;
use crate::models::WsMessage;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    info!("WebSocket connection requested");
    
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket connection established");
    
    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                info!("Received WebSocket message: {}", text);
                
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        let response = process_ws_message(ws_msg).await;
                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if socket.send(Message::Text(response_json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let error_response = WsMessage {
                            msg_type: "error".to_string(),
                            data: Some(serde_json::json!({"error": format!("Parse error: {}", e)})),
                        };
                        if let Ok(response_json) = serde_json::to_string(&error_response) {
                            let _ = socket.send(Message::Text(response_json)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Ok(Message::Ping(data)) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Binary(data)) => {
                info!("Received binary data: {} bytes", data.len());
            }
            Err(e) => {
                info!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    info!("WebSocket connection closed");
}

async fn process_ws_message(msg: WsMessage) -> WsMessage {
    match msg.msg_type.as_str() {
        "ping" => {
            WsMessage {
                msg_type: "pong".to_string(),
                data: Some(serde_json::json!({"time": chrono::Utc::now().timestamp()})),
            }
        }
        "getConfig" => {
            WsMessage {
                msg_type: "config".to_string(),
                data: Some(serde_json::json!({
                    "version": "1.56.0",
                    "server": "rust"
                })),
            }
        }
        "subscribe" => {
            info!("Client subscribed to updates");
            WsMessage {
                msg_type: "subscribed".to_string(),
                data: msg.data,
            }
        }
        "getRecords" => {
            WsMessage {
                msg_type: "records".to_string(),
                data: Some(serde_json::json!({
                    "records": []
                })),
            }
        }
        _ => {
            warn!("Unknown message type: {}", msg.msg_type);
            WsMessage {
                msg_type: "error".to_string(),
                data: Some(serde_json::json!({"error": "Unknown message type"})),
            }
        }
    }
}
