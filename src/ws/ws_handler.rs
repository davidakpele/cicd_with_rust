// ws_handler.rs
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;
use crate::payloads::connection_request::ConnectionRequest;
use crate::{
    services::user_service::UserService,
    utils::jwt::Claims,
    ws::{ws_auth::WsAuth, ws_channel::WsBroadcaster},
};

pub async fn handle_ws_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    client_id: Uuid,
    peer: SocketAddr,
    broadcaster: Arc<WsBroadcaster>,
    user_service: Arc<UserService>,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // AUTHENTICATION PHASE 
    let (user_id, claims) = match ws_receiver.next().await {
        Some(Ok(first_msg)) => {
            match WsAuth::from_first_message(&first_msg).await {
                Ok(WsAuth(claims)) => {
                    println!("[{client_id}] JWT authentication succeeded");
                    (claims.sub as u64, claims)
                }
                Err((code, msg)) => {
                    let _ = ws_sender.send(Message::Text(
                        json!({
                            "type": "error",
                            "status": "authentication_failed",
                            "error": msg,
                            "code": code.as_u16()
                        }).to_string().into()
                    )).await;
                    return;
                }
            }
        }
        Some(Err(e)) => {
            let _ = ws_sender.send(Message::Text(
                json!({
                    "type": "error",
                    "status": "connection_error",
                    "error": format!("Failed to read message: {}", e),
                    "code": 400
                }).to_string().into()
            )).await;
            return;
        }
        None => {
            let _ = ws_sender.send(Message::Text(
                json!({
                    "type": "error",
                    "status": "no_message",
                    "error": "No initial message received",
                    "code": 400
                }).to_string().into()
            )).await;
            return;
        }
    };

    // SESSION CREATION
    let session_id = Uuid::new_v4().to_string();

    // Register client
    let (tx, mut rx) = mpsc::unbounded_channel();
    broadcaster.add_client(client_id, tx).await;

    // Message sending task
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => (),
    }

    println!("[{}] Connection closed", client_id);
}