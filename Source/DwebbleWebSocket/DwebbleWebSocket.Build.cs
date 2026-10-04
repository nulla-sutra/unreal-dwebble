// Copyright 2026 tarnishablec. All Rights Reserved.

using System.IO;
using UnrealBuildTool;
// ReSharper disable RedundantExplicitArrayCreation

public class DwebbleWebSocket : ModuleRules
{
	public DwebbleWebSocket(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;

		// Add Rust FFI header `include` (dwebble-rws is now under Source/)
		var RustIncludeDir = Path.Combine(PluginDirectory, "Source", "dwebble-rws", "include");
		PublicIncludePaths.Add(RustIncludeDir);

		PublicDependencyModuleNames.AddRange(
			new string[]
			{
				"Core",
			}
		);

		PrivateDependencyModuleNames.AddRange(
			new string[]
			{
				"CoreUObject",
				"Engine",
				"Projects",
			}
		);

		// Find Rust DLL and import a library
		var BinariesDir = Path.Combine(PluginDirectory, "Binaries", "Win64");
		const string DllName = "dwebble_rws.dll";
		const string LibName = "dwebble_rws.dll.lib";
		var DllPath = Path.Combine(BinariesDir, DllName);
		var LibPath = Path.Combine(BinariesDir, LibName);

		// Missing artifacts must not silently remove the Rust FFI symbols from the link.
		foreach (var RequiredPath in new[] { DllPath, LibPath })
		{
			if (!File.Exists(RequiredPath))
			{
				var RustSourceDir = Path.Combine(PluginDirectory, "Source", "dwebble-rws");
				throw new BuildException(
					$"Dwebble Rust artifact is missing: '{RequiredPath}'. " +
					$"Run 'cargo make --env TARGET=x86_64-pc-windows-msvc release' in '{RustSourceDir}' " +
					$"to build and copy both '{DllName}' and '{LibName}' to '{BinariesDir}' before building Unreal.");
			}
		}

		// Add an import library for linking
		PublicAdditionalLibraries.Add(LibPath);
		
		// Setup delay load DLL
		PublicDelayLoadDLLs.Add(DllName);
		RuntimeDependencies.Add(DllPath);
	}
}