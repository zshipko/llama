#[cfg(feature = "docs-rs")]
fn main() {}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    if cfg!(not(target_os = "windows")) {
        println!("cargo:rustc-link-lib=ffi");
        println!("cargo:rustc-link-lib=LLVM");
        println!("cargo:rustc-link-lib=LTO");
    }
}
