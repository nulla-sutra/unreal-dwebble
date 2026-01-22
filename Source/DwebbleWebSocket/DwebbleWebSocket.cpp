// Copyright 2024 tarnishablec. All Rights Reserved.

#include "DwebbleWebSocket.h"
#include "Interfaces/IPluginManager.h"
#include "Misc/Paths.h"
#include "HAL/PlatformProcess.h"

#define LOCTEXT_NAMESPACE "FDwebbleModule"

void* GDwebbleRwsDllHandle = nullptr;

void FDwebbleWebSocketModule::StartupModule()
{
	// Load the Rust DLL
	const FString PluginDir = IPluginManager::Get().FindPlugin(TEXT("Dwebble"))->GetBaseDir();
	const FString DllPath = FPaths::Combine(PluginDir, TEXT("Binaries/Win64/dwebble_rws.dll"));

	if (FPaths::FileExists(DllPath))
	{
		GDwebbleRwsDllHandle = FPlatformProcess::GetDllHandle(*DllPath);
		if (GDwebbleRwsDllHandle)
		{
			UE_LOG(LogTemp, Log, TEXT("Dwebble: Loaded dwebble_rws.dll from %s"), *DllPath);
		}
		else
		{
			UE_LOG(LogTemp, Error, TEXT("Dwebble: Failed to load dwebble_rws.dll from %s"), *DllPath);
		}
	}
	else
	{
		UE_LOG(LogTemp, Warning, TEXT("Dwebble: dwebble_rws.dll not found at %s"), *DllPath);
	}
}

void FDwebbleWebSocketModule::ShutdownModule()
{
	if (GDwebbleRwsDllHandle)
	{
		FPlatformProcess::FreeDllHandle(GDwebbleRwsDllHandle);
		GDwebbleRwsDllHandle = nullptr;
		UE_LOG(LogTemp, Log, TEXT("Dwebble: Unloaded dwebble_rws.dll"));
	}
}

#undef LOCTEXT_NAMESPACE

IMPLEMENT_MODULE(FDwebbleWebSocketModule, DwebbleWebSocket)