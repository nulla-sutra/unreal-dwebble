// Copyright 2024 tarnishablec. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "DwebbleTypes.h"

namespace Dwebble::WebSocket
{
	/**
	 * WebSocket Server interface
	 */
	class DWEBBLE_API IServer
	{
	public:
		virtual ~IServer() = default;

		/** Create a new WebSocket server instance */
		static TSharedPtr<IServer> Create(const FServerConfig& Config);

		/** Start the server */
		virtual EResult Start() = 0;

		/** Stop the server */
		virtual EResult Stop() = 0;

		/** Check if the server is running */
		virtual bool IsRunning() const = 0;

		/** Get the actual port the server is listening to */
		virtual int32 GetPort() const = 0;

		/** Get the number of active connections */
		virtual int32 GetConnectionCount() const = 0;

		/** Get server info string (address:port) */
		virtual FString Info() const = 0;

		/** Send binary data to a connection */
		virtual EResult Send(uint64 ConnectionId, const TArray<uint8>& Data) = 0;

		/** Send text data to a connection */
		virtual EResult SendText(uint64 ConnectionId, const FString& Text) = 0;

		/** Disconnect a connection */
		virtual EResult Disconnect(uint64 ConnectionId) = 0;

		/** Poll for events (call from Tick) */
		virtual bool PollEvent(FEvent& OutEvent) = 0;

		// Event delegates
		FOnClientConnected OnClientConnected;
		FOnClientDisconnected OnClientDisconnected;
		FOnMessageReceived OnMessageReceived;
		FOnError OnError;
	};
}
