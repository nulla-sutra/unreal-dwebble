/*
 * Copyright 2019-Present tarnishablec. All Rights Reserved.
 */

//! Build script for dwebble-rws
//!
//! Generates a C++ header using cbindgen.
//! Cargo-make handles DLL copying (see Makefile.toml).

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    generate_bindings(Path::new(&crate_dir));
}

fn generate_bindings(crate_path: &Path) {
    let config_path = crate_path.join("cbindgen.toml");
    let output_path = crate_path.join("include").join("dwebble_rws.h");

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
        })
        .unwrap_or_else(|e| {
            println!("cargo:warning=cbindgen failed: {}", e);
        });
}
