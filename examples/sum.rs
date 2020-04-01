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