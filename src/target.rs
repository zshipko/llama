use crate::*;

pub struct TargetData<'a>(
    NonNull<llvm::target::LLVMOpaqueTargetData>,
    PhantomData<&'a ()>,
);

llvm_inner_impl!(TargetData<'a>, llvm::target::LLVMOpaqueTargetData);

impl<'a> Drop for TargetData<'a> {
    fn drop(&mut self) {
        unsafe { llvm::target::LLVMDisposeTargetData(self.llvm_inner()) }
    }
}

impl<'a> TargetData<'a> {
    pub(crate) fn from_inner(
        x: *mut llvm::target::LLVMOpaqueTargetData,
    ) -> Result<TargetData<'a>, Error> {
        Ok(TargetData(wrap_inner(x)?, PhantomData))
    }

    pub fn new(s: impl AsRef<str>) -> Result<TargetData<'a>, Error> {
        let s = cstr!(s.as_ref());
        unsafe { TargetData::from_inner(llvm::target::LLVMCreateTargetData(s.as_ptr())) }
    }

    pub fn string_rep(&self) -> Message {
        let ptr = unsafe { llvm::target::LLVMCopyStringRepOfTargetData(self.llvm_inner()) };
        Message::from_raw(ptr)
    }

    pub fn byte_order(&self) -> ByteOrder {
        unsafe { llvm::target::LLVMByteOrder(self.llvm_inner()) }
    }

    pub fn pointer_size(&self) -> usize {
        unsafe { llvm::target::LLVMPointerSize(self.llvm_inner()) as usize }
    }

    pub fn pointer_size_for_address_space(&self, addr_space: usize) -> usize {
        unsafe {
            llvm::target::LLVMPointerSizeForAS(self.llvm_inner(), addr_space as c_uint) as usize
        }
    }

    pub fn int_ptr_type(&self, ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::target::LLVMIntPtrTypeInContext(
                ctx.llvm_inner(),
                self.llvm_inner(),
            ))
        }
    }

    pub fn int_ptr_type_for_address_space(
        &self,
        ctx: &Context<'a>,
        addr_space: usize,
    ) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::target::LLVMIntPtrTypeForASInContext(
                ctx.llvm_inner(),
                self.llvm_inner(),
                addr_space as c_uint,
            ))
        }
    }

    pub fn size_of_type_in_bits(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMSizeOfTypeInBits(self.llvm_inner(), t.as_ref().llvm_inner()) as usize
        }
    }

    pub fn store_size_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMStoreSizeOfType(self.llvm_inner(), t.as_ref().llvm_inner()) as usize
        }
    }

    pub fn abi_size_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMABISizeOfType(self.llvm_inner(), t.as_ref().llvm_inner()) as usize
        }
    }

    pub fn abi_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMABIAlignmentOfType(self.llvm_inner(), t.as_ref().llvm_inner())
                as usize
        }
    }

    pub fn call_frame_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMCallFrameAlignmentOfType(self.llvm_inner(), t.as_ref().llvm_inner())
                as usize
        }
    }

    pub fn preferred_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMPreferredAlignmentOfType(self.llvm_inner(), t.as_ref().llvm_inner())
                as usize
        }
    }

    pub fn preferred_alignment_of_global(&self, t: impl AsRef<Value<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMPreferredAlignmentOfGlobal(self.llvm_inner(), t.as_ref().llvm_inner())
                as usize
        }
    }

    pub fn struct_element_at_offset(&self, t: impl AsRef<Type<'a>>, offset: usize) -> usize {
        unsafe {
            llvm::target::LLVMElementAtOffset(
                self.llvm_inner(),
                t.as_ref().llvm_inner(),
                offset as u64,
            ) as usize
        }
    }

    pub fn struct_offset_of_element(&self, t: impl AsRef<Type<'a>>, offset: usize) -> usize {
        unsafe {
            llvm::target::LLVMOffsetOfElement(
                self.llvm_inner(),
                t.as_ref().llvm_inner(),
                offset as c_uint,
            ) as usize
        }
    }
}
