use crate::*;

/// Wraps LLVMModule
pub struct Module<'a>(NonNull<llvm::LLVMModule>, PhantomData<&'a ()>);

llvm_inner_impl!(Module<'a>, llvm::LLVMModule);

impl<'a> Drop for Module<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeModule(self.llvm()) }
    }
}

impl<'a> Clone for Module<'a> {
    fn clone(&self) -> Module<'a> {
        let m = unsafe {
            wrap_inner(llvm::core::LLVMCloneModule(self.llvm())).expect("Invalid module")
        };
        Module(m, PhantomData)
    }
}

impl<'a> Module<'a> {
    /// Create a new module
    pub fn new(ctx: &Context<'a>, name: impl AsRef<str>) -> Result<Module<'a>, Error> {
        let name = cstr!(name.as_ref());
        let m = unsafe {
            wrap_inner(llvm::core::LLVMModuleCreateWithNameInContext(
                name.as_ptr(),
                ctx.llvm(),
            ))?
        };
        Ok(Module(m, PhantomData))
    }

    /// Get the associated context
    pub fn context(&self) -> Result<Context<'a>, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetModuleContext(self.llvm()))? };
        Ok(Context(ctx, false, PhantomData))
    }

    /// Get the module identifier
    pub fn identifier(&self) -> Result<&str, Error> {
        let mut size = 0usize;
        unsafe {
            let s = llvm::core::LLVMGetModuleIdentifier(self.llvm(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    /// Set the module source file name
    pub fn set_source_file(&mut self, name: impl AsRef<str>) {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetSourceFileName(self.llvm(), name.as_ptr(), len) }
    }

    /// Get the source file name
    pub fn source_file(&self) -> Result<&str, Error> {
        let mut size = 0;
        unsafe {
            let s = llvm::core::LLVMGetSourceFileName(self.llvm(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    /// Set the module target string
    pub fn set_target(&mut self, target: impl AsRef<str>) {
        let target = cstr!(target.as_ref());
        unsafe { llvm::core::LLVMSetTarget(self.llvm(), target.as_ptr()) }
    }

    /// Get the target string
    pub fn target(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetTarget(self.llvm());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    /// Set the module data layout string
    pub fn set_data_layout(&mut self, layout: impl AsRef<str>) {
        let layout = cstr!(layout.as_ref());
        unsafe { llvm::core::LLVMSetDataLayout(self.llvm(), layout.as_ptr()) }
    }

    /// Get data layout string
    pub fn data_layout(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetDataLayoutStr(self.llvm());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    /// Set module inline assembly
    pub fn set_inline_asm(&mut self, asm: impl AsRef<str>) {
        let len = asm.as_ref().len();
        let asm = cstr!(asm.as_ref());
        unsafe { llvm::core::LLVMSetModuleInlineAsm2(self.llvm(), asm.as_ptr(), len) }
    }

    /// Append module inline assembly
    pub fn append_inline_asm(&mut self, asm: impl AsRef<str>) {
        let len = asm.as_ref().len();
        let asm = cstr!(asm.as_ref());
        unsafe { llvm::core::LLVMAppendModuleInlineAsm(self.llvm(), asm.as_ptr(), len) }
    }

    /// Get module inline assembly
    pub fn inline_asm(&self) -> Result<&str, Error> {
        let mut len = 0;
        unsafe {
            let s = llvm::core::LLVMGetModuleInlineAsm(self.llvm(), &mut len);
            let s = std::slice::from_raw_parts(s as *const u8, len);
            let s = std::str::from_utf8_unchecked(s);
            Ok(s)
        }
    }

    /// Verify the module, returning an error on failure
    pub fn verify(&self) -> Result<(), Error> {
        let mut message = std::ptr::null_mut();
        let ok = unsafe {
            llvm::analysis::LLVMVerifyModule(
                self.llvm(),
                llvm::analysis::LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);

        if !ok {
            return Err(Error::Message(message));
        }

        Ok(())
    }

    /// Define a new function without declaring a function body
    pub fn define_function(
        &mut self,
        name: impl AsRef<str>,
        t: FuncType,
    ) -> Result<Func<'a>, Error> {
        let name = cstr!(name.as_ref());
        let value =
            unsafe { llvm::core::LLVMAddFunction(self.llvm(), name.as_ptr(), t.as_ref().llvm()) };
        Ok(Func(Value::from_inner(value)?))
    }

    /// Declare a new function with function body
    pub fn declare_function<T: Into<Value<'a>>, F: FnOnce(&Func<'a>) -> Result<T, Error>>(
        &mut self,
        builder: &Builder<'a>,
        name: impl AsRef<str>,
        ft: FuncType,
        def: F,
    ) -> Result<Instr<'a>, Error> {
        let f = self.define_function(name, ft)?;
        builder.function_body(f, |_, _| def(&f))
    }

    /// Create a new global
    pub fn global(
        &mut self,
        name: impl AsRef<str>,
        t: impl AsRef<Type<'a>>,
    ) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        let value =
            unsafe { llvm::core::LLVMAddGlobal(self.llvm(), t.as_ref().llvm(), name.as_ptr()) };
        Ok(Value::from_inner(value)?)
    }

    /// Create a new global in the given address space
    pub fn global_in_address_space(
        &mut self,
        name: impl AsRef<str>,
        t: impl AsRef<Type<'a>>,
        addr: usize,
    ) -> Result<Func<'a>, Error> {
        let name = cstr!(name.as_ref());
        let value = unsafe {
            llvm::core::LLVMAddGlobalInAddressSpace(
                self.llvm(),
                t.as_ref().llvm(),
                name.as_ptr(),
                addr as c_uint,
            )
        };
        Ok(Func(Value::from_inner(value)?))
    }

    /// Get a function by name
    pub fn named_function(&self, name: impl AsRef<str>) -> Result<Func<'a>, Error> {
        let name = cstr!(name.as_ref());
        let value = unsafe { llvm::core::LLVMGetNamedFunction(self.llvm(), name.as_ptr()) };
        Ok(Func(Value::from_inner(value)?))
    }

    /// Get the first global
    pub fn first_global(&self) -> Result<Value<'a>, Error> {
        let value = unsafe { llvm::core::LLVMGetFirstGlobal(self.llvm()) };
        Value::from_inner(value)
    }

    /// Get the last global
    pub fn last_global(&self) -> Result<Value<'a>, Error> {
        let value = unsafe { llvm::core::LLVMGetLastGlobal(self.llvm()) };
        Value::from_inner(value)
    }

    /// Get the next global
    pub fn next_global(&self, global: impl AsRef<Value<'a>>) -> Result<Value<'a>, Error> {
        let value = unsafe { llvm::core::LLVMGetNextGlobal(global.as_ref().llvm()) };
        Value::from_inner(value)
    }

    /// Get the first function
    pub fn first_function(&self) -> Result<Func<'a>, Error> {
        let value = unsafe { llvm::core::LLVMGetFirstFunction(self.llvm()) };
        Ok(Func(Value::from_inner(value)?))
    }

    /// Get the last function
    pub fn last_function(&self) -> Result<Func<'a>, Error> {
        let value = unsafe { llvm::core::LLVMGetLastFunction(self.llvm()) };
        Ok(Func(Value::from_inner(value)?))
    }

    /// Create a new module from existing IR
    pub fn parse_ir(ctx: &Context, mem_buf: &MemoryBuffer) -> Result<Module<'a>, Error> {
        let mut module = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let ok = unsafe {
            llvm::ir_reader::LLVMParseIRInContext(
                ctx.llvm(),
                mem_buf.llvm(),
                &mut module,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);

        if !ok {
            return Err(Error::Message(message));
        }

        let module = wrap_inner(module)?;

        Ok(Module(module, PhantomData))
    }

    /// Create a new module from existing bitcode
    pub fn parse_bitcode(ctx: &Context, mem_buf: &MemoryBuffer) -> Option<Module<'a>> {
        let mut module = std::ptr::null_mut();
        let ok = unsafe {
            llvm::bit_reader::LLVMParseBitcodeInContext2(ctx.llvm(), mem_buf.llvm(), &mut module)
                == 1
        };

        if !ok {
            return None;
        }

        let module = match wrap_inner(module) {
            Ok(m) => m,
            Err(_) => return None,
        };

        Some(Module(module, PhantomData))
    }

    /// Write module bitcode to file
    pub fn write_bitcode_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<bool, Error> {
        let path = match path.as_ref().to_str() {
            Some(p) => cstr!(p),
            None => return Err(Error::InvalidPath),
        };

        let r =
            unsafe { llvm::bit_writer::LLVMWriteBitcodeToFile(self.llvm(), path.as_ptr()) == 0 };

        Ok(r)
    }

    /// Write module bitcode to in-memory buffer
    pub fn write_bitcode_to_memory_buffer(&self) -> Result<MemoryBuffer, Error> {
        let mem = unsafe { llvm::bit_writer::LLVMWriteBitcodeToMemoryBuffer(self.llvm()) };
        MemoryBuffer::from_raw(mem)
    }

    /// Link another module into `self`
    pub fn link(&mut self, other: &Module) -> bool {
        unsafe {
            let other = llvm::core::LLVMCloneModule(other.llvm());
            llvm::linker::LLVMLinkModules2(self.llvm(), other) == 1
        }
    }

    /// Get type by name
    pub fn type_by_name(&self, name: impl AsRef<str>) -> Result<Type<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe { Type::from_inner(llvm::core::LLVMGetTypeByName(self.llvm(), name.as_ptr())) }
    }

    /// Set WASM32 target/data layout
    pub fn set_wasm32(&mut self) {
        self.set_target("wasm32");
        self.set_data_layout("p:32:32:32");
    }

    /// Set target data
    pub fn set_target_data(&mut self, target: &TargetData) {
        unsafe { llvm::target::LLVMSetModuleDataLayout(self.llvm(), target.llvm()) }
    }

    /// Get target data
    pub fn target_data(&self) -> Result<TargetData, Error> {
        let x = unsafe { llvm::target::LLVMGetModuleDataLayout(self.llvm()) };
        TargetData::from_inner(x)
    }
}

impl<'a> std::fmt::Display for Module<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message =
            unsafe { Message::from_raw(llvm::core::LLVMPrintModuleToString(self.llvm())) };
        write!(fmt, "{}", message.as_ref())
    }
}
