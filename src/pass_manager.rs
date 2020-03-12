use crate::*;

pub struct PassManager(NonNull<llvm::LLVMPassManager>);

llvm_inner_impl!(PassManager, llvm::LLVMPassManager);

impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposePassManager(self.llvm_inner()) }
    }
}
