// client/build.rs
// Taken from https://michael-f-bryan.github.io/rust-ffi-guide/cbindgen.html
// and modified minimially to get working.

extern crate cbindgen;

use cbindgen::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir().join("demes_forward.h").display().to_string();

    let config = Config {
        language: cbindgen::Language::C,
        pragma_once: true,
        cpp_compat: true,
        tab_width: 4,
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        //.join("target")
    }
}
