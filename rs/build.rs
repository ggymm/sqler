use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Re-run build script if FFI or config changes
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    // Try to run cbindgen if available to generate include/sqler.h
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let include_dir = out_dir.join("include");
    let header_path = include_dir.join("sqler.h");

    let _ = std::fs::create_dir_all(&include_dir);

    let status = Command::new("cbindgen")
        .current_dir(&out_dir)
        .arg("--config").arg("cbindgen.toml")
        .arg("--crate").arg("sqler")
        .arg("--output").arg(&header_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Generated C header at {}", header_path.display());
        }
        Ok(s) => {
            println!("cargo:warning=cbindgen failed with status {:?}; header not regenerated", s.code());
        }
        Err(e) => {
            println!("cargo:warning=cbindgen not found ({}); skipping header generation", e);
        }
    }
}

