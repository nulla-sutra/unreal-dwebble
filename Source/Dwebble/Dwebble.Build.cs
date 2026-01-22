// Copyright 2026 tarnishablec. All Rights Reserved.

using System.IO;
using UnrealBuildTool;
// ReSharper disable RedundantExplicitArrayCreation

public class Dwebble : ModuleRules
{
	public Dwebble(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;

		// Add Rust FFI header include path (dwebble-rws is now under Source/)
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

		if (!File.Exists(DllPath) || !File.Exists(LibPath)) return;

		// Add an import library for linking
		PublicAdditionalLibraries.Add(LibPath);
		
		// Setup delay load DLL
		PublicDelayLoadDLLs.Add(DllName);
		RuntimeDependencies.Add(DllPath);
	}
}