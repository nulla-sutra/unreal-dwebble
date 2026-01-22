/*
 * Copyright 2019-Present tarnishablec. All Rights Reserved.
 */

//! WebSocket Server implementation

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::http::Response as HttpResponse;
use tokio_tungstenite::tungstenite::Message;

use crate::connection::Connection;
use crate::tls::TlsConfig;
use crate::types::{DwebbleWSEventType, DwebbleWSResult};

/// Internal event for the event queue
#[derive(Debug)]
pub struct ServerEvent {
    pub event_type: DwebbleWSEventType,
    pub connection_id: u64,
    pub data: Option<Vec<u8>>,
    pub error: Option<String>,
}

/// Server configuration
pub struct ServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub subprotocols: Vec<String>,
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 0,
            bind_address: "127.0.0.1".to_string(),
            subprotocols: vec![],
            tls: None,
        }
    }
}

/// WebSocket Server
pub struct Server {
    config: ServerConfig,
    connections: Arc<Mutex<HashMap<u64, Arc<Connection>>>>,
    event_rx: Mutex<mpsc::UnboundedReceiver<ServerEvent>>,
    event_tx: mpsc::UnboundedSender<ServerEvent>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    runtime: Option<tokio::runtime::Runtime>,
    actual_port: Mutex<u16>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_rx: Mutex::new(event_rx),
            event_tx,
            shutdown_tx: None,
            runtime: None,
            actual_port: Mutex::new(0),
        }
    }

    pub fn start(&mut self) -> DwebbleWSResult {
        if self.runtime.is_some() {
            return DwebbleWSResult::AlreadyRunning;
        }

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return DwebbleWSResult::RuntimeError,
        };

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = match runtime.block_on(TcpListener::bind(&addr)) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind to {}: {}", addr, e);
                return DwebbleWSResult::BindFailed;
            }
        };

        let local_addr = listener.local_addr().unwrap();
        *self.actual_port.lock() = local_addr.port();

        tracing::info!("WebSocket server listening on {}", local_addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let connections = Arc::clone(&self.connections);
        let event_tx = self.event_tx.clone();
        let subprotocols = self.config.subprotocols.clone();
        let tls_config = self.config.tls.take();

        runtime.spawn(async move {
            let tls_acceptor = tls_config.map(|c| c.acceptor);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Server shutdown signal received");
                        break;
                    }
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                let connections = Arc::clone(&connections);
                                let event_tx = event_tx.clone();
                                let subprotocols = subprotocols.clone();
                                let tls_acceptor = tls_acceptor.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = handle_connection(
                                        stream,
                                        addr,
                                        connections,
                                        event_tx,
                                        subprotocols,
                                        tls_acceptor,
                                    ).await {
                                        tracing::error!("Connection error from {}: {}", addr, e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Accept error: {}", e);
                            }
                        }
                    }
                }
            }
        });

        self.runtime = Some(runtime);
        DwebbleWSResult::Ok
    }

    pub fn stop(&mut self) -> DwebbleWSResult {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = self.runtime.as_ref().map(|rt| {
                rt.block_on(async {
                    let _ = shutdown_tx.send(()).await;
                });
            });
        }

        // Close all connections
        {
            let mut conns = self.connections.lock();
            for (_, conn) in conns.drain() {
                conn.close();
            }
        }

        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(std::time::Duration::from_secs(5));
        }

        *self.actual_port.lock() = 0;
        DwebbleWSResult::Ok
    }

    pub fn poll_event(&self) -> Option<ServerEvent> {
        self.event_rx.lock().try_recv().ok()
    }

    pub fn send(&self, connection_id: u64, data: &[u8]) -> DwebbleWSResult {
        let conns = self.connections.lock();
        if let Some(conn) = conns.get(&connection_id) {
            if conn.send(data) {
                DwebbleWSResult::Ok
            } else {
                DwebbleWSResult::SendFailed
            }
        } else {
            DwebbleWSResult::InvalidHandle
        }
    }

    pub fn send_text(&self, connection_id: u64, text: &str) -> DwebbleWSResult {
        let conns = self.connections.lock();
        if let Some(conn) = conns.get(&connection_id) {
            if conn.send_text(text) {
                DwebbleWSResult::Ok
            } else {
                DwebbleWSResult::SendFailed
            }
        } else {
            DwebbleWSResult::InvalidHandle
        }
    }

    pub fn disconnect(&self, connection_id: u64) -> DwebbleWSResult {
        let mut conns = self.connections.lock();
        if let Some(conn) = conns.remove(&connection_id) {
            conn.close();
            DwebbleWSResult::Ok
        } else {
            DwebbleWSResult::InvalidHandle
        }
    }

    pub fn get_actual_port(&self) -> u16 {
        *self.actual_port.lock()
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.lock().len()
    }

    pub fn info(&self) -> String {
        format!("{}:{}", self.config.bind_address, self.get_actual_port())
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    connections: Arc<Mutex<HashMap<u64, Arc<Connection>>>>,
    event_tx: mpsc::UnboundedSender<ServerEvent>,
    subprotocols: Vec<String>,
    tls_acceptor: Option<tokio_rustls::TlsAcceptor>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(acceptor) = tls_acceptor {
        let tls_stream = acceptor.accept(stream).await?;
        handle_websocket(tls_stream, addr, connections, event_tx, subprotocols).await
    } else {
        handle_websocket(stream, addr, connections, event_tx, subprotocols).await
    }
}

async fn handle_websocket<S>(
    stream: S,
    addr: SocketAddr,
    connections: Arc<Mutex<HashMap<u64, Arc<Connection>>>>,
    event_tx: mpsc::UnboundedSender<ServerEvent>,
    subprotocols: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let mut selected_protocol: Option<String> = None;

    // Callback to handle subprotocol negotiation
    let callback = |req: &Request, mut response: Response| -> Result<Response, HttpResponse<Option<String>>> {
        if !subprotocols.is_empty() {
            if let Some(protocols) = req.headers().get("Sec-WebSocket-Protocol") {
                if let Ok(protocols_str) = protocols.to_str() {
                    for requested in protocols_str.split(',').map(|s| s.trim()) {
                        if subprotocols.iter().any(|s| s == requested) {
                            selected_protocol = Some(requested.to_string());
                            response.headers_mut().insert(
                                "Sec-WebSocket-Protocol",
                                requested.parse().unwrap(),
                            );
                            break;
                        }
                    }
                }
            }
        }
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback).await?;
    let (write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let conn = Arc::new(Connection::new(
        addr.to_string(),
        selected_protocol,
        tx,
    ));
    let connection_id = conn.id;

    // Add to the connections map
    connections.lock().insert(connection_id, Arc::clone(&conn));

    // Notify connected
    let _ = event_tx.send(ServerEvent {
        event_type: DwebbleWSEventType::ClientConnected,
        connection_id,
        data: None,
        error: None,
    });

    tracing::info!("Client connected: {} (id: {})", addr, connection_id);

    // Spawn writer task
    let write = Arc::new(tokio::sync::Mutex::new(write));
    let write_handle = {
        let write = Arc::clone(&write);
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let mut w = write.lock().await;
                if w.send(msg).await.is_err() {
                    break;
                }
            }
        })
    };

    // Read messages
    while let Some(result) = read.next().await {
        match result {
            Ok(msg) => match msg {
                Message::Binary(data) => {
                    let _ = event_tx.send(ServerEvent {
                        event_type: DwebbleWSEventType::MessageReceived,
                        connection_id,
                        data: Some(data.to_vec()),
                        error: None,
                    });
                }
                Message::Text(text) => {
                    let _ = event_tx.send(ServerEvent {
                        event_type: DwebbleWSEventType::MessageReceived,
                        connection_id,
                        data: Some(text.as_bytes().to_vec()),
                        error: None,
                    });
                }
                Message::Ping(data) => {
                    let mut w = write.lock().await;
                    let _ = w.send(Message::Pong(data)).await;
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            },
            Err(e) => {
                tracing::error!("Read error from {}: {}", addr, e);
                let _ = event_tx.send(ServerEvent {
                    event_type: DwebbleWSEventType::Error,
                    connection_id,
                    data: None,
                    error: Some(e.to_string()),
                });
                break;
            }
        }
    }

    // Cleanup
    write_handle.abort();
    connections.lock().remove(&connection_id);

    let _ = event_tx.send(ServerEvent {
        event_type: DwebbleWSEventType::ClientDisconnected,
        connection_id,
        data: None,
        error: None,
    });

    tracing::info!("Client disconnected: {} (id: {})", addr, connection_id);

    Ok(())
}


impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
