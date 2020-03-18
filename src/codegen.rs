use crate::*;

use std::sync::Mutex;

pub struct Codegen(Vec<u8>, Vec<String>, Binary);

impl AsRef<[u8]> for Codegen {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Codegen> for Vec<u8> {
    fn from(x: Codegen) -> Vec<u8> {
        x.0
    }
}

lazy_static::lazy_static! {
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

impl Codegen {
    pub fn new<'a>(module: &Module, symbols: impl AsRef<[&'a str]>) -> Result<Codegen, Error> {
        let _handle = MUTEX.lock()?;

        let lto = unsafe { wrap_inner(llvm::lto::lto_codegen_create_in_local_context())? };

        let s = module.write_bitcode_to_memory_buffer()?;
        let context = module.context()?;
        let bin = Binary::new(&context, s)?;

        let mut cg = Codegen(Vec::new(), Vec::new(), bin);

        for sym in symbols.as_ref() {
            cg.preserve_symbol(lto.as_ptr(), sym)
        }

        cg.add_module(lto.as_ptr(), module)?;

        cg.0 = cg.compile(lto.as_ptr())?;
        unsafe { llvm::lto::lto_codegen_dispose(lto.as_ptr()) }
        Ok(cg)
    }

    fn add_module(&mut self, lto: llvm::lto::lto_code_gen_t, module: &Module) -> Result<(), Error> {
        if let Ok(func) = module.first_function() {
            let mut func = func;
            self.1.push(func.as_ref().name()?.to_string());

            while let Ok(f) = func.next_function() {
                self.1.push(f.as_ref().name()?.to_string());
                func = f;
            }
        }

        let module = unsafe {
            llvm::lto::lto_module_create_in_codegen_context(
                self.2.as_ref().as_ptr() as *mut c_void,
                self.2.as_ref().len(),
                std::ptr::null(),
                lto,
            )
        };

        if module.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        unsafe { llvm::lto::lto_codegen_set_module(lto, module) };
        Ok(())
    }

    /*fn compile(&self, lto: llvm::lto::lto_code_gen_t) -> Result<Vec<u8>, Error> {
        let mut len = 0;
        let ptr = unsafe { llvm::lto::lto_codegen_compile(lto, &mut len) };

        if ptr.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const u8, len).into()) }
    }*/

    fn compile(&self, lto: llvm::lto::lto_code_gen_t) -> Result<Vec<u8>, Error> {
        let mut len = 0;
        let ptr = unsafe { llvm::lto::lto_codegen_compile_optimized(lto, &mut len) };

        if ptr.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const u8, len).into()) }
    }

    fn preserve_symbol(&self, lto: llvm::lto::lto_code_gen_t, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        unsafe { llvm::lto::lto_codegen_add_must_preserve_symbol(lto, name.as_ptr()) }
    }

    pub fn symbols(&self) -> &Vec<String> {
        &self.1
    }
}
