use crate::*;

use std::collections::BTreeSet;

pub struct Codegen(
    NonNull<llvm::lto::LLVMOpaqueLTOCodeGenerator>,
    Vec<llvm::lto::lto_module_t>,
    BTreeSet<String>,
);

llvm_inner_impl!(Codegen, llvm::lto::LLVMOpaqueLTOCodeGenerator);

impl Drop for Codegen {
    fn drop(&mut self) {
        self.1
            .iter()
            .for_each(|x| unsafe { llvm::lto::lto_module_dispose(*x) });
        unsafe { llvm::lto::lto_codegen_dispose(self.0.as_ptr()) }
    }
}

impl Codegen {
    pub fn new() -> Result<Codegen, Error> {
        let lto = unsafe { wrap_inner(llvm::lto::lto_codegen_create())? };
        Ok(Codegen(lto, Vec::new(), BTreeSet::new()))
    }

    pub fn add_module(&mut self, module: &Module) -> Result<(), Error> {
        let s = module.write_bitcode_to_memory_buffer()?;
        let context = module.context()?;
        let bin = Binary::new(&context, &s)?;

        if let Ok(func) = module.first_function() {
            let mut func = func;
            self.2.insert(func.as_ref().name()?.to_string());

            while let Ok(f) = func.next_function() {
                self.2.insert(f.as_ref().name()?.to_string());
                func = f;
            }
        }

        let module = unsafe {
            llvm::lto::lto_module_create_from_memory(
                bin.as_ref().as_ptr() as *mut c_void,
                bin.as_ref().len(),
            )
        };

        if module.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        let r = unsafe { llvm::lto::lto_codegen_add_module(self.llvm_inner(), module) == 0 };
        if !r {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }
        Ok(())
    }

    pub fn compile(&self) -> Result<&[u8], Error> {
        let mut len = 0;
        let ptr = unsafe { llvm::lto::lto_codegen_compile(self.llvm_inner(), &mut len) };

        if ptr.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const u8, len)) }
    }

    pub fn compile_optimized(&self) -> Result<&[u8], Error> {
        let mut len = 0;
        let ptr = unsafe { llvm::lto::lto_codegen_compile_optimized(self.llvm_inner(), &mut len) };

        if ptr.is_null() {
            let msg = unsafe { llvm::core::LLVMCreateMessage(llvm::lto::lto_get_error_message()) };
            return Err(Error::Message(Message(msg)));
        }

        unsafe { Ok(std::slice::from_raw_parts(ptr as *const u8, len)) }
    }

    pub fn preserve_symbol(&self, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        unsafe { llvm::lto::lto_codegen_add_must_preserve_symbol(self.llvm_inner(), name.as_ptr()) }
    }

    pub fn symbols(&self) -> &BTreeSet<String> {
        &self.2
    }
}
