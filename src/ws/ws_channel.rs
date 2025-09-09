use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::{Mutex, mpsc::{UnboundedSender}};

#[derive(Clone)]
pub struct WsBroadcaster {
    clients: Arc<Mutex<HashMap<Uuid, UnboundedSender<String>>>>,
}

impl WsBroadcaster {
    pub fn new() -> Self {
        WsBroadcaster {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_client(&self, client_id: Uuid, tx: UnboundedSender<String>) {
        self.clients.lock().await.insert(client_id, tx);
    }

    pub async fn remove_client(&self, client_id: &Uuid) {
        self.clients.lock().await.remove(client_id);
    }

    #[allow(dead_code)]
    // Broadcast to all clients
    pub async fn broadcast(&self, message: String) {
        let clients: Vec<(Uuid, UnboundedSender<String>)> = {
            let clients_guard = self.clients.lock().await;
            clients_guard.iter().map(|(id, tx)| (*id, tx.clone())).collect()
        };

        let mut disconnected = Vec::new();

        for (id, tx) in clients {
            if tx.send(message.clone()).is_err() {
                disconnected.push(id);
            }
        }

        for id in disconnected {
            self.remove_client(&id).await;
        }
    }

    #[allow(dead_code)]
    pub async fn broadcast_except(&self, sender_id: &Uuid, message: String) {
        let clients: Vec<(Uuid, UnboundedSender<String>)> = {
            let clients_guard = self.clients.lock().await;
            clients_guard.iter()
                .filter(|(id, _)| *id != sender_id)
                .map(|(id, tx)| (*id, tx.clone()))
                .collect()
        };

        for (_, tx) in clients {
            let _ = tx.send(message.clone());
        }
    }


    // Send to specific client
    pub async fn send_to(&self, client_id: &Uuid, message: String) -> bool {
        let clients = self.clients.lock().await;
        if let Some(tx) = clients.get(client_id) {
            tx.send(message).is_ok()
        } else {
            false
        }
    }

    #[allow(dead_code)]
    // Get all connected client IDs
    pub async fn get_client_ids(&self) -> Vec<Uuid> {
        self.clients.lock().await.keys().cloned().collect()
    }
}