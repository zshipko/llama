# llama

A friendly LLVM library for Rust

Goals:
- Support the latest `llvm-sys` release
- Provide a nice interface, while still remaining as close as possible to the LLVM C API.

**NOTE**: `llama` will let you generate invalid IR, take a look at [inkwell](https://github.com/TheDan64/inkwell) for LLVM bindings with a focus on type-safety

## Building

`build.rs` will look for either an executable in the path named `llvm-config` or an environment variable named `LLVM_CONFIG` for setting the correct `llvm-config` executable.

## Documentation

- [llama](https://zshipko.github.io/llama/llama)
- [llama-build](https://zshipko.github.io/llama/llam-build)
- [llama-wasm](https://zshipko.github.io/llama/llam-wasm)
