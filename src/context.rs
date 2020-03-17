use crate::*;

/// Context wraps LLVMContext
pub struct Context<'a>(
    pub(crate) NonNull<llvm::LLVMContext>,
    pub(crate) bool,
    pub(crate) PhantomData<&'a ()>,
);

llvm_inner_impl!(Context<'a>, llvm::LLVMContext);

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        if !self.1 {
            return;
        }

        unsafe { llvm::core::LLVMContextDispose(self.llvm_inner()) }
    }
}

impl<'a> Context<'a> {
    fn init() {
        unsafe {
            llvm::target::LLVM_InitializeNativeTarget();
            llvm::target::LLVM_InitializeNativeAsmPrinter();
            llvm::target::LLVM_InitializeNativeAsmParser();

            llvm::target::LLVMInitializeWebAssemblyTarget();
            llvm::target::LLVMInitializeWebAssemblyAsmPrinter();
            llvm::target::LLVMInitializeWebAssemblyAsmParser();

            llvm::target::LLVMInitializeAArch64Target();
            llvm::target::LLVMInitializeAArch64AsmPrinter();
            llvm::target::LLVMInitializeAArch64AsmParser();

            llvm::target::LLVMInitializeARMTarget();
            llvm::target::LLVMInitializeARMAsmPrinter();
            llvm::target::LLVMInitializeARMAsmParser();

            llvm::target::LLVMInitializeX86Target();
            llvm::target::LLVMInitializeX86AsmPrinter();
            llvm::target::LLVMInitializeX86AsmParser();
        }
    }

    /// Create a new context
    pub fn new() -> Result<Self, Error> {
        Self::init();
        let ctx = unsafe { wrap_inner(llvm::core::LLVMContextCreate())? };
        Ok(Context(ctx, true, PhantomData))
    }

    /// Return the global context
    pub fn global() -> Result<Self, Error> {
        Self::init();
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetGlobalContext())? };
        Ok(Context(ctx, false, PhantomData))
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

    /// Insert a new basic block
    pub fn insert_basic_block(
        &self,
        bb: &BasicBlock<'a>,
        name: impl AsRef<str>,
    ) -> Result<BasicBlock<'a>, Error> {
        let name = cstr!(name.as_ref());
        let bb = unsafe {
            llvm::core::LLVMInsertBasicBlockInContext(
                self.llvm_inner(),
                bb.llvm_inner(),
                name.as_ptr(),
            )
        };
        BasicBlock::from_inner(bb)
    }

    // TODO: LLVMContextGetDiagnosticHandler, LLVMContextSetDiagnosticHandler,
    // LLVMContextSetYieldCallback, ...
}
