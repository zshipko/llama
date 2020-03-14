use crate::*;

pub struct BasicBlock<'a>(NonNull<llvm::LLVMBasicBlock>, PhantomData<&'a ()>);

llvm_inner_impl!(BasicBlock<'a>, llvm::LLVMBasicBlock);

impl<'a> BasicBlock<'a> {
    pub fn from_inner(ptr: *mut llvm::LLVMBasicBlock) -> Result<Self, Error> {
        Ok(BasicBlock(wrap_inner(ptr)?, PhantomData))
    }

    pub fn new(ctx: &'a Context, name: impl AsRef<str>) -> Result<Self, Error> {
        let name = cstr!(name.as_ref());
        let bb =
            unsafe { llvm::core::LLVMCreateBasicBlockInContext(ctx.llvm_inner(), name.as_ptr()) };
        Self::from_inner(bb)
    }

    pub fn context(&self) -> Result<Context, Error> {
        self.to_value()?.into_context()
    }

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

    pub fn insert(&self, name: impl AsRef<str>) -> Result<Self, Error> {
        let name = cstr!(name.as_ref());
        let ctx = self.context()?;
        let bb = unsafe {
            llvm::core::LLVMInsertBasicBlockInContext(
                ctx.llvm_inner(),
                self.llvm_inner(),
                name.as_ptr(),
            )
        };
        Self::from_inner(bb)
    }

    pub fn delete(self) {
        unsafe { llvm::core::LLVMDeleteBasicBlock(self.llvm_inner()) }
    }

    pub fn remove_from_parent(&self) {
        unsafe { llvm::core::LLVMRemoveBasicBlockFromParent(self.llvm_inner()) }
    }

    pub fn move_before(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMMoveBasicBlockBefore(self.llvm_inner(), bb.llvm_inner()) }
    }

    pub fn move_after(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMMoveBasicBlockAfter(self.llvm_inner(), bb.llvm_inner()) }
    }

    pub fn to_value(&self) -> Result<Value<'a>, Error> {
        let ptr = unsafe { llvm::core::LLVMBasicBlockAsValue(self.llvm_inner()) };
        Value::from_inner(ptr)
    }

    pub fn name(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetBasicBlockName(self.llvm_inner());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    pub fn parent(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetBasicBlockParent(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    pub fn terminator(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetBasicBlockTerminator(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    pub fn first_instruction(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetFirstInstruction(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    pub fn last_instruction(&self) -> Result<Value<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetLastInstruction(self.llvm_inner());
            Value::from_inner(ptr)
        }
    }

    pub fn next(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetNextBasicBlock(self.llvm_inner());
            Self::from_inner(ptr)
        }
    }

    pub fn prev(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe {
            let ptr = llvm::core::LLVMGetPreviousBasicBlock(self.llvm_inner());
            Self::from_inner(ptr)
        }
    }
}
