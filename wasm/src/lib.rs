use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Target is not WASM")]
    InvalidTarget,

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Llama: {0}")]
    Llama(#[from] llama::Error),

    #[error("Lightbeam")]
    Lightbeam,
}

pub use lightbeam::ExecutableModule as Exec;

pub struct Wasm<'a> {
    exec: Exec,
    func_map: BTreeMap<String, usize>,
    pub module: &'a llama::Module<'a>,
    pub codegen: llama::Codegen,
}

impl<'a> Wasm<'a> {
    pub fn new<'b>(
        module: &'a llama::Module,
        exports: impl AsRef<[&'b str]>,
    ) -> Result<Wasm<'a>, Error> {
        if !module.target()?.to_ascii_lowercase().starts_with("wasm") {
            return Err(Error::InvalidTarget);
        }

        let mut func_map = BTreeMap::new();

        let mut codegen = llama::Codegen::new()?;
        codegen.add_module(&module).unwrap();

        let mut index = 0;
        let exports = exports.as_ref();

        for sym in exports.iter() {
            codegen.preserve_symbol(sym);
        }

        let bin = codegen.compile()?;

        let symbols = codegen.symbols();
        for sym in symbols {
            println!("SYM: {}", sym);
            if exports.contains(&sym.as_str()) {
                println!("EXPORT: {}", sym);
                func_map.insert(sym.clone(), index);
                index += 1;
            }
        }

        let mut f = std::fs::File::create("test.wasm").unwrap();
        std::io::Write::write_all(&mut f, bin).unwrap();

        let exec = lightbeam::translate(bin).map_err(|_| Error::Lightbeam)?;
        Ok(Wasm {
            module,
            codegen,
            exec,
            func_map,
        })
    }

    pub fn index(&self, name: impl AsRef<str>) -> Option<u32> {
        self.func_map.get(name.as_ref()).map(|x| *x as u32)
    }

    pub fn exec(&self) -> &Exec {
        &self.exec
    }
}

#[macro_export]
macro_rules! call {
    ($wasm:ident.$name:ident($($arg:expr),*$(,)?)) => {
        $wasm.exec().execute_func($wasm.index(stringify!($name)).expect("Invalid function"), ($($arg),*))
    };
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn codegen() {
        let context = llama::Context::new().unwrap();
        let mut module = llama::Module::new(&context, "test").unwrap();
        module.set_target("wasm32-unknown-wasi");

        let builder = llama::Builder::new(&context).unwrap();

        let i32 = llama::Type::int(&context, 32).unwrap();

        let ft = llama::FunctionType::new(&i32, &[&i32, &i32], false).unwrap();
        let f = module.add_function("testing_sub", &ft).unwrap();
        builder
            .define_function(&f, |builder, _| {
                let params = f.params();
                let a = builder.sub(&params[0], &params[1], "a")?;
                builder.ret(&a)
            })
            .unwrap();

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

        let wasm = Wasm::new(&module, &["testing"]).unwrap();
        println!("{:?}", wasm.func_map);

        let x: i32 = call!(wasm.testing(1i32, 2i32)).unwrap();
        assert_eq!(x, 3)
    }
}
