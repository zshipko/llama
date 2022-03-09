#[cfg(target_os = "macos")]
use std::os::unix::fs::PermissionsExt;

#[cfg(feature = "docs-rs")]
fn main() {}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    let output = std::process::Command::new(std::env::var("LLVM_CONFIG").unwrap_or_else(|_| {
        std::env::var("DEP_LLVM_CONFIG_PATH").unwrap_or_else(|_| "llvm-config".to_string())
    }))
    .arg("--libdir")
    .output()
    .unwrap()
    .stdout;

    #[cfg(not(target_os = "macos"))]
    let shared_lib = "so";

    #[cfg(target_os = "macos")]
    let shared_lib = "dylib";

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let libdir = std::path::PathBuf::from(String::from_utf8(output).unwrap().trim());

    // Copy LTO lib
    let lto_file_name = format!("libLTO.{}", shared_lib);
    let lto = libdir.join(&lto_file_name);

    let out_file = out_dir.join(&lto_file_name);
    if !out_file.exists() {
        std::fs::copy(&lto, &out_file).unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        let meta = std::fs::metadata(&out_file).unwrap();
        let mut permissions = meta.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&out_file, permissions).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(lto_full) = std::fs::read_link(&lto) {
            let out_file = out_dir.join(lto_full.file_name().unwrap());
            if !out_file.exists() {
                std::fs::copy(libdir.join(&lto_full), out_file).unwrap();
            }
        }
    }

    // Copy LLVM lib
    let llvm_file_name = format!("libLLVM.{}", shared_lib);
    let out_file = out_dir.join(&llvm_file_name);
    println!("FILENAME: {}", llvm_file_name);
    println!("LIBDIR: {}", libdir.display());
    if !out_file.exists() {
        std::fs::copy(libdir.join(&llvm_file_name), &out_file).unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        let meta = std::fs::metadata(&out_file).unwrap();
        let mut permissions = meta.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&out_file, permissions).unwrap();
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=LLVM");
    println!("cargo:rustc-link-lib=LTO");
}
