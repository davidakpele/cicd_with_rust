use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use uuid::Uuid;

use crate::{
    services::{user_service::UserService}, ws::{ws_channel::WsBroadcaster, ws_handler::handle_ws_connection}
};

pub async fn start_ws_server(
    addr: &str,
    broadcaster: Arc<WsBroadcaster>,
    user_service: Arc<UserService>,
) {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind WebSocket port");

    println!("🔌 WebSocket server running at ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 0)));

        let broadcaster = broadcaster.clone();
        let user_service = user_service.clone(); 
        
        tokio::spawn(async move {
            handle_connection(stream, peer, broadcaster,  user_service).await;
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    broadcaster: Arc<WsBroadcaster>,
    user_service: Arc<UserService>
) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let client_id = Uuid::new_v4();
        handle_ws_connection(ws_stream, client_id, peer, broadcaster,  user_service).await;
    }
}