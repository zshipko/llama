fn main() {
    let output = std::process::Command::new(
        std::env::var("LLVM_CONFIG").unwrap_or("llvm-config".to_string()),
    )
    .arg("--prefix")
    .output()
    .unwrap()
    .stdout;

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let prefix = std::path::PathBuf::from(String::from_utf8(output).unwrap().trim());

    let lto_full = std::fs::read_link(prefix.join("lib").join("libLTO.so")).unwrap();

    std::fs::copy(prefix.join("lib").join(&lto_full), &out_dir.join(&lto_full)).unwrap();

    std::fs::copy(
        prefix.join("lib").join("libLLVM.so"),
        &out_dir.join("libLLVM.so"),
    )
    .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=LLVM");
    println!("cargo:rustc-link-lib=LTO");
}
