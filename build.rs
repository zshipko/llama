fn main() {
    let output = std::process::Command::new(
        std::env::var("LLVM_CONFIG").unwrap_or("llvm-config".to_string()),
    )
    .arg("--prefix")
    .output()
    .unwrap()
    .stdout;
    let prefix = std::path::PathBuf::from(String::from_utf8(output).unwrap().trim());
    println!("cargo:rustc-link-search={}", prefix.join("lib").display());
    //println!("cargo:rustc-link-lib=LLVM");
    println!("cargo:rustc-link-lib=LTO");
}
