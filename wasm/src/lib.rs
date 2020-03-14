#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Target is not WASM")]
    InvalidTarget,

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Llama: {0}")]
    Llama(#[from] llama::Error),

    #[error("Error: {0}")]
    Error(#[from] anyhow::Error),
}

#[cfg(feature = "wasmtime")]
mod wasm {
    use crate::*;

    pub use wasmtime::{Extern, Func, Store};

    pub struct Wasm<'a> {
        pub store: &'a wasmtime::Store,
        pub module: wasmtime::Module,
        pub instance: wasmtime::Instance,
    }

    impl<'a> Wasm<'a> {
        pub fn new(
            store: &'a Store,
            module: &llama::Module,
            bindings: impl AsRef<[Extern]>,
        ) -> Result<Wasm<'a>, Error> {
            if !module.target()?.to_ascii_lowercase().starts_with("wasm") {
                return Err(Error::InvalidTarget);
            }

            let mut codegen = llama::Codegen::new()?;
            codegen.add_module(&module).unwrap();
            let bin = codegen.compile_optimized()?;

            let mut f = std::fs::File::create("test.wasm").unwrap();
            std::io::Write::write_all(&mut f, bin).unwrap();

            let module = wasmtime::Module::from_binary(&store, bin)?;
            let instance = wasmtime::Instance::new(&module, bindings.as_ref())?;

            Ok(Wasm {
                store,
                module,
                instance,
            })
        }

        pub fn func(&self, name: impl AsRef<str>) -> Result<&Func, Error> {
            let f = match self.instance.get_export(name.as_ref()) {
                Some(x) => x,
                None => return Err(Error::FunctionNotFound(name.as_ref().into())),
            };

            let f = match f.func() {
                Some(x) => x,
                None => return Err(Error::FunctionNotFound(name.as_ref().into())),
            };

            Ok(f)
        }
    }
}

pub use wasm::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn codegen() {
        let context = llama::Context::new().unwrap();
        let mut module = llama::Module::new(&context, "test").unwrap();
        module.set_target("wasm32-unknown-unknown-wasm");

        let builder = llama::Builder::new(&context).unwrap();

        let i32 = llama::Type::int(&context, 32).unwrap();

        let ft = llama::FunctionType::new(&i32, &[&i32, &i32], false).unwrap();
        let f = module.add_function("testing", &ft).unwrap();
        builder
            .define_function(&f, |builder, _| {
                let params = f.params();
                let a = builder.add(&params[0], &params[1], "a")?;
                builder.ret(&a)
            })
            .unwrap();

        println!("{}", module);

        let store = Store::default();
        let wasm = Wasm::new(&store, &module, &[]).unwrap();
        let testing = wasm.func("testing").unwrap();
        let f = testing.get2::<i32, i32, i32>().unwrap();
        assert_eq!(f(1, 2).unwrap(), 3);
    }
}
