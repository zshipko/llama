use crate::*;

/// LLVM Value wrapper
#[derive(Copy)]
pub struct Value<'a>(NonNull<llvm::LLVMValue>, PhantomData<&'a ()>);

llvm_inner_impl!(Value<'a>, llvm::LLVMValue);

/// An enumeration of possible Value kinds
pub type ValueKind = llvm::LLVMValueKind;

/// Attribute placement index
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttributeIndex {
    /// Return attribute
    Return,

    /// Param attribute
    Param(u32),

    /// Func attribute
    Func,
}

impl AttributeIndex {
    pub(crate) fn get_index(self) -> u32 {
        match self {
            AttributeIndex::Return => llvm::LLVMAttributeReturnIndex,
            AttributeIndex::Param(index) => {
                assert!(
                    index <= u32::max_value() - 2,
                    "Param index must be <= u32::max_value() - 2"
                );

                index + 1
            }
            AttributeIndex::Func => llvm::LLVMAttributeFunctionIndex,
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
        Value(self.0, PhantomData)
    }
}

impl<'a> Value<'a> {
    /// Wrap an LLVMValue pointer
    pub fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Value<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Value(t, PhantomData))
    }

    /// Convert from value to metadata
    pub fn to_metadata(self) -> Metadata<'a> {
        Metadata(self)
    }

    /// Returns true if the value is a `BasicBlock`
    pub fn is_basic_block(self) -> bool {
        unsafe { llvm::core::LLVMValueIsBasicBlock(self.llvm()) == 0 }
    }

    /// Get the value kind
    pub fn kind(self) -> ValueKind {
        unsafe { llvm::core::LLVMGetValueKind(self.llvm()) }
    }

    /// Returns true when the value kind matches `kind`
    pub fn is(self, kind: ValueKind) -> bool {
        self.kind() == kind
    }

    /// Get the `Type` of a value
    pub fn type_of(self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMTypeOf(self.llvm()) };
        Type::from_inner(t)
    }

    pub(crate) fn into_context(self) -> Result<Context<'a>, Error> {
        self.type_of()?.into_context()
    }

    /// Get the context associated with a value
    pub fn context(self) -> Result<Context<'a>, Error> {
        self.type_of()?.into_context()
    }

    /// Get the name of a value
    pub fn name(self) -> Result<&'a str, Error> {
        let mut size = 0;
        unsafe {
            let s = llvm::core::LLVMGetValueName2(self.llvm(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    /// Set the name of a value
    pub fn set_name(&mut self, name: impl AsRef<str>) {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetValueName2(self.llvm(), name.as_ptr(), len) }
    }

    /// Replace all uses of a value with another value
    pub fn replace_all_uses_with(self, other: impl AsRef<Value<'a>>) {
        unsafe { llvm::core::LLVMReplaceAllUsesWith(self.llvm(), other.as_ref().llvm()) }
    }

    /// Delete a global value
    pub fn delete_global(self) {
        unsafe { llvm::core::LLVMDeleteGlobal(self.llvm()) }
    }

    /// Get value initializer
    pub fn initializer(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetInitializer(self.llvm())) }
    }

    /// Set value initializer
    pub fn set_initializer(&mut self, val: Const<'a>) {
        unsafe { llvm::core::LLVMSetInitializer(self.llvm(), val.as_ref().llvm()) }
    }

    /// Returns true if the value is a global constant
    pub fn is_global_constant(self) -> bool {
        unsafe { llvm::core::LLVMIsGlobalConstant(self.llvm()) == 1 }
    }

    /// Toggle whether or not the value is a global constant
    pub fn set_global_constant(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetGlobalConstant(self.llvm(), b as c_int) }
    }

    /// Returns true if the value is externally defined
    pub fn is_extern(self) -> bool {
        unsafe { llvm::core::LLVMIsExternallyInitialized(self.llvm()) == 1 }
    }

    /// Toggle whether or not the value is externally declared
    pub fn set_extern(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetExternallyInitialized(self.llvm(), if b { 1 } else { 0 }) }
    }

    /// Returns true if the value is thread local
    pub fn is_thread_local(self) -> bool {
        unsafe { llvm::core::LLVMIsThreadLocal(self.llvm()) == 1 }
    }

    /// Set whether or not a value is thread local
    pub fn set_thread_local(&mut self, b: bool) {
        unsafe { llvm::core::LLVMSetThreadLocal(self.llvm(), if b { 1 } else { 0 }) }
    }

    /// Returns true when the value is a `Const`
    pub fn is_const(self) -> bool {
        unsafe { llvm::core::LLVMIsConstant(self.llvm()) == 1 }
    }

    /// Convert from value to `Const`
    pub fn to_const(self) -> Result<Const<'a>, Error> {
        if !self.is_const() {
            return Err(Error::InvalidConst);
        }

        Ok(Const(self))
    }

    /// Convert from value to `BasicBlock`
    pub fn to_basic_block(self) -> Result<BasicBlock<'a>, Error> {
        if !self.is_basic_block() {
            return Err(Error::InvalidBasicBlock);
        }

        let ptr = unsafe { llvm::core::LLVMValueAsBasicBlock(self.llvm()) };
        BasicBlock::from_inner(ptr)
    }

    /// Returns true if the value is undefined
    pub fn is_undef(self) -> bool {
        unsafe { llvm::core::LLVMIsUndef(self.llvm()) == 1 }
    }

    /// Returns true if the value is null
    pub fn is_null(self) -> bool {
        unsafe { llvm::core::LLVMIsNull(self.llvm()) == 1 }
    }

    /// Returns true if the value is a constant string
    pub fn is_constant_string(self) -> bool {
        unsafe { llvm::core::LLVMIsConstantString(self.llvm()) == 1 }
    }

    /// Count the number of basic blocks
    pub fn count_basic_blocks(self) -> usize {
        unsafe { llvm::core::LLVMCountBasicBlocks(self.llvm()) as usize }
    }

    /// Get a list of all basic blocks
    pub fn basic_blocks(self) -> Vec<BasicBlock<'a>> {
        let count = self.count_basic_blocks();
        let ptr = std::ptr::null_mut();
        unsafe { llvm::core::LLVMGetBasicBlocks(self.llvm(), ptr) }
        let slice = unsafe { std::slice::from_raw_parts(ptr, count) };
        slice
            .iter()
            .map(|x| BasicBlock::from_inner(*x).unwrap())
            .collect()
    }

    /// Get first basic block
    pub fn first_basic_block(self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetFirstBasicBlock(self.llvm()) })
    }

    /// Get last basic block
    pub fn last_basic_block(self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetLastBasicBlock(self.llvm()) })
    }

    /// Get entry block
    pub fn entry_basic_block(self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetEntryBasicBlock(self.llvm()) })
    }

    /// Append a new block
    pub fn append_basic_block(self, bb: BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAppendExistingBasicBlock(self.llvm(), bb.llvm()) }
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message = unsafe { Message::from_raw(llvm::core::LLVMPrintValueToString(self.llvm())) };
        write!(fmt, "{}", message.as_ref())
    }
}

/// Functions
#[derive(Clone, Copy)]
pub struct Func<'a>(pub(crate) Value<'a>);

impl<'a> AsRef<Value<'a>> for Func<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Func<'a>> for Value<'a> {
    fn from(x: Func<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Func<'a> {
    /// Get the number of params
    pub fn param_count(self) -> usize {
        let n = unsafe { llvm::core::LLVMCountParams(self.as_ref().llvm()) };
        n as usize
    }

    /// Get the `FuncType`
    pub fn func_type(self) -> Result<FuncType<'a>, Error> {
        self.as_ref().type_of().map(|x| x.to_func_type().unwrap())
    }

    /// Get a single param by index
    pub fn param(self, i: usize) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetParam(self.as_ref().llvm(), i as u32)) }
    }

    /// Get all params
    pub fn params(self) -> Vec<Value<'a>> {
        let len = self.param_count();
        let mut data = vec![std::ptr::null_mut(); len];

        unsafe { llvm::core::LLVMGetParams(self.as_ref().llvm(), data.as_mut_ptr()) }
        data.into_iter()
            .map(|x| Value::from_inner(x).unwrap())
            .collect()
    }

    /// Verify the function, returning an error on failure
    pub fn verify(self) -> Result<(), Error> {
        let ok = unsafe {
            llvm::analysis::LLVMVerifyFunction(
                self.as_ref().llvm(),
                llvm::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            ) == 0
        };

        if !ok {
            return Err(Error::InvalidFunction);
        }

        Ok(())
    }

    /// Get the next function
    pub fn next_function(self) -> Result<Func<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetNextFunction(self.as_ref().llvm()) };
        Value::from_inner(v).map(Func)
    }

    /// Get the previous function
    pub fn prev_function(self) -> Result<Func<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetPreviousFunction(self.as_ref().llvm()) };
        Value::from_inner(v).map(Func)
    }

    /// Delete a function
    pub fn delete(self) {
        unsafe { llvm::core::LLVMDeleteFunction(self.as_ref().llvm()) }
    }

    /// Returns true when the value has a `personality_fn`
    pub fn has_personality_fn(self) -> bool {
        unsafe { llvm::core::LLVMHasPersonalityFn(self.as_ref().llvm()) == 1 }
    }

    /// Returns the `personality_fn`
    pub fn personality_fn(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMGetPersonalityFn(self.as_ref().llvm())) }
    }

    /// Set the `personality_fn`
    pub fn set_personality_fn(&mut self, f: impl AsRef<Value<'a>>) {
        unsafe { llvm::core::LLVMSetPersonalityFn(self.as_ref().llvm(), f.as_ref().llvm()) }
    }

    /// Get garbage collection scheme name
    pub fn gc(self) -> Option<&'a str> {
        let gc = unsafe { llvm::core::LLVMGetGC(self.as_ref().llvm()) };
        if gc.is_null() {
            return None;
        }

        unsafe {
            let slice = std::slice::from_raw_parts(gc as *const u8, strlen(gc));
            Some(std::str::from_utf8_unchecked(slice))
        }
    }

    /// Set garbage collection scheme name
    pub fn set_gc(&mut self, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetGC(self.as_ref().llvm(), name.as_ptr()) }
    }

    /// Get calling convention
    pub fn call_conv(self) -> CallConv {
        unsafe { std::mem::transmute(llvm::core::LLVMGetFunctionCallConv(self.as_ref().llvm())) }
    }

    /// Set calling convention
    pub fn set_call_conv(&mut self, conv: CallConv) {
        unsafe { llvm::core::LLVMSetFunctionCallConv(self.as_ref().llvm(), conv as u32) }
    }

    /// Set attribute
    pub fn add_attribute(&mut self, index: AttributeIndex, attr: &Attribute<'a>) {
        unsafe {
            llvm::core::LLVMAddAttributeAtIndex(
                self.as_ref().llvm(),
                index.get_index(),
                attr.llvm(),
            )
        }
    }

    /// Remove an enum attribute
    pub fn remove_enum_atribute(&mut self, index: AttributeIndex, kind_id: u32) {
        unsafe {
            llvm::core::LLVMRemoveEnumAttributeAtIndex(
                self.as_ref().llvm(),
                index.get_index(),
                kind_id,
            )
        }
    }

    /// Remove a string attribute
    pub fn remove_string_atribute(&mut self, index: AttributeIndex, k: impl AsRef<str>) {
        let len = k.as_ref().len();
        let k = cstr!(k.as_ref());
        unsafe {
            llvm::core::LLVMRemoveStringAttributeAtIndex(
                self.as_ref().llvm(),
                index.get_index(),
                k.as_ptr(),
                len as u32,
            )
        }
    }

    /// Get all attributes
    pub fn attributes(self, index: usize) -> Vec<Attribute<'a>> {
        let count = unsafe {
            llvm::core::LLVMGetAttributeCountAtIndex(self.as_ref().llvm(), index as c_uint)
        };

        let mut output = vec![std::ptr::null_mut(); count as usize];

        unsafe {
            llvm::core::LLVMGetAttributesAtIndex(
                self.as_ref().llvm(),
                index as c_uint,
                output.as_mut_ptr(),
            );
        }

        output
            .into_iter()
            .map(|x| Attribute::from_inner(x).unwrap())
            .collect()
    }

    /// Create specified uniqued inline asm string (intel syntax)
    pub fn inline_asm_intel(
        t: impl AsRef<Type<'a>>,
        s: impl AsRef<str>,
        constraints: impl AsRef<str>,
        has_side_effects: bool,
        is_align_stack: bool,
    ) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetInlineAsm(
                t.as_ref().llvm(),
                s.as_ref().as_ptr() as *mut c_char,
                s.as_ref().len(),
                constraints.as_ref().as_ptr() as *mut c_char,
                constraints.as_ref().len(),
                has_side_effects as c_int,
                is_align_stack as c_int,
                llvm::LLVMInlineAsmDialect::LLVMInlineAsmDialectIntel,
            ))
        }
    }

    /// Create specified uniqued inline asm string (att syntax)
    pub fn inline_asm_att(
        t: impl AsRef<Type<'a>>,
        s: impl AsRef<str>,
        constraints: impl AsRef<str>,
        has_side_effects: bool,
        is_align_stack: bool,
    ) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetInlineAsm(
                t.as_ref().llvm(),
                s.as_ref().as_ptr() as *mut c_char,
                s.as_ref().len(),
                constraints.as_ref().as_ptr() as *mut c_char,
                constraints.as_ref().len(),
                has_side_effects as c_int,
                is_align_stack as c_int,
                llvm::LLVMInlineAsmDialect::LLVMInlineAsmDialectATT,
            ))
        }
    }

    /// Get value alignment
    pub fn alignment(self) -> usize {
        unsafe { llvm::core::LLVMGetAlignment(self.as_ref().llvm()) as usize }
    }

    /// Set value alignment
    pub fn set_alignment(&mut self, align: usize) {
        unsafe { llvm::core::LLVMSetAlignment(self.as_ref().llvm(), align as u32) }
    }

    /// Get global linkage
    pub fn global_linkage(self) -> Linkage {
        unsafe { llvm::core::LLVMGetLinkage(self.as_ref().llvm()) }
    }

    /// Set global linkage
    pub fn set_global_linkage(&mut self, l: Linkage) {
        unsafe { llvm::core::LLVMSetLinkage(self.as_ref().llvm(), l) }
    }

    /// Get global visibility
    pub fn global_visibility(self) -> Visibility {
        unsafe { llvm::core::LLVMGetVisibility(self.as_ref().llvm()) }
    }

    /// Set global visibility
    pub fn set_global_visibility(&mut self, l: Visibility) {
        unsafe { llvm::core::LLVMSetVisibility(self.as_ref().llvm(), l) }
    }
}
