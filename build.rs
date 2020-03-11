fn main() {
    let dst = cmake::Config::new("wavm")
        //.define("WAVM_ENABLE_STATIC_LINKING", "1")
        .build();

    //println!("cargo:rustc-link-lib=LLVM");
    //println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-search={}", dst.join("build").display());
    println!("cargo:rustc-link-lib=WAVM");
}
