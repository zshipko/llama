# llama

A friendly LLVM library for Rust.

Goals:
- Support the latest `llvm-sys` release (as of LLVM 14 and llama 0.14.0 the version numbers match)
- Provide an improved interface, while still remaining as close as possible to the LLVM C API.

Due to the size of the LLVM API there is bound to be missing, broken or incomplete functionality in `llama`, please create an issue if something you need isn't implemented.

**NOTE**: `llama` will let you generate invalid IR, take a look at [inkwell](https://github.com/TheDan64/inkwell) for LLVM bindings with a focus on type-safety

## Documentation

- [llama](https://zshipko.github.io/llama/llama) <a href="https://crates.io/crates/llama"><img src="https://img.shields.io/crates/v/llama.svg"></a>
- [llama-build](https://zshipko.github.io/llama/llama_build) <a href="https://crates.io/crates/llama-build"><img src="https://img.shields.io/crates/v/llama-build.svg"></a>

## Examples

Inkwell's example using `llama`:

```rust
use llama::*;

// Convenience type alias for the `sum` function.
//
// Calling this is innately `unsafe` because there's no guarantee it doesn't
// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;


// Context should be last to perform cleanup in the correct order
struct CodeGen<'ctx> {
    engine: ExecutionEngine<'ctx>,
    build: Builder<'ctx>,
    context: Context<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_sum(&mut self) -> Result<SumFunc, Error> {
        let i64 = Type::i64(&self.context)?;
        let sum_t = FuncType::new(i64, [i64, i64, i64])?;
        self.engine
            .module()
            .declare_function(&self.build, "sum", sum_t, |f| {
                let params = f.params();
                let x = params[0];
                let y = params[1];
                let z = params[2];

                let sum = self.build.add(x, y, "sum")?;
                let sum = self.build.add(sum, z, "sum")?;
                self.build.ret(sum)
            })?;

        unsafe { self.engine.function("sum") }
    }
}

fn main() -> Result<(), Error> {
    let context = Context::new()?;
    let module = Module::new(&context, "sum")?;
    let build = Builder::new(&context)?;
    let engine = ExecutionEngine::new_jit(module, 0)?;
    let mut codegen = CodeGen {
        context: context,
        build,
        engine,
    };

    let sum = codegen.jit_compile_sum()?;

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
