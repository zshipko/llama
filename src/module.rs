use crate::*;

pub struct Module<'a>(NonNull<llvm::LLVMModule>, PhantomData<&'a ()>);

impl<'a> LLVMInner<llvm::LLVMModule> for Module<'a> {
    fn llvm_inner(&self) -> *mut llvm::LLVMModule {
        self.0.as_ptr()
    }
}

impl<'a> Drop for Module<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeModule(self.llvm_inner()) }
    }
}

impl<'a> Module<'a> {
    pub fn new(ctx: &'a Context, name: impl AsRef<str>) -> Result<Module<'a>, Error> {
        let name = cstr!(name.as_ref());
        let m = unsafe {
            wrap_inner(llvm::core::LLVMModuleCreateWithNameInContext(
                name.as_ptr(),
                ctx.llvm_inner(),
            ))?
        };
        Ok(Module(m, PhantomData))
    }

    pub fn identifier(&self) -> Result<&str, Error> {
        let mut size = 0usize;
        unsafe {
            let s = llvm::core::LLVMGetModuleIdentifier(self.llvm_inner(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }
}
