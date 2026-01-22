//! FFI types shared between Rust and C++

use std::ffi::{c_char, c_void};

/// Result codes for WebSocket FFI operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwebbleWSResult {
    Ok = 0,
    InvalidHandle = 1,
    InvalidParam = 2,
    AlreadyRunning = 3,
    NotRunning = 4,
    BindFailed = 5,
    TlsError = 6,
    RuntimeError = 7,
    SendFailed = 8,
    ConnectionClosed = 9,
}

/// WebSocket event types for polling
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DwebbleWSEventType {
    None = 0,
    ClientConnected = 1,
    ClientDisconnected = 2,
    MessageReceived = 3,
    Error = 4,
}

/// WebSocket server configuration passed from C++
#[repr(C)]
pub struct DwebbleWSServerConfig {
    /// Port to listen on (0 for auto)
    pub port: u16,
    /// Bind address (null-terminated UTF-8)
    pub bind_address: *const c_char,
    /// Subprotocols (null-terminated, comma-separated)
    pub subprotocols: *const c_char,
    /// TLS certificate path (null for no TLS)
    pub tls_cert_path: *const c_char,
    /// TLS private key path
    pub tls_key_path: *const c_char,
}

/// WebSocket event data returned from polling
#[repr(C)]
pub struct DwebbleWSEvent {
    pub event_type: DwebbleWSEventType,
    /// Connection ID (valid for Connected/Disconnected/MessageReceived)
    pub connection_id: u64,
    /// Message data pointer (valid for MessageReceived)
    pub data: *const u8,
    /// Message data length
    pub data_len: usize,
    /// Error message (valid for Error, null-terminated)
    pub error_message: *const c_char,
}

impl Default for DwebbleWSEvent {
    fn default() -> Self {
        Self {
            event_type: DwebbleWSEventType::None,
            connection_id: 0,
            data: std::ptr::null(),
            data_len: 0,
            error_message: std::ptr::null(),
        }
    }
}

/// WebSocket server handle (opaque pointer)
pub type DwebbleWSServerHandle = *mut c_void;

/// WebSocket connection handle
pub type DwebbleWSConnectionId = u64;
