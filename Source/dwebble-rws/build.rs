//! Build script for dwebble-rws
//!
//! Generates a C++ header using cbindgen and copies output to appropriate locations.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_path = PathBuf::from(&crate_dir);
    let profile = env::var("PROFILE").unwrap(); // "debug" or "release"

    // Generate C++ header with cbindgen
    generate_bindings(&crate_path);

    // Copy DLL and LIB to Binaries/Win64 (post-build)
    copy_binaries_to_plugin(&crate_path, &profile);
}

fn generate_bindings(crate_path: &PathBuf) {
    let config_path = crate_path.join("cbindgen.toml");
    let output_path = crate_path.join("include").join("dwebble_rws.h");

    // Ensure the include directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let config = cbindgen::Config::from_file(&config_path).unwrap_or_default();

    cbindgen::Builder::new()
        .with_crate(crate_path)
        .with_config(config)
        .generate()
        .map(|bindings| {
            bindings.write_to_file(&output_path);
            println!("cargo:warning=Generated header: {}", output_path.display());
        })
        .unwrap_or_else(|e| {
            println!("cargo:warning=cbindgen failed: {}", e);
        });
}

fn copy_binaries_to_plugin(crate_path: &PathBuf, profile: &str) {
    let target_dir = crate_path.join("target").join(profile);
    // crate_path is Source/dwebble-rws, so we need to go up two levels to reach PluginDirectory
    let plugin_dir = crate_path.parent().unwrap().parent().unwrap();
    let binaries_dir = plugin_dir.join("Binaries").join("Win64");

    // Ensure the Binaries / Win64 directory exists
    fs::create_dir_all(&binaries_dir).ok();

    // Files to copy
    let files = [
        ("dwebble_rws.dll", "dwebble_rws.dll"),
        ("dwebble_rws.dll.lib", "dwebble_rws.dll.lib"),
    ];

    for (src_name, dst_name) in files {
        let src = target_dir.join(src_name);
        let dst = binaries_dir.join(dst_name);

        if src.exists() {
            match fs::copy(&src, &dst) {
                Ok(_) => println!("cargo:warning=Copied {} -> {}", src.display(), dst.display()),
                Err(e) => println!("cargo:warning=Failed to copy {}: {}", src_name, e),
            }
        }
    }
}
