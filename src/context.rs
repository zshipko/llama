use crate::*;

pub struct Context(NonNull<llvm::LLVMContext>);

llvm_inner_impl!(Context, llvm::LLVMContext);

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            let global = llvm::core::LLVMGetGlobalContext();
            if self.llvm_inner() != global {
                llvm::core::LLVMContextDispose(self.llvm_inner())
            }
        }
    }
}

impl Context {
    pub fn new() -> Result<Context, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMContextCreate())? };
        Ok(Context(ctx))
    }

    pub fn global() -> Result<Context, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetGlobalContext())? };
        Ok(Context(ctx))
    }

    pub fn module<'a>(&'a self, name: impl AsRef<str>) -> Result<Module<'a>, Error> {
        Module::new(self, name)
    }
}
