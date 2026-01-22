// Copyright 2024 tarnishablec. All Rights Reserved.

#include "DwebbleWebSocketServer.h"
#include "dwebble_rws.h"
#include "HAL/PlatformProcess.h"
#include "Misc/Paths.h"

class FDwebbleWebSocketServerImpl : public IDwebbleWebSocketServer
{
public:
	explicit FDwebbleWebSocketServerImpl(const FDwebbleServerConfig& InConfig)
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

	virtual EDwebbleResult Start() override
	{
		if (bIsRunning)
		{
			return EDwebbleResult::AlreadyRunning;
		}

		// Convert config to FFI struct
		const auto BindAddressAnsi = StringCast<ANSICHAR>(*Config.BindAddress);
		const auto SubprotocolsAnsi = StringCast<ANSICHAR>(*Config.Subprotocols);
		const auto CertPathAnsi = StringCast<ANSICHAR>(*Config.TlsCertPath);
		const auto KeyPathAnsi = StringCast<ANSICHAR>(*Config.TlsKeyPath);

		DwebbleServerConfig FfiConfig;
		FfiConfig.port = static_cast<uint16_t>(Config.Port);
		FfiConfig.bind_address = BindAddressAnsi.Get();
		FfiConfig.subprotocols = Config.Subprotocols.IsEmpty() ? nullptr : SubprotocolsAnsi.Get();
		FfiConfig.tls_cert_path = Config.TlsCertPath.IsEmpty() ? nullptr : CertPathAnsi.Get();
		FfiConfig.tls_key_path = Config.TlsKeyPath.IsEmpty() ? nullptr : KeyPathAnsi.Get();

		ServerHandle = dwebble_rws_server_create(&FfiConfig);
		if (!ServerHandle)
		{
			return EDwebbleResult::RuntimeError;
		}

		const DwebbleResult Result = dwebble_rws_server_start(ServerHandle);
		if (Result == DwebbleResult::DWEBBLE_RESULT_OK)
		{
			bIsRunning = true;
		}

		return ConvertResult(Result);
	}

	virtual EDwebbleResult Stop() override
	{
		if (!bIsRunning || !ServerHandle)
		{
			return EDwebbleResult::NotRunning;
		}

		const DwebbleResult Result = dwebble_rws_server_stop(ServerHandle);
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
		return static_cast<int32>(dwebble_rws_server_get_port(ServerHandle));
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

	virtual EDwebbleResult Send(uint64 ConnectionId, const TArray<uint8>& Data) override
	{
		if (!ServerHandle) return EDwebbleResult::InvalidHandle;

		const DwebbleResult Result = dwebble_rws_server_send(
			ServerHandle,
			ConnectionId,
			Data.GetData(),
			Data.Num()
		);

		return ConvertResult(Result);
	}

	virtual EDwebbleResult SendText(uint64 ConnectionId, const FString& Text) override
	{
		if (!ServerHandle) return EDwebbleResult::InvalidHandle;

		const auto TextAnsi = StringCast<ANSICHAR>(*Text);
		const DwebbleResult Result = dwebble_rws_server_send_text(
			ServerHandle,
			ConnectionId,
			TextAnsi.Get()
		);

		return ConvertResult(Result);
	}

	virtual EDwebbleResult Disconnect(uint64 ConnectionId) override
	{
		if (!ServerHandle) return EDwebbleResult::InvalidHandle;

		const DwebbleResult Result = dwebble_rws_server_disconnect(ServerHandle, ConnectionId);
		return ConvertResult(Result);
	}

	virtual bool PollEvent(FDwebbleEvent& OutEvent) override
	{
		if (!ServerHandle) return false;

		DwebbleEvent FfiEvent;
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
	static EDwebbleResult ConvertResult(DwebbleResult Result)
	{
		switch (Result)
		{
		case DwebbleResult::DWEBBLE_RESULT_OK: return EDwebbleResult::Ok;
		case DwebbleResult::DWEBBLE_RESULT_INVALID_HANDLE: return EDwebbleResult::InvalidHandle;
		case DwebbleResult::DWEBBLE_RESULT_INVALID_PARAM: return EDwebbleResult::InvalidParam;
		case DwebbleResult::DWEBBLE_RESULT_ALREADY_RUNNING: return EDwebbleResult::AlreadyRunning;
		case DwebbleResult::DWEBBLE_RESULT_NOT_RUNNING: return EDwebbleResult::NotRunning;
		case DwebbleResult::DWEBBLE_RESULT_BIND_FAILED: return EDwebbleResult::BindFailed;
		case DwebbleResult::DWEBBLE_RESULT_TLS_ERROR: return EDwebbleResult::TlsError;
		case DwebbleResult::DWEBBLE_RESULT_RUNTIME_ERROR: return EDwebbleResult::RuntimeError;
		case DwebbleResult::DWEBBLE_RESULT_SEND_FAILED: return EDwebbleResult::SendFailed;
		case DwebbleResult::DWEBBLE_RESULT_CONNECTION_CLOSED: return EDwebbleResult::ConnectionClosed;
		default: return EDwebbleResult::RuntimeError;
		}
	}

	static EDwebbleEventType ConvertEventType(DwebbleEventType Type)
	{
		switch (Type)
		{
		case DwebbleEventType::DWEBBLE_EVENT_TYPE_NONE: return EDwebbleEventType::None;
		case DwebbleEventType::DWEBBLE_EVENT_TYPE_CLIENT_CONNECTED: return EDwebbleEventType::ClientConnected;
		case DwebbleEventType::DWEBBLE_EVENT_TYPE_CLIENT_DISCONNECTED: return EDwebbleEventType::ClientDisconnected;
		case DwebbleEventType::DWEBBLE_EVENT_TYPE_MESSAGE_RECEIVED: return EDwebbleEventType::MessageReceived;
		case DwebbleEventType::DWEBBLE_EVENT_TYPE_ERROR: return EDwebbleEventType::Error;
		default: return EDwebbleEventType::None;
		}
	}

	FDwebbleServerConfig Config;
	DwebbleServerHandle ServerHandle;
	bool bIsRunning;
};

TSharedPtr<IDwebbleWebSocketServer> IDwebbleWebSocketServer::Create(const FDwebbleServerConfig& Config)
{
	return MakeShared<FDwebbleWebSocketServerImpl>(Config);
}
