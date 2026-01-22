//! FFI entry points for dwebble-rws
//!
//! All functions use the `dwebble_rws_` prefix to avoid naming conflicts.

mod connection;
mod server;
mod tls;
mod types;

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use parking_lot::Mutex;

use crate::server::{Server, ServerConfig};
use crate::tls::TlsConfig;
use crate::types::*;

/// Stored event data for FFI (to keep strings alive)
struct EventData {
    #[allow(dead_code)]
    data: Vec<u8>,
    error: CString,
}

static CURRENT_EVENT_DATA: Mutex<Option<EventData>> = Mutex::new(None);

/// Initialize tracing (optional, call once)
#[no_mangle]
pub extern "C" fn dwebble_rws_init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Create a new WebSocket server with the given configuration.
/// Returns a server handle or null on failure.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_create(config: *const DwebbleWSServerConfig) -> DwebbleWSServerHandle {
    if config.is_null() {
        return ptr::null_mut();
    }

    let config = unsafe { &*config };

    let bind_address = if config.bind_address.is_null() {
        "127.0.0.1".to_string()
    } else {
        unsafe { CStr::from_ptr(config.bind_address) }
            .to_string_lossy()
            .into_owned()
    };

    let subprotocols = if config.subprotocols.is_null() {
        vec![]
    } else {
        let s = unsafe { CStr::from_ptr(config.subprotocols) }
            .to_string_lossy();
        s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let tls = if !config.tls_cert_path.is_null() && !config.tls_key_path.is_null() {
        let cert_path = unsafe { CStr::from_ptr(config.tls_cert_path) }
            .to_string_lossy();
        let key_path = unsafe { CStr::from_ptr(config.tls_key_path) }
            .to_string_lossy();

        match TlsConfig::from_pem_files(&cert_path, &key_path) {
            Ok(tls) => Some(tls),
            Err(e) => {
                tracing::error!("TLS configuration error: {}", e);
                return ptr::null_mut();
            }
        }
    } else {
        None
    };

    let server_config = ServerConfig {
        port: config.port,
        bind_address,
        subprotocols,
        tls,
    };

    let server = Box::new(Server::new(server_config));
    Box::into_raw(server) as DwebbleWSServerHandle
}

/// Destroy a server handle and free resources.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_destroy(handle: DwebbleWSServerHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut Server);
        }
    }
}

/// Start the WebSocket server.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_start(handle: DwebbleWSServerHandle) -> DwebbleWSResult {
    if handle.is_null() {
        return DwebbleWSResult::InvalidHandle;
    }

    let server = unsafe { &mut *(handle as *mut Server) };
    server.start()
}

/// Stop the WebSocket server.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_stop(handle: DwebbleWSServerHandle) -> DwebbleWSResult {
    if handle.is_null() {
        return DwebbleWSResult::InvalidHandle;
    }

    let server = unsafe { &mut *(handle as *mut Server) };
    server.stop()
}

/// Poll for the next event. Returns the event in the out parameter.
/// Returns true if an event was available, false otherwise.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_poll(
    handle: DwebbleWSServerHandle,
    out_event: *mut DwebbleWSEvent,
) -> bool {
    if handle.is_null() || out_event.is_null() {
        return false;
    }

    let server = unsafe { &*(handle as *const Server) };

    if let Some(event) = server.poll_event() {
        let mut event_data = CURRENT_EVENT_DATA.lock();

        let data_ptr: *const u8;
        let data_len: usize;
        let error_ptr: *const c_char;

        if let Some(data) = event.data {
            data_ptr = data.as_ptr();
            data_len = data.len();
            *event_data = Some(EventData {
                data,
                error: CString::default(),
            });
        } else {
            data_ptr = ptr::null();
            data_len = 0;
            *event_data = None;
        }

        if let Some(error) = event.error {
            let c_error = CString::new(error).unwrap_or_default();
            error_ptr = c_error.as_ptr();
            if let Some(ref mut ed) = *event_data {
                ed.error = c_error;
            } else {
                *event_data = Some(EventData {
                    data: vec![],
                    error: c_error,
                });
            }
        } else {
            error_ptr = ptr::null();
        }

        unsafe {
            (*out_event).event_type = event.event_type;
            (*out_event).connection_id = event.connection_id;
            (*out_event).data = data_ptr;
            (*out_event).data_len = data_len;
            (*out_event).error_message = error_ptr;
        }

        true
    } else {
        unsafe {
            *out_event = DwebbleWSEvent::default();
        }
        false
    }
}

/// Send binary data to a specific connection.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_send(
    handle: DwebbleWSServerHandle,
    connection_id: DwebbleWSConnectionId,
    data: *const u8,
    data_len: usize,
) -> DwebbleWSResult {
    if handle.is_null() || data.is_null() {
        return DwebbleWSResult::InvalidParam;
    }

    let server = unsafe { &*(handle as *const Server) };
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };

    server.send(connection_id, data_slice)
}

/// Send text data to a specific connection.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_send_text(
    handle: DwebbleWSServerHandle,
    connection_id: DwebbleWSConnectionId,
    text: *const c_char,
) -> DwebbleWSResult {
    if handle.is_null() || text.is_null() {
        return DwebbleWSResult::InvalidParam;
    }

    let server = unsafe { &*(handle as *const Server) };
    let text_str = unsafe { CStr::from_ptr(text) }.to_string_lossy();

    server.send_text(connection_id, &text_str)
}

/// Disconnect a specific connection.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_disconnect(
    handle: DwebbleWSServerHandle,
    connection_id: DwebbleWSConnectionId,
) -> DwebbleWSResult {
    if handle.is_null() {
        return DwebbleWSResult::InvalidHandle;
    }

    let server = unsafe { &*(handle as *const Server) };
    server.disconnect(connection_id)
}

/// Get the actual port the server is listening to.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_get_port(handle: DwebbleWSServerHandle) -> u16 {
    if handle.is_null() {
        return 0;
    }

    let server = unsafe { &*(handle as *const Server) };
    server.get_actual_port()
}

/// Get the number of active connections.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_get_connection_count(handle: DwebbleWSServerHandle) -> usize {
    if handle.is_null() {
        return 0;
    }

    let server = unsafe { &*(handle as *const Server) };
    server.get_connection_count()
}

/// Get server info string. Caller must free with `dwebble_rws_free_string`.
#[no_mangle]
pub extern "C" fn dwebble_rws_server_info(handle: DwebbleWSServerHandle) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }

    let server = unsafe { &*(handle as *const Server) };
    let info = server.info();

    match CString::new(info) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string allocated by this library.
#[no_mangle]
pub extern "C" fn dwebble_rws_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
