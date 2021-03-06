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
