use crate::*;

pub struct ExecutionEngine(NonNull<llvm::execution_engine::LLVMOpaqueExecutionEngine>);

llvm_inner_impl!(
    ExecutionEngine,
    llvm::execution_engine::LLVMOpaqueExecutionEngine
);

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        unsafe { llvm::execution_engine::LLVMDisposeExecutionEngine(self.llvm_inner()) }
    }
}
