# Dwebble

High-performance WebSocket plugin for Unreal Engine, powered by Rust.

> ⚠️ **Note**: Currently only tested on Windows. Other platforms (Linux, macOS) may work but are not yet verified.

## Why Dwebble exists?

Unreal Engine's built-in [WebSocketNetworking](https://github.com/EpicGames/UnrealEngine/tree/release/Engine/Plugins/Experimental/WebSocketNetworking) plugin has significant limitations:

1. **No Subprotocol Support** - `IWebSocketServer::Init()` doesn't accept subprotocol parameters, making it incompatible with protocols like MCP (Model Context Protocol) that require specific subprotocols.

2. **No Actual Port Retrieval** - `IWebSocketServer` supports ephemeral port (port 0), but provides no API to retrieve the actual assigned port after binding. See [related PR](https://github.com/EpicGames/UnrealEngine/pull/14308).

<details>
    <summary style="background-color:black; color:black;">
    </summary>
    Why rust? Honestly, I just don't want to write a single extra line of C++.
</details>

## Solution

Dwebble provides a clean, standalone WebSocket server implementation using Rust's excellent async ecosystem:

- **[tokio](https://tokio.rs/)** - Industry-standard async runtime
- **[tungstenite](https://github.com/snapview/tungstenite-rs)** - Native Rust WebSocket implementation
- **[rustls](https://github.com/rustls/rustls)** - Modern TLS implementation (no OpenSSL dependency)

## Features

- ✅ **Subprotocol Support** - Full WebSocket subprotocol negotiation
- ✅ **Native TLS** - Built-in TLS with rustls (ring crypto)
- ✅ **Cross-Platform** - Windows x64/ARM64 support
- ✅ **Zero UE Dependencies** – Standalone, doesn't interfere with existing networking
- ✅ **High Performance** – Rust's zero-cost abstractions and tokio's efficient async I/O
- ✅ **Simple API** - Clean C++ interface with UE-friendly types

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Unreal Engine                        │
├─────────────────────────────────────────────────────────┤
│  DwebbleWebSocket Module (C++)                          │
│  ├── Dwebble::WebSocket::IServer                        │
│  ├── Dwebble::WebSocket::FServerConfig                  │
│  └── Dwebble::WebSocket::FEvent                         │
├─────────────────────────────────────────────────────────┤
│  FFI Layer (C ABI)                                      │
│  └── dwebble_rws.dll / dwebble_rws.dll.lib              │
├─────────────────────────────────────────────────────────┤
│  Rust Core (dwebble-rws)                                │
│  ├── tokio (async runtime)                              │
│  ├── tokio-tungstenite (WebSocket)                      │
│  └── tokio-rustls (TLS)                                 │
└─────────────────────────────────────────────────────────┘
```

## Usage

### Basic Server

```cpp
#include "WebSocketServer.h"

// Create server configuration
Dwebble::WebSocket::FServerConfig Config;
Config.Port = 8080;  // Use 0 for auto-assign
Config.BindAddress = TEXT("127.0.0.1");
Config.Subprotocols = { TEXT("mcp") };  // Optional subprotocols

// Create and start server
TSharedPtr<Dwebble::WebSocket::IServer> Server = Dwebble::WebSocket::IServer::Create(Config);
Server->Start();

// Get actual port (useful when Port = 0)
int32 ActualPort = Server->GetPort();
UE_LOG(LogTemp, Log, TEXT("Server listening on %s"), *Server->Info());
```

### Processing Events

```cpp
// In your Tick function
void UMySubsystem::Tick(float DeltaTime)
{
    if (!Server || !Server->IsRunning()) return;
    
    Dwebble::WebSocket::FEvent Event;
    while (Server->PollEvent(Event))
    {
        switch (Event.EventType)
        {
        case Dwebble::WebSocket::EEventType::ClientConnected:
            UE_LOG(LogTemp, Log, TEXT("Client connected: %llu"), Event.ConnectionId);
            break;
            
        case Dwebble::WebSocket::EEventType::ClientDisconnected:
            UE_LOG(LogTemp, Log, TEXT("Client disconnected: %llu"), Event.ConnectionId);
            break;
            
        case Dwebble::WebSocket::EEventType::MessageReceived:
            HandleMessage(Event.ConnectionId, Event.Data);
            break;
            
        case Dwebble::WebSocket::EEventType::Error:
            UE_LOG(LogTemp, Error, TEXT("Error: %s"), *Event.ErrorMessage);
            break;
        }
    }
}
```

### Sending Messages

```cpp
// Send binary data
TArray<uint8> BinaryData = /* ... */;
Server->Send(ConnectionId, BinaryData);

// Send text
Server->SendText(ConnectionId, TEXT("Hello, Client!"));

// Disconnect a client
Server->Disconnect(ConnectionId);
```

## Building the Rust Library

Requires:
- Rust toolchain (rustup), with the `x86_64-pc-windows-msvc` target installed
- Visual Studio C++ build tools and the Windows SDK
- cargo-make (`cargo install cargo-make`)

Close Unreal Editor before replacing a DLL it has loaded. From the plugin directory, build the Rust library before building the Unreal project:

```powershell
cd Source/dwebble-rws

# Debug build for Windows x64 (MSVC)
cargo make --env TARGET=x86_64-pc-windows-msvc dev

# Release build for Windows x64 (MSVC)
cargo make --env TARGET=x86_64-pc-windows-msvc release
```

Choose either build configuration. The crate uses `cdylib`, producing a DLL and its import library:

- `dwebble_rws.dll`: the Rust implementation, loaded at runtime.
- `dwebble_rws.dll.lib`: the MSVC import library used when linking the Unreal module.

The task copies both files from `Source/dwebble-rws/target/x86_64-pc-windows-msvc/<debug-or-release>/` to the plugin's `Binaries/Win64/` directory. It fails if either artifact is missing or a copy fails. Running `cargo build` alone does not perform this copy.

Both files must exist before building the Unreal project. `DwebbleWebSocket.Build.cs` checks for them and reports the missing path and build command. The module delay-loads the DLL, and `RuntimeDependencies` stages it for packaged builds. Distribute the DLL with the plugin; the import library is needed to build the Unreal module, but not to run it.

## Module Dependencies

In your module's `Build.cs`:

```csharp
PublicDependencyModuleNames.AddRange(new string[] {
    "DwebbleWebSocket"
});
```

## License

Copyright 2026 tarnishablec. All Rights Reserved.
