// Copyright 2024 tarnishablec. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "DwebbleTypes.generated.h"

// ============================================================================
// Global scope definitions for UE reflection (UENUM/USTRUCT)
// Use Dwebble::WebSocket:: aliases in code
// ============================================================================

/**
 * Event types from WebSocket server
 */
UENUM(BlueprintType)
enum class EDwebbleWSEventType : uint8
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
UENUM(BlueprintType)
enum class EDwebbleWSResult : uint8
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
 * WebSocket server configuration
 */
USTRUCT(BlueprintType)
struct DWEBBLEWEBSOCKET_API FDwebbleWSServerConfig
{
	GENERATED_BODY()

	/** Port to listen on. Use 0 for automatic port selection. */
	UPROPERTY(EditAnywhere, BlueprintReadWrite)
	int32 Port = 0;

	/** Address to bind to */
	UPROPERTY(EditAnywhere, BlueprintReadWrite)
	FString BindAddress = TEXT("127.0.0.1");

	/** Subprotocols to support */
	UPROPERTY(EditAnywhere, BlueprintReadWrite)
	TArray<FString> Subprotocols;

	/** Path to a TLS certificate file (PEM format). Empty for no TLS. */
	UPROPERTY(EditAnywhere, BlueprintReadWrite)
	FString TlsCertPath;

	/** Path to TLS private key file (PEM format) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite)
	FString TlsKeyPath;

	/** Default configuration */
	static FDwebbleWSServerConfig Default() { return FDwebbleWSServerConfig(); }
};

/**
 * WebSocket event data
 */
USTRUCT(BlueprintType)
struct DWEBBLEWEBSOCKET_API FDwebbleWSEvent
{
	GENERATED_BODY()

	UPROPERTY(BlueprintReadOnly)
	EDwebbleWSEventType EventType = EDwebbleWSEventType::None;

	uint64 ConnectionId = 0;

	UPROPERTY(BlueprintReadOnly)
	TArray<uint8> Data;

	UPROPERTY(BlueprintReadOnly)
	FString ErrorMessage;
};

// Delegate types (global scope for UE macro compatibility)
DECLARE_DELEGATE_OneParam(FDwebbleWSOnClientConnected, uint64 /* ConnectionId */);
DECLARE_DELEGATE_OneParam(FDwebbleWSOnClientDisconnected, uint64 /* ConnectionId */);
DECLARE_DELEGATE_TwoParams(FDwebbleWSOnMessageReceived, uint64 /* ConnectionId */, const TArray<uint8>& /* Data */);
DECLARE_DELEGATE_TwoParams(FDwebbleWSOnError, uint64 /* ConnectionId */, const FString& /* ErrorMessage */);

// ============================================================================
// Dwebble::WebSocket namespace aliases
// ============================================================================

namespace Dwebble::WebSocket
{
	// Type aliases for cleaner usage
	using EEventType = EDwebbleWSEventType;
	using EResult = EDwebbleWSResult;
	using FServerConfig = FDwebbleWSServerConfig;
	using FEvent = FDwebbleWSEvent;

	// Delegate aliases
	using FOnClientConnected = FDwebbleWSOnClientConnected;
	using FOnClientDisconnected = FDwebbleWSOnClientDisconnected;
	using FOnMessageReceived = FDwebbleWSOnMessageReceived;
	using FOnError = FDwebbleWSOnError;
}
