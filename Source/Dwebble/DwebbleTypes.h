// Copyright 2024 tarnishablec. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"

// Forward declare FFI types (defined in DwebbleFfi.h)
struct DwebbleServerConfig;
struct DwebbleEvent;
enum class DwebbleResult : int32;
enum class DwebbleEventType : int32;

/**
 * WebSocket server configuration for Dwebble
 */
struct DWEBBLE_API FDwebbleServerConfig
{
	/** Port to listen on. Use 0 for automatic port selection. */
	int32 Port = 0;

	/** Address to bind to */
	FString BindAddress = TEXT("127.0.0.1");

	/** Subprotocols to support (comma-separated) */
	FString Subprotocols;

	/** Path to TLS certificate file (PEM format). Empty for no TLS. */
	FString TlsCertPath;

	/** Path to TLS private key file (PEM format) */
	FString TlsKeyPath;

	/** Default configuration */
	static FDwebbleServerConfig Default() { return FDwebbleServerConfig(); }
};

/**
 * Event types from WebSocket server
 */
UENUM()
enum class EDwebbleEventType : uint8
{
	None = 0,
	ClientConnected = 1,
	ClientDisconnected = 2,
	MessageReceived = 3,
	Error = 4,
};

/**
 * Result codes from WebSocket operations
 */
UENUM()
enum class EDwebbleResult : uint8
{
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
};

/**
 * WebSocket event data
 */
struct DWEBBLE_API FDwebbleEvent
{
	EDwebbleEventType EventType = EDwebbleEventType::None;
	uint64 ConnectionId = 0;
	TArray<uint8> Data;
	FString ErrorMessage;
};

// Delegate types
DECLARE_DELEGATE_OneParam(FOnDwebbleClientConnected, uint64 /* ConnectionId */);
DECLARE_DELEGATE_OneParam(FOnDwebbleClientDisconnected, uint64 /* ConnectionId */);
DECLARE_DELEGATE_TwoParams(FOnDwebbleMessageReceived, uint64 /* ConnectionId */, const TArray<uint8>& /* Data */);
DECLARE_DELEGATE_TwoParams(FOnDwebbleError, uint64 /* ConnectionId */, const FString& /* ErrorMessage */);
