use crate::*;

/// LLVMTargetData wrapper
pub struct TargetData<'a>(
    NonNull<llvm::target::LLVMOpaqueTargetData>,
    PhantomData<&'a ()>,
);

llvm_inner_impl!(TargetData<'a>, llvm::target::LLVMOpaqueTargetData);

impl<'a> Drop for TargetData<'a> {
    fn drop(&mut self) {
        unsafe { llvm::target::LLVMDisposeTargetData(self.llvm()) }
    }
}

impl<'a> TargetData<'a> {
    pub(crate) fn from_inner(
        x: *mut llvm::target::LLVMOpaqueTargetData,
    ) -> Result<TargetData<'a>, Error> {
        Ok(TargetData(wrap_inner(x)?, PhantomData))
    }

    /// Create new target data with the given triple
    pub fn new(s: impl AsRef<str>) -> Result<TargetData<'a>, Error> {
        let s = cstr!(s.as_ref());
        unsafe { TargetData::from_inner(llvm::target::LLVMCreateTargetData(s.as_ptr())) }
    }

    /// Return a string representation
    pub fn string_rep(&self) -> Message {
        let ptr = unsafe { llvm::target::LLVMCopyStringRepOfTargetData(self.llvm()) };
        Message::from_raw(ptr)
    }

    /// Get configured byte order
    pub fn byte_order(&self) -> ByteOrder {
        unsafe { llvm::target::LLVMByteOrder(self.llvm()) }
    }

    /// Get configured pointer size
    pub fn pointer_size(&self) -> usize {
        unsafe { llvm::target::LLVMPointerSize(self.llvm()) as usize }
    }

    /// Get pointer size for the given address space
    pub fn pointer_size_for_address_space(&self, addr_space: usize) -> usize {
        unsafe { llvm::target::LLVMPointerSizeForAS(self.llvm(), addr_space as c_uint) as usize }
    }

    /// Get the type used for integer pointers
    pub fn int_ptr_type(&self, ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::target::LLVMIntPtrTypeInContext(
                ctx.llvm(),
                self.llvm(),
            ))
        }
    }

    /// Get the type used for integer pointers in the given address space
    pub fn int_ptr_type_for_address_space(
        &self,
        ctx: &Context<'a>,
        addr_space: usize,
    ) -> Result<Type<'a>, Error> {
        unsafe {
            Type::from_inner(llvm::target::LLVMIntPtrTypeForASInContext(
                ctx.llvm(),
                self.llvm(),
                addr_space as c_uint,
            ))
        }
    }

    /// Get the size of a type in bits
    pub fn size_of_type_in_bits(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe { llvm::target::LLVMSizeOfTypeInBits(self.llvm(), t.as_ref().llvm()) as usize }
    }

    /// Get the storage size of a type
    pub fn store_size_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe { llvm::target::LLVMStoreSizeOfType(self.llvm(), t.as_ref().llvm()) as usize }
    }

    /// Get the ABI size of a type
    pub fn abi_size_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe { llvm::target::LLVMABISizeOfType(self.llvm(), t.as_ref().llvm()) as usize }
    }

    /// Get the ABI alignment of a type
    pub fn abi_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe { llvm::target::LLVMABIAlignmentOfType(self.llvm(), t.as_ref().llvm()) as usize }
    }

    /// Get the call frame alignment of a type
    pub fn call_frame_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMCallFrameAlignmentOfType(self.llvm(), t.as_ref().llvm()) as usize
        }
    }

    /// Get type's preferred alignment
    pub fn preferred_alignment_of_type(&self, t: impl AsRef<Type<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMPreferredAlignmentOfType(self.llvm(), t.as_ref().llvm()) as usize
        }
    }

    /// Get preferred alignment for a global value
    pub fn preferred_alignment_of_global(&self, t: impl AsRef<Value<'a>>) -> usize {
        unsafe {
            llvm::target::LLVMPreferredAlignmentOfGlobal(self.llvm(), t.as_ref().llvm()) as usize
        }
    }

    /// Get the index of the struct element at the given offset
    pub fn struct_element_at_offset(&self, t: impl AsRef<Type<'a>>, offset: usize) -> usize {
        unsafe {
            llvm::target::LLVMElementAtOffset(self.llvm(), t.as_ref().llvm(), offset as u64)
                as usize
        }
    }

    /// Get the offset of the element at the given index
    pub fn struct_offset_of_element(&self, t: impl AsRef<Type<'a>>, index: usize) -> usize {
        unsafe {
            llvm::target::LLVMOffsetOfElement(self.llvm(), t.as_ref().llvm(), index as c_uint)
                as usize
        }
    }
}

/// LLVMTarget wrapper
pub struct Target(llvm::target_machine::LLVMTargetRef);

impl<'a> Target {
    /// Get target from triple
    pub fn new(s: impl AsRef<str>) -> Result<Target, Error> {
        let s = cstr!(s.as_ref());
        unsafe {
            let ptr = llvm::target_machine::LLVMGetTargetFromName(s.as_ptr());
            if ptr.is_null() {
                return Err(Error::NullPointer);
            }
            Ok(Target(ptr))
        }
    }

    /// Get the default target
    pub fn default() -> Result<Target, Error> {
        Target::new(default_target_triple())
    }

    /// Get host CPU name
    pub fn host_cpu_name() -> Message {
        unsafe { Message::from_raw(llvm::target_machine::LLVMGetHostCPUName()) }
    }

    /// Get host CPU features
    pub fn host_cpu_features() -> Message {
        unsafe { Message::from_raw(llvm::target_machine::LLVMGetHostCPUFeatures()) }
    }

    /// Get first registered target
    pub fn first() -> Result<Target, Error> {
        unsafe {
            let ptr = llvm::target_machine::LLVMGetFirstTarget();
            if ptr.is_null() {
                return Err(Error::NullPointer);
            }
            Ok(Target(ptr))
        }
    }

    /// Get next targer
    pub fn next_target(&self) -> Result<Target, Error> {
        unsafe {
            let ptr = llvm::target_machine::LLVMGetNextTarget(self.0);
            if ptr.is_null() {
                return Err(Error::NullPointer);
            }
            Ok(Target(ptr))
        }
    }

    /// Get target name
    pub fn name(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::target_machine::LLVMGetTargetName(self.0);
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    /// Returns true when the target has JIT capabilities
    pub fn has_jit(&self) -> bool {
        unsafe { llvm::target_machine::LLVMTargetHasJIT(self.0) == 1 }
    }

    /// Returns true when the target has an ASM backend
    pub fn has_asm_backend(&self) -> bool {
        unsafe { llvm::target_machine::LLVMTargetHasAsmBackend(self.0) == 1 }
    }
}

/// Information about the target machine
pub struct TargetMachine<'a>(
    NonNull<llvm::target_machine::LLVMOpaqueTargetMachine>,
    PhantomData<&'a ()>,
);

llvm_inner_impl!(
    TargetMachine<'a>,
    llvm::target_machine::LLVMOpaqueTargetMachine
);

impl<'a> Drop for TargetMachine<'a> {
    fn drop(&mut self) {
        unsafe { llvm::target_machine::LLVMDisposeTargetMachine(self.llvm()) }
    }
}

impl<'a> TargetMachine<'a> {
    /// Create a new `TargetMachine`
    pub fn new(
        target: &Target,
        triple: impl AsRef<str>,
        cpu: impl AsRef<str>,
        features: impl AsRef<str>,
        opt_level: CodeGenOptLevel,
        reloc: RelocMode,
        code_model: CodeModel,
    ) -> Result<TargetMachine<'a>, Error> {
        let triple = cstr!(triple.as_ref());
        let cpu = cstr!(cpu.as_ref());
        let features = cstr!(features.as_ref());
        unsafe {
            Ok(TargetMachine(
                wrap_inner(llvm::target_machine::LLVMCreateTargetMachine(
                    target.0,
                    triple.as_ptr(),
                    cpu.as_ptr(),
                    features.as_ptr(),
                    opt_level,
                    reloc,
                    code_model,
                ))?,
                PhantomData,
            ))
        }
    }

    /// Get the target machine triple
    pub fn triple(&self) -> Message {
        unsafe {
            Message::from_raw(llvm::target_machine::LLVMGetTargetMachineTriple(
                self.llvm(),
            ))
        }
    }

    /// Get CPU name
    pub fn cpu(&self) -> Message {
        unsafe { Message::from_raw(llvm::target_machine::LLVMGetTargetMachineCPU(self.llvm())) }
    }

    /// Get feature string
    pub fn features(&self) -> Message {
        unsafe {
            Message::from_raw(llvm::target_machine::LLVMGetTargetMachineFeatureString(
                self.llvm(),
            ))
        }
    }

    /// Get data layout
    pub fn data_layout(&self) -> Result<TargetData<'a>, Error> {
        unsafe {
            TargetData::from_inner(llvm::target_machine::LLVMCreateTargetDataLayout(
                self.llvm(),
            ))
        }
    }
}
