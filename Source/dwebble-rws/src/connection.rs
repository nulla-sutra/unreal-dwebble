//! WebSocket connection management

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// Unique connection ID generator
static CONNECTION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn next_connection_id() -> u64 {
    CONNECTION_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Represents a single WebSocket connection
pub struct Connection {
    pub id: u64,
    #[allow(dead_code)]
    pub remote_addr: String,
    #[allow(dead_code)]
    pub subprotocol: Option<String>,
    pub tx: mpsc::UnboundedSender<Message>,
}

impl Connection {
    pub fn new(
        remote_addr: String,
        subprotocol: Option<String>,
        tx: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            id: next_connection_id(),
            remote_addr,
            subprotocol,
            tx,
        }
    }

    pub fn send(&self, data: &[u8]) -> bool {
        self.tx.send(Message::Binary(data.to_vec().into())).is_ok()
    }

    pub fn send_text(&self, text: &str) -> bool {
        self.tx.send(Message::Text(text.to_string().into())).is_ok()
    }

    pub fn close(&self) {
        let _ = self.tx.send(Message::Close(None));
    }
}
