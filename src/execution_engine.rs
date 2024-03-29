use crate::*;

/// An execution engine can be used to execute JIT compiled code
pub struct ExecutionEngine<'a>(
    NonNull<llvm::execution_engine::LLVMOpaqueExecutionEngine>,
    Module<'a>,
    PhantomData<&'a ()>,
);

llvm_inner_impl!(
    ExecutionEngine<'a>,
    llvm::execution_engine::LLVMOpaqueExecutionEngine
);

impl<'a> Drop for ExecutionEngine<'a> {
    fn drop(&mut self) {
        unsafe { llvm::execution_engine::LLVMDisposeExecutionEngine(self.llvm()) }
    }
}

impl<'a> ExecutionEngine<'a> {
    /// Create a new execution engine using `LLVMCreateExectionEngineForModule`
    pub fn new(module: Module<'a>) -> Result<ExecutionEngine<'a>, Error> {
        unsafe { llvm::execution_engine::LLVMLinkInInterpreter() }

        if module
            .1
            .compare_exchange(
                true,
                false,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(Error::ModuleIsAlreadyOwned);
        }

        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateExecutionEngineForModule(
                &mut engine,
                module.llvm(),
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if r {
            module.1.store(true, std::sync::atomic::Ordering::Relaxed);
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, module, PhantomData))
    }

    /// Create new JIT compiler with optimization level
    pub fn new_jit(module: Module<'a>, opt: usize) -> Result<ExecutionEngine<'a>, Error> {
        unsafe { llvm::execution_engine::LLVMLinkInMCJIT() }

        if module
            .1
            .compare_exchange(
                true,
                false,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(Error::ModuleIsAlreadyOwned);
        }

        let mut opts = llvm::execution_engine::LLVMMCJITCompilerOptions {
            OptLevel: opt as c_uint,
            CodeModel: llvm::target_machine::LLVMCodeModel::LLVMCodeModelJITDefault,
            NoFramePointerElim: 0,
            EnableFastISel: 0,
            MCJMM: std::ptr::null_mut(),
        };
        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateMCJITCompilerForModule(
                &mut engine,
                module.llvm(),
                &mut opts,
                std::mem::size_of::<llvm::execution_engine::LLVMMCJITCompilerOptions>(),
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if r {
            module.1.store(true, std::sync::atomic::Ordering::Relaxed);
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, module, PhantomData))
    }

    /// Get a function from within the execution engine
    ///
    /// # Safety
    /// This function does nothing to ensure that the function actually matches the type you give
    /// it
    pub unsafe fn function<T: 'a + Copy>(&self, name: impl AsRef<str>) -> Result<T, Error> {
        let name = cstr!(name.as_ref());
        let ptr = llvm::execution_engine::LLVMGetFunctionAddress(self.llvm(), name.as_ptr());
        Ok(std::mem::transmute_copy(&(ptr as *mut c_void)))
    }

    /// Get a pointer to a global value from within the execution engine
    ///
    /// # Safety
    /// This function does nothing to ensure that the function actually matches the type you give
    /// it
    pub unsafe fn global_value<T>(&self, name: impl AsRef<str>) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        let ptr = llvm::execution_engine::LLVMGetGlobalValueAddress(self.llvm(), name.as_ptr())
            as *mut llvm::LLVMValue;
        Value::from_inner(ptr)
    }

    /// Get a pointer to a global from within the execution engine
    ///
    /// # Safety
    /// This function does nothing to ensure that the function actually matches the type you give
    /// it
    pub unsafe fn global<T: 'a>(&self, global: impl AsRef<Value<'a>>) -> Result<&mut T, Error> {
        let ptr =
            llvm::execution_engine::LLVMGetPointerToGlobal(self.llvm(), global.as_ref().llvm());

        Ok(&mut *(ptr as *mut T))
    }

    /// Run static constructors
    pub fn run_static_constructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticConstructors(self.llvm()) }
    }

    /// Run static destructors
    pub fn run_static_destructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticDestructors(self.llvm()) }
    }

    /// Add mapping between global value and a local object
    pub fn add_global_mapping<T: 'a>(&mut self, global: impl AsRef<Value<'a>>, data: *mut T) {
        unsafe {
            llvm::execution_engine::LLVMAddGlobalMapping(
                self.llvm(),
                global.as_ref().llvm(),
                data as *mut c_void,
            )
        }
    }

    /// Get target data
    pub fn target_data(&self) -> Result<TargetData, Error> {
        let x = unsafe { llvm::execution_engine::LLVMGetExecutionEngineTargetData(self.llvm()) };
        TargetData::from_inner(x)
    }

    /// Get handle to underlying module
    pub fn module(&self) -> &Module<'a> {
        &self.1
    }

    /// Get mutable handle to underlying module
    pub fn module_mut(&mut self) -> &mut Module<'a> {
        &mut self.1
    }

    /// Get inner module
    pub fn into_module(self) -> Result<Module<'a>, Error> {
        let mut message = std::ptr::null_mut();
        let mut ptr = std::ptr::null_mut();
        let rc = unsafe {
            llvm::execution_engine::LLVMRemoveModule(
                self.llvm(),
                self.1.llvm(),
                &mut ptr,
                &mut message as *mut *mut std::os::raw::c_char,
            )
        };

        let message = Message::from_raw(message);
        if rc > 0 {
            return Err(Error::Message(message));
        }

        let ptr = match std::ptr::NonNull::new(ptr) {
            Some(x) => x,
            None => return Err(Error::NullPointer),
        };

        Ok(Module(
            ptr,
            std::sync::atomic::AtomicBool::new(true),
            PhantomData,
        ))
    }
}
