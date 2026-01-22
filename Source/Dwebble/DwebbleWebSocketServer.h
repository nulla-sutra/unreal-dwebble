// Copyright 2024 tarnishablec. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "DwebbleTypes.h"

/**
 * WebSocket Server interface
 */
class DWEBBLE_API IDwebbleWebSocketServer
{
public:
	virtual ~IDwebbleWebSocketServer() = default;

	/** Create a new WebSocket server instance */
	static TSharedPtr<IDwebbleWebSocketServer> Create(const FDwebbleServerConfig& Config);

	/** Start the server */
	virtual EDwebbleResult Start() = 0;

	/** Stop the server */
	virtual EDwebbleResult Stop() = 0;

	/** Check if server is running */
	virtual bool IsRunning() const = 0;

	/** Get the actual port the server is listening on */
	virtual int32 GetPort() const = 0;

	/** Get number of active connections */
	virtual int32 GetConnectionCount() const = 0;

	/** Get server info string (address:port) */
	virtual FString Info() const = 0;

	/** Send binary data to a connection */
	virtual EDwebbleResult Send(uint64 ConnectionId, const TArray<uint8>& Data) = 0;

	/** Send text data to a connection */
	virtual EDwebbleResult SendText(uint64 ConnectionId, const FString& Text) = 0;

	/** Disconnect a connection */
	virtual EDwebbleResult Disconnect(uint64 ConnectionId) = 0;

	/** Poll for events (call from Tick) */
	virtual bool PollEvent(FDwebbleEvent& OutEvent) = 0;

	// Event delegates
	FOnDwebbleClientConnected OnClientConnected;
	FOnDwebbleClientDisconnected OnClientDisconnected;
	FOnDwebbleMessageReceived OnMessageReceived;
	FOnDwebbleError OnError;
};
