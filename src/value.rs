use crate::*;

pub struct Value<'a>(NonNull<llvm::LLVMValue>, PhantomData<&'a ()>);

llvm_inner_impl!(Value<'a>, llvm::LLVMValue);

pub type ValueKind = llvm::LLVMValueKind;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttributeIndex {
    Return,
    Param(u32),
    Function,
}

impl AttributeIndex {
    pub(crate) fn get_index(&self) -> u32 {
        match self {
            AttributeIndex::Return => llvm::LLVMAttributeReturnIndex,
            AttributeIndex::Param(index) => {
                assert!(
                    *index <= u32::max_value() - 2,
                    "Param index must be <= u32::max_value() - 2"
                );

                index + 1
            }
            AttributeIndex::Function => llvm::LLVMAttributeFunctionIndex,
        }
    }
}

impl<'a> AsRef<Value<'a>> for Value<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self
    }
}

impl<'a> Clone for Value<'a> {
    fn clone(&self) -> Value<'a> {
        Value(self.0.clone(), PhantomData)
    }
}

impl<'a> Value<'a> {
    pub(crate) fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Value<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Value(t, PhantomData))
    }

    pub fn into_metadata(self) -> Metadata<'a> {
        Metadata(self)
    }

    pub fn is_basic_block(&self) -> bool {
        unsafe { llvm::core::LLVMValueIsBasicBlock(self.llvm_inner()) == 0 }
    }

    pub fn kind(&self) -> ValueKind {
        unsafe { llvm::core::LLVMGetValueKind(self.llvm_inner()) }
    }

    pub fn is(&self, kind: ValueKind) -> bool {
        self.kind() == kind
    }

    pub fn type_of(&self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMTypeOf(self.llvm_inner()) };
        Type::from_inner(t)
    }

    pub(crate) fn into_context(self) -> Result<Context<'a>, Error> {
        self.type_of()?.into_context()
    }

    pub fn context(&self) -> Result<Context, Error> {
        self.type_of()?.into_context()
    }

    pub fn name(&self) -> Result<&str, Error> {
        let mut size = 0;
        unsafe {
            let s = llvm::core::LLVMGetValueName2(self.llvm_inner(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetValueName2(self.llvm_inner(), name.as_ptr(), len) }
    }

    pub fn replace_all_uses_with(&self, other: impl AsRef<Value<'a>>) {
        unsafe {
            llvm::core::LLVMReplaceAllUsesWith(self.llvm_inner(), other.as_ref().llvm_inner())
        }
    }

    pub fn is_const(&self) -> bool {
        unsafe { llvm::core::LLVMIsConstant(self.llvm_inner()) == 1 }
    }

    pub fn into_const(self) -> Result<Const<'a>, Error> {
        if !self.is_const() {
            return Err(Error::InvalidConst);
        }

        Ok(Const(self))
    }

    pub fn to_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        if !self.is_basic_block() {
            return Err(Error::InvalidBasicBlock);
        }

        let ptr = unsafe { llvm::core::LLVMValueAsBasicBlock(self.llvm_inner()) };
        BasicBlock::from_inner(ptr)
    }

    pub fn is_undef(&self) -> bool {
        unsafe { llvm::core::LLVMIsUndef(self.llvm_inner()) == 1 }
    }

    pub fn is_null(&self) -> bool {
        unsafe { llvm::core::LLVMIsNull(self.llvm_inner()) == 1 }
    }

    pub fn is_constant_string(&self) -> bool {
        unsafe { llvm::core::LLVMIsConstantString(self.llvm_inner()) == 1 }
    }

    pub fn count_basic_blocks(&self) -> usize {
        unsafe { llvm::core::LLVMCountBasicBlocks(self.llvm_inner()) as usize }
    }

    pub fn basic_blocks(&self) -> Vec<BasicBlock<'a>> {
        let count = self.count_basic_blocks();
        let ptr = std::ptr::null_mut();
        unsafe { llvm::core::LLVMGetBasicBlocks(self.llvm_inner(), ptr) }
        let slice = unsafe { std::slice::from_raw_parts(ptr, count) };
        slice
            .iter()
            .map(|x| BasicBlock::from_inner(*x).unwrap())
            .collect()
    }

    pub fn first_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetFirstBasicBlock(self.llvm_inner()) })
    }

    pub fn last_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetLastBasicBlock(self.llvm_inner()) })
    }

    pub fn entry_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetEntryBasicBlock(self.llvm_inner()) })
    }

    pub fn append_basic_block(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAppendExistingBasicBlock(self.llvm_inner(), bb.llvm_inner()) }
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message =
            unsafe { Message::from_raw(llvm::core::LLVMPrintValueToString(self.llvm_inner())) };
        write!(fmt, "{}", message.as_ref())
    }
}

pub struct Function<'a>(pub(crate) Value<'a>);

impl<'a> AsRef<Value<'a>> for Function<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Function<'a>> for Value<'a> {
    fn from(x: Function<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Function<'a> {
    pub fn param_count(&self) -> usize {
        let n = unsafe { llvm::core::LLVMCountParams(self.as_ref().llvm_inner()) };
        n as usize
    }

    pub fn param(&self, i: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetParam(
                self.as_ref().llvm_inner(),
                i as u32,
            ))
        }
    }

    pub fn params(&self) -> Vec<Value<'a>> {
        let len = self.param_count();
        let mut data = vec![std::ptr::null_mut(); len];

        unsafe { llvm::core::LLVMGetParams(self.as_ref().llvm_inner(), data.as_mut_ptr()) }
        data.into_iter()
            .map(|x| Value::from_inner(x).unwrap())
            .collect()
    }

    /// Verify the function, returning an error on failure
    pub fn verify(&self) -> Result<(), Error> {
        let ok = unsafe {
            llvm::analysis::LLVMVerifyFunction(
                self.as_ref().llvm_inner(),
                llvm::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            ) == 0
        };

        if !ok {
            return Err(Error::InvalidFunction);
        }

        Ok(())
    }

    pub fn next_function(&self) -> Result<Function<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetNextFunction(self.as_ref().llvm_inner()) };
        Value::from_inner(v).map(Function)
    }

    pub fn prev_function(&self) -> Result<Function<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetPreviousFunction(self.as_ref().llvm_inner()) };
        Value::from_inner(v).map(Function)
    }

    pub fn delete(self) {
        unsafe { llvm::core::LLVMDeleteFunction(self.as_ref().llvm_inner()) }
    }

    pub fn has_personality_fn(&self) -> bool {
        unsafe { llvm::core::LLVMHasPersonalityFn(self.as_ref().llvm_inner()) == 1 }
    }

    pub fn personality_fn(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetPersonalityFn(self.as_ref().llvm_inner())) }
    }

    pub fn set_personality_fn(&mut self, f: impl AsRef<Value<'a>>) {
        unsafe {
            llvm::core::LLVMSetPersonalityFn(self.as_ref().llvm_inner(), f.as_ref().llvm_inner())
        }
    }

    pub fn gc(&self) -> Option<&str> {
        let gc = unsafe { llvm::core::LLVMGetGC(self.as_ref().llvm_inner()) };
        if gc.is_null() {
            return None;
        }

        unsafe {
            let slice = std::slice::from_raw_parts(gc as *const u8, strlen(gc));
            Some(std::str::from_utf8_unchecked(slice))
        }
    }

    pub fn set_gc(&mut self, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetGC(self.as_ref().llvm_inner(), name.as_ptr()) }
    }

    pub fn call_conv(&self) -> CallConv {
        unsafe {
            std::mem::transmute(llvm::core::LLVMGetFunctionCallConv(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn set_call_conv(&mut self, conv: CallConv) {
        unsafe { llvm::core::LLVMSetFunctionCallConv(self.as_ref().llvm_inner(), conv as u32) }
    }

    pub fn add_attribute(&mut self, index: AttributeIndex, attr: &Attribute<'a>) {
        unsafe {
            llvm::core::LLVMAddAttributeAtIndex(
                self.as_ref().llvm_inner(),
                index.get_index(),
                attr.llvm_inner(),
            )
        }
    }

    pub fn attributes(&self, index: usize) -> Vec<Attribute<'a>> {
        let count = unsafe {
            llvm::core::LLVMGetAttributeCountAtIndex(self.as_ref().llvm_inner(), index as c_uint)
        };

        let mut output = vec![std::ptr::null_mut(); count as usize];

        unsafe {
            llvm::core::LLVMGetAttributesAtIndex(
                self.as_ref().llvm_inner(),
                index as c_uint,
                output.as_mut_ptr(),
            );
        }

        output
            .into_iter()
            .map(|x| Attribute::from_inner(x).unwrap())
            .collect()
    }
}
