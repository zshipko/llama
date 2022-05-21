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

        unsafe { llvm::core::LLVMContextDispose(self.llvm()) }
    }
}

static INIT: std::sync::Once = std::sync::Once::new();

impl<'a> Clone for Context<'a> {
    fn clone(&self) -> Context<'a> {
        Context(self.0, false, PhantomData)
    }
}

impl<'a> Context<'a> {
    fn init() {
        INIT.call_once(|| unsafe {
            llvm::target::LLVM_InitializeAllTargetInfos();
            llvm::target::LLVM_InitializeAllTargetMCs();
            llvm::target::LLVM_InitializeAllDisassemblers();
            llvm::target::LLVM_InitializeAllTargets();
            llvm::target::LLVM_InitializeAllAsmParsers();
            llvm::target::LLVM_InitializeAllAsmPrinters();
        });
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

    /// Allow context to discard value names, this can be used to save on runtime allocations
    pub fn set_discard_value_names(&mut self, discard: bool) {
        unsafe {
            llvm::core::LLVMContextSetDiscardValueNames(self.llvm(), if discard { 1 } else { 0 })
        }
    }

    /// Returns true when the context is set to discard value names
    pub fn discard_value_names(&mut self) -> bool {
        unsafe { llvm::core::LLVMContextShouldDiscardValueNames(self.llvm()) == 1 }
    }

    /// Insert a new basic block
    pub fn insert_basic_block(
        &self,
        bb: BasicBlock<'a>,
        name: impl AsRef<str>,
    ) -> Result<BasicBlock<'a>, Error> {
        let name = cstr!(name.as_ref());
        let bb = unsafe {
            llvm::core::LLVMInsertBasicBlockInContext(self.llvm(), bb.llvm(), name.as_ptr())
        };
        BasicBlock::from_inner(bb)
    }

    /// Get metadata kind ID
    pub fn md_kind_id(&self, name: impl AsRef<str>) -> u32 {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMGetMDKindIDInContext(self.llvm(), name.as_ptr(), len as u32) }
    }

    /// Get enum attribute kind ID
    pub fn enum_attribute_kind_for_name(&self, name: impl AsRef<str>) -> u32 {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMGetEnumAttributeKindForName(name.as_ptr(), len) }
    }

    /// Get type by name
    pub fn type_by_name(&self, name: impl AsRef<str>) -> Result<Type<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe { Type::from_inner(llvm::core::LLVMGetTypeByName2(self.llvm(), name.as_ptr())) }
    }

    // TODO: LLVMContextGetDiagnosticHandler, LLVMContextSetDiagnosticHandler,
    // LLVMContextSetYieldCallback, ...
}
