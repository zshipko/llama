use crate::*;

pub struct Value<'a>(NonNull<llvm::LLVMValue>, PhantomData<&'a ()>);

llvm_inner_impl!(Value<'a>, llvm::LLVMValue);

pub type ValueKind = llvm::LLVMValueKind;

impl<'a> Value<'a> {
    pub(crate) fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Value<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Value(t, PhantomData))
    }

    pub fn is_basic_block(&self) -> bool {
        unsafe { llvm::core::LLVMValueIsBasicBlock(self.llvm_inner()) == 0 }
    }

    pub fn kind(&self) -> ValueKind {
        unsafe { llvm::core::LLVMGetValueKind(self.llvm_inner()) }
    }
}
