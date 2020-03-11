use crate::*;

pub struct BasicBlock<'a>(NonNull<llvm::LLVMBasicBlock>, PhantomData<&'a ()>);

llvm_inner_impl!(BasicBlock<'a>, llvm::LLVMBasicBlock);

impl<'a> Drop for BasicBlock<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDeleteBasicBlock(self.llvm_inner()) }
    }
}
