# llama

<a href="https://crates.io/crates/llama">
    <img src="https://img.shields.io/crates/v/llama.svg">
</a>

A friendly LLVM library for Rust.

Goals:
- Support the latest `llvm-sys` release
- Provide a nice interface, while still remaining as close as possible to the LLVM C API.

Due to the size of the LLVM API there is bound to be missing, broken or incomplete functionality in `llama`, please create an issue if something you need isn't implemented.

**NOTE**: `llama` will let you generate invalid IR, take a look at [inkwell](https://github.com/TheDan64/inkwell) for LLVM bindings with a focus on type-safety

## Documentation

- [llama](https://zshipko.github.io/llama/llama)
- [llama-build](https://zshipko.github.io/llama/llama_build)
- [llama-wasm](https://zshipko.github.io/llama/llama_wasm)

## Examples

Inkwell's example using `llama`:

```rust
use llama::*;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

struct CodeGen<'ctx> {
    context: Context<'ctx>,
    module: Module<'ctx>,
    build: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_sum(&mut self) -> Result<(), Error> {
        let i64 = Type::i64(&self.context)?;
        let sum_t = FuncType::new(i64, [i64, i64, i64])?;
        self.module
            .declare_function(&self.build, "sum", sum_t, |f| {
                let params = f.params();
                let x = params[0];
                let y = params[1];
                let z = params[2];

                let sum = self.build.add(x, y, "sum")?;
                let sum = self.build.add(sum, z, "sum")?;
                self.build.ret(sum)
            })?;
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let context = Context::new()?;
    let module = Module::new(&context, "sum")?;
    let build = Builder::new(&context)?;
    let mut codegen = CodeGen {
        context: context,
        module,
        build,
    };

    codegen.jit_compile_sum()?;

    // Since an execution engine takes ownership of a module in `llama`, this step must be done
    // after code generation
    let execution_engine = ExecutionEngine::new_jit(codegen.module, 0)?;
    let sum: SumFunc = unsafe { execution_engine.function("sum")? };

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum(x, y, z));
        assert_eq!(sum(x, y, z), x + y + z);
    }

    Ok(())
}
```
