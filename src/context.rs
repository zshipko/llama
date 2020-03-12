use crate::*;

/// Context wraps LLVMContext
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

/*pub struct DiagnosticInfo(NonNull<llvm::LLVMDiagnosticInfo>);

llvm_inner_impl!(DiagnosticInfo, llvm::LLVMDiagnosticInfo);

impl DiagnosticInfo {
    pub fn severity(&self) -> DiagnosticSeverity {
        unsafe { llvm::core::LLVMGetDiagInfoSeverity(self.llvm_inner()) }
    }

    pub fn description(&self) -> Message {
        let message = unsafe { llvm::core::LLVMGetDiagInfoDescription(self.llvm_inner()) };
        Message::from_raw(message)
    }
}*/

impl Context {
    /// Create a new context
    pub fn new() -> Result<Context, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMContextCreate())? };
        Ok(Context(ctx))
    }

    /// Return the global context
    pub fn global() -> Result<Context, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetGlobalContext())? };
        Ok(Context(ctx))
    }

    pub fn set_discard_value_names(&mut self, discard: bool) {
        unsafe {
            llvm::core::LLVMContextSetDiscardValueNames(
                self.llvm_inner(),
                if discard { 1 } else { 0 },
            )
        }
    }

    pub fn discard_value_names(&mut self) -> bool {
        unsafe { llvm::core::LLVMContextShouldDiscardValueNames(self.llvm_inner()) == 1 }
    }

    // TODO: LLVMContextGetDiagnosticHandler, LLVMContextSetDiagnosticHandler,
    // LLVMContextSetYieldCallback, ...
}
