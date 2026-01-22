// Copyright 2024 tarnishablec. All Rights Reserved.

#include "WebSocketServer.h"
#include "dwebble_rws.h"
#include "HAL/PlatformProcess.h"

namespace DwebbleWS = Dwebble::WebSocket;

class FDwebbleWebSocketServerImpl : public DwebbleWS::IServer
{
public:
	explicit FDwebbleWebSocketServerImpl(const DwebbleWS::FServerConfig& InConfig)
		: Config(InConfig)
		  , ServerHandle(nullptr)
		  , bIsRunning(false)
	{
	}

	virtual ~FDwebbleWebSocketServerImpl() override
	{
		if (ServerHandle)
		{
			this->FDwebbleWebSocketServerImpl::Stop();
			dwebble_rws_server_destroy(ServerHandle);
			ServerHandle = nullptr;
		}
	}

	virtual DwebbleWS::EResult Start() override
	{
		if (bIsRunning)
		{
			return DwebbleWS::EResult::AlreadyRunning;
		}

		// Convert config to FFI struct
		const auto BindAddressAnsi = StringCast<ANSICHAR>(*Config.BindAddress);
		const auto SubprotocolsAnsi = StringCast<ANSICHAR>(*Config.Subprotocols);
		const auto CertPathAnsi = StringCast<ANSICHAR>(*Config.TlsCertPath);
		const auto KeyPathAnsi = StringCast<ANSICHAR>(*Config.TlsKeyPath);

		DwebbleWSServerConfig FfiConfig;
		FfiConfig.port = static_cast<uint16_t>(Config.Port);
		FfiConfig.bind_address = BindAddressAnsi.Get();
		FfiConfig.subprotocols = Config.Subprotocols.IsEmpty() ? nullptr : SubprotocolsAnsi.Get();
		FfiConfig.tls_cert_path = Config.TlsCertPath.IsEmpty() ? nullptr : CertPathAnsi.Get();
		FfiConfig.tls_key_path = Config.TlsKeyPath.IsEmpty() ? nullptr : KeyPathAnsi.Get();

		ServerHandle = dwebble_rws_server_create(&FfiConfig);
		if (!ServerHandle)
		{
			return DwebbleWS::EResult::RuntimeError;
		}

		const DwebbleWSResult Result = dwebble_rws_server_start(ServerHandle);
		if (Result == DwebbleWSResult::DWEBBLE_WS_RESULT_OK)
		{
			bIsRunning = true;
		}

		return ConvertResult(Result);
	}

	virtual DwebbleWS::EResult Stop() override
	{
		if (!bIsRunning || !ServerHandle)
		{
			return DwebbleWS::EResult::NotRunning;
		}

		const DwebbleWSResult Result = dwebble_rws_server_stop(ServerHandle);
		bIsRunning = false;

		return ConvertResult(Result);
	}

	virtual bool IsRunning() const override
	{
		return bIsRunning;
	}

	virtual int32 GetPort() const override
	{
		if (!ServerHandle) return 0;
		return dwebble_rws_server_get_port(ServerHandle);
	}

	virtual int32 GetConnectionCount() const override
	{
		if (!ServerHandle) return 0;
		return static_cast<int32>(dwebble_rws_server_get_connection_count(ServerHandle));
	}

	virtual FString Info() const override
	{
		if (!ServerHandle) return TEXT("");

		char* InfoStr = dwebble_rws_server_info(ServerHandle);
		if (!InfoStr) return TEXT("");

		FString Result = UTF8_TO_TCHAR(InfoStr);
		dwebble_rws_free_string(InfoStr);
		return Result;
	}

	virtual DwebbleWS::EResult Send(uint64 ConnectionId, const TArray<uint8>& Data) override
	{
		if (!ServerHandle) return DwebbleWS::EResult::InvalidHandle;

		const DwebbleWSResult Result = dwebble_rws_server_send(
			ServerHandle,
			ConnectionId,
			Data.GetData(),
			Data.Num()
		);

		return ConvertResult(Result);
	}

	virtual DwebbleWS::EResult SendText(uint64 ConnectionId, const FString& Text) override
	{
		if (!ServerHandle) return DwebbleWS::EResult::InvalidHandle;

		const auto TextAnsi = StringCast<ANSICHAR>(*Text);
		const DwebbleWSResult Result = dwebble_rws_server_send_text(
			ServerHandle,
			ConnectionId,
			TextAnsi.Get()
		);

		return ConvertResult(Result);
	}

	virtual DwebbleWS::EResult Disconnect(const uint64 ConnectionId) override
	{
		if (!ServerHandle) return DwebbleWS::EResult::InvalidHandle;

		const DwebbleWSResult Result = dwebble_rws_server_disconnect(ServerHandle, ConnectionId);
		return ConvertResult(Result);
	}

	virtual bool PollEvent(DwebbleWS::FEvent& OutEvent) override
	{
		if (!ServerHandle) return false;

		DwebbleWSEvent FfiEvent;
		if (!dwebble_rws_server_poll(ServerHandle, &FfiEvent))
		{
			return false;
		}

		OutEvent.EventType = ConvertEventType(FfiEvent.event_type);
		OutEvent.ConnectionId = FfiEvent.connection_id;

		if (FfiEvent.data && FfiEvent.data_len > 0)
		{
			OutEvent.Data.SetNumUninitialized(FfiEvent.data_len);
			FMemory::Memcpy(OutEvent.Data.GetData(), FfiEvent.data, FfiEvent.data_len);
		}
		else
		{
			OutEvent.Data.Empty();
		}

		if (FfiEvent.error_message)
		{
			OutEvent.ErrorMessage = UTF8_TO_TCHAR(FfiEvent.error_message);
		}
		else
		{
			OutEvent.ErrorMessage.Empty();
		}

		return true;
	}

private:
	static DwebbleWS::EResult ConvertResult(const DwebbleWSResult Result)
	{
		switch (Result)
		{
		case DwebbleWSResult::DWEBBLE_WS_RESULT_OK: return DwebbleWS::EResult::Ok;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_INVALID_HANDLE: return DwebbleWS::EResult::InvalidHandle;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_INVALID_PARAM: return DwebbleWS::EResult::InvalidParam;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_ALREADY_RUNNING: return DwebbleWS::EResult::AlreadyRunning;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_NOT_RUNNING: return DwebbleWS::EResult::NotRunning;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_BIND_FAILED: return DwebbleWS::EResult::BindFailed;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_TLS_ERROR: return DwebbleWS::EResult::TlsError;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_RUNTIME_ERROR: return DwebbleWS::EResult::RuntimeError;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_SEND_FAILED: return DwebbleWS::EResult::SendFailed;
		case DwebbleWSResult::DWEBBLE_WS_RESULT_CONNECTION_CLOSED: return DwebbleWS::EResult::ConnectionClosed;
		default: return DwebbleWS::EResult::RuntimeError;
		}
	}

	static DwebbleWS::EEventType ConvertEventType(DwebbleWSEventType Type)
	{
		switch (Type)
		{
		case DwebbleWSEventType::DWEBBLE_WS_EVENT_TYPE_NONE: return DwebbleWS::EEventType::None;
		case DwebbleWSEventType::DWEBBLE_WS_EVENT_TYPE_CLIENT_CONNECTED: return DwebbleWS::EEventType::ClientConnected;
		case DwebbleWSEventType::DWEBBLE_WS_EVENT_TYPE_CLIENT_DISCONNECTED: return DwebbleWS::EEventType::ClientDisconnected;
		case DwebbleWSEventType::DWEBBLE_WS_EVENT_TYPE_MESSAGE_RECEIVED: return DwebbleWS::EEventType::MessageReceived;
		case DwebbleWSEventType::DWEBBLE_WS_EVENT_TYPE_ERROR: return DwebbleWS::EEventType::Error;
		default: return DwebbleWS::EEventType::None;
		}
	}

	DwebbleWS::FServerConfig Config;
	DwebbleWSServerHandle ServerHandle;
	bool bIsRunning;
};

TSharedPtr<DwebbleWS::IServer> DwebbleWS::IServer::Create(const FServerConfig& Config)
{
	return MakeShared<FDwebbleWebSocketServerImpl>(Config);
}
