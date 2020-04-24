use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Target is not WASM")]
    InvalidTarget,

    #[error("Func not found: {0}")]
    FuncNotFound(String),

    #[error("Llama: {0}")]
    Llama(#[from] llama::Error),

    #[error("Lightbeam")]
    Lightbeam(String),
}

/// Exectuable module type
pub use lightbeam::ExecutableModule as Exec;

/// WASM context
pub struct Wasm<'a> {
    exec: Exec,
    func_map: BTreeMap<String, usize>,
    _module: &'a llama::Module<'a>,
    _codegen: llama::Codegen,
}

impl<'a> Wasm<'a> {
    /// Create a new WASM context with the given LLVM module and export names
    pub fn new<'b>(
        module: &'a llama::Module,
        exports: impl AsRef<[&'b str]>,
    ) -> Result<Wasm<'a>, Error> {
        if !module.target()?.to_ascii_lowercase().starts_with("wasm") {
            return Err(Error::InvalidTarget);
        }

        let mut func_map = BTreeMap::new();

        let exports = exports.as_ref();

        let codegen = llama::Codegen::new(&module, exports, true)?;

        let mut index = 0;
        let symbols = codegen.symbols();
        for sym in symbols {
            if exports.contains(&sym.as_str()) {
                func_map.insert(sym.clone(), index);
            }
            index += 1;
        }

        let exec = lightbeam::translate(codegen.as_ref())
            .map_err(|x| Error::Lightbeam(format!("{:?}", x)))?;
        Ok(Wasm {
            _module: module,
            _codegen: codegen,
            exec,
            func_map,
        })
    }

    /// Get index of named export
    pub fn index(&self, name: impl AsRef<str>) -> Option<u32> {
        self.func_map.get(name.as_ref()).map(|x| *x as u32)
    }

    /// Get executable module
    pub fn exec(&self) -> &Exec {
        &self.exec
    }
}

/// Call a function within a WASM context:
/// call!(module_name.function_name(i32, i64))
#[macro_export]
macro_rules! call {
    ($wasm:ident.$name:ident($($arg:expr),*$(,)?)) => {
        $wasm.exec().execute_func($wasm.index(stringify!($name)).expect("Invalid function"), ($($arg,)*)).map_err(|x| $crate::Error::Lightbeam(format!("{:?}", x)))
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn codegen() -> Result<(), Error> {
        let context = llama::Context::new()?;
        let mut module = llama::Module::new(&context, "test")?;
        module.set_wasm32();

        let builder = llama::Builder::new(&context)?;

        let i32 = llama::Type::int(&context, 32)?;

        let ft = llama::FuncType::new(i32, &[i32, i32])?;
        module.declare_function(&builder, "testing_sub", ft, |f| {
            let params = f.params();
            let a = builder.sub(&params[0], &params[1], "a")?;
            builder.ret(&a)
        })?;

        let ft = llama::FuncType::new(i32, &[i32, i32])?;
        module.declare_function(&builder, "testing", ft, |f| {
            let params = f.params();
            let a = builder.add(&params[0], &params[1], "a")?;
            builder.ret(&a)
        })?;

        println!("{}", module);

        let wasm = Wasm::new(&module, &["testing"])?;
        println!("{:?}", wasm.func_map);

        let x: i32 = call!(wasm.testing(1i32, 2i32))?;
        assert_eq!(x, 3);
        Ok(())
    }

    #[test]
    fn test_for_loop() -> Result<(), Error> {
        use llama::*;

        let ctx = Context::new()?;
        let mut module = Module::new(&ctx, "test_for_loop")?;
        module.set_wasm32();
        let builder = Builder::new(&ctx)?;

        let i64 = Type::int(&ctx, 64)?;
        let ft = FuncType::new(i64, &[i64])?;
        module.declare_function(&builder, "testing", ft, |f| {
            let params = f.params();
            let one = Const::int_sext(i64, 1)?;
            let f = builder.for_loop(
                Const::int_sext(i64, 0)?,
                |x| builder.icmp(Icmp::LLVMIntSLE, x, &params[0], "cond"),
                |x| builder.add(x, one, "add"),
                |x| Ok(*x),
            )?;
            builder.ret(f)
        })?;

        println!("{}", module);

        let wasm = Wasm::new(&module, &["testing"])?;
        println!("{:?}", wasm.func_map);

        let x: i64 = call!(wasm.testing(9i64))?;
        assert_eq!(x, 9);
        Ok(())
    }
}
