use crate::*;

/// BasicBlock wraps LLVMBasicBlock
pub struct BasicBlock<'a>(NonNull<llvm::LLVMBasicBlock>, PhantomData<&'a ()>);

llvm_inner_impl!(BasicBlock<'a>, llvm::LLVMBasicBlock);

impl<'a> Clone for BasicBlock<'a> {
    fn clone(&self) -> BasicBlock<'a> {
        BasicBlock(self.0, PhantomData)
    }
}

impl<'a> BasicBlock<'a> {
    pub fn from_inner(ptr: *mut llvm::LLVMBasicBlock) -> Result<Self, Error> {
        Ok(BasicBlock(wrap_inner(ptr)?, PhantomData))
    }

    /// Create a new basic block
    pub fn new(ctx: &'a Context, name: impl AsRef<str>) -> Result<Self, Error> {
        let name = cstr!(name.as_ref());
        let bb =
            unsafe { llvm::core::LLVMCreateBasicBlockInContext(ctx.llvm_inner(), name.as_ptr()) };
        Self::from_inner(bb)
    }

    /// Get the context used to create the basic block
    pub fn context(&self) -> Result<Context, Error> {
        self.to_value()?.into_context()
    }

    /// Append a new value to the basic block
    pub fn append(
        ctx: &'a Context,
        f: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let name = cstr!(name.as_ref());
        let bb = unsafe {
            llvm::core::LLVMAppendBasicBlockInContext(
                ctx.llvm_inner(),
                f.as_ref().llvm_inner(),
                name.as_ptr(),
            )
        };
        Self::from_inner(bb)
    }

    /// Remove and destroy basic block
    pub fn delete(self) {
        unsafe { llvm::core::LLVMDeleteBasicBlock(self.llvm_inner()) }
    }

    /// Remove basic block, keeping the block alive
    pub fn remove_from_parent(&self) {
        unsafe { llvm::core::LLVMRemoveBasicBlockFromParent(self.llvm_inner()) }
    }

    /// Move basic block before another basic block
    pub fn move_before(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMMoveBasicBlockBefore(self.llvm_inner(), bb.llvm_inner()) }
    }

    /// Move basic block after another basic block
    pub fn move_after(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMMoveBasicBlockAfter(self.llvm_inner(), bb.llvm_inner()) }
    }

    /// Convert a basic block to a `Value`
    pub fn to_value(&self) -> Result<Value<'a>, Error> {
        let ptr = unsafe { llvm::core::LLVMBasicBlockAsValue(self.llvm_inner()) };
        Value::from_inner(ptr)
    }

    /// Get the basic block name
    pub fn name(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetBasicBlockName(self.llvm_inner());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    /// Get the basic block parent value
    pub fn parent(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetBasicBlockParent(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    /// Get the basic block terminator value
    pub fn terminator(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetBasicBlockTerminator(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    /// Get the first instruction in a basic block
    pub fn first_instruction(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetFirstInstruction(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    /// Get the last instruction in a basic block
    pub fn last_instruction(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetLastInstruction(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    /// Get the next basic block
    pub fn next_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetNextBasicBlock(self.llvm_inner());
            Self::from_inner(ptr)
        }
    }

    /// Get the previous basic_block
    pub fn prev_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetPreviousBasicBlock(self.llvm_inner());
            Self::from_inner(ptr)
        }
    }
}
