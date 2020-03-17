use std::os::unix::fs::PermissionsExt;

fn main() {
    let output = std::process::Command::new(
        std::env::var("LLVM_CONFIG").unwrap_or_else(|_| "llvm-config".to_string()),
    )
    .arg("--prefix")
    .output()
    .unwrap()
    .stdout;

    #[cfg(not(target_os = "macos"))]
    let shared_lib = "so";

    #[cfg(target_os = "macos")]
    let shared_lib = "dylib";

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let prefix = std::path::PathBuf::from(String::from_utf8(output).unwrap().trim());

    // Copy LTO lib
    let lto_file_name = format!("libLTO.{}", shared_lib);
    let lto = prefix.join("lib").join(&lto_file_name);
    let lto_full = match std::fs::read_link(&lto) {
        Ok(x) => x,
        Err(_) => lto,
    };

    let out_file = out_dir.join(lto_full.file_name().unwrap());

    match lto_full.parent() {
        None => std::fs::copy(prefix.join("lib").join(&lto_full), &out_file).unwrap(),
        Some(_) => std::fs::copy(prefix.join("lib").join(&lto_full), &out_file).unwrap(),
    };

    let meta = std::fs::metadata(&out_file).unwrap();
    let mut permissions = meta.permissions();
    permissions.set_mode(0o655);
    std::fs::set_permissions(&out_file, permissions).unwrap();

    // Copy LLVM lib
    let llvm_file_name = format!("libLLVM.{}", shared_lib);
    let out_file = out_dir.join(&llvm_file_name);

    std::fs::copy(prefix.join("lib").join(&llvm_file_name), &out_file).unwrap();

    let meta = std::fs::metadata(&out_file).unwrap();
    let mut permissions = meta.permissions();
    permissions.set_mode(0o655);
    std::fs::set_permissions(&out_file, permissions).unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=LLVM");
    println!("cargo:rustc-link-lib=LTO");
}
