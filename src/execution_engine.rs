use crate::*;

pub struct ExecutionEngine<'a>(
    NonNull<llvm::execution_engine::LLVMOpaqueExecutionEngine>,
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
    pub fn new(module: &'a Module) -> Result<ExecutionEngine<'a>, Error> {
        unsafe { llvm::execution_engine::LLVMLinkInInterpreter() }

        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateExecutionEngineForModule(
                &mut engine,
                llvm::core::LLVMCloneModule(module.llvm()),
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn new_jit(module: &'a Module, opt: usize) -> Result<ExecutionEngine<'a>, Error> {
        unsafe { llvm::execution_engine::LLVMLinkInMCJIT() }

        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateJITCompilerForModule(
                &mut engine,
                llvm::core::LLVMCloneModule(module.llvm()),
                opt as u32,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn new_mcjit(module: &'a Module, opt: usize) -> Result<ExecutionEngine<'a>, Error> {
        unsafe { llvm::execution_engine::LLVMLinkInMCJIT() }

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
                llvm::core::LLVMCloneModule(module.llvm()),
                &mut opts,
                std::mem::size_of::<llvm::execution_engine::LLVMMCJITCompilerOptions>(),
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn function<T>(&self, name: impl AsRef<str>) -> Result<T, Error> {
        let name = cstr!(name.as_ref());
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetFunctionAddress(self.llvm(), name.as_ptr())
        };

        unsafe { Ok(std::mem::transmute_copy(&(ptr as *mut c_void))) }
    }

    pub fn global_value<T>(&self, name: impl AsRef<str>) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetGlobalValueAddress(self.llvm(), name.as_ptr())
        } as *mut llvm::LLVMValue;
        Value::from_inner(ptr)
    }

    pub fn global<T>(&self, global: impl AsRef<Value<'a>>) -> Result<*mut T, Error> {
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetPointerToGlobal(
                self.llvm(),
                global.as_ref().llvm(),
            )
        };

        unsafe { Ok(std::mem::transmute_copy(&(ptr as *mut c_void))) }
    }

    pub fn run_static_constructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticConstructors(self.llvm()) }
    }

    pub fn run_static_destructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticDestructors(self.llvm()) }
    }

    pub fn add_module(&mut self, module: &Module<'a>) {
        unsafe { llvm::execution_engine::LLVMAddModule(self.llvm(), module.llvm()) }
    }

    pub fn add_global_mapping<T>(&mut self, global: impl AsRef<Value<'a>>, data: &'a T) {
        unsafe {
            llvm::execution_engine::LLVMAddGlobalMapping(
                self.llvm(),
                global.as_ref().llvm(),
                data as *const T as *mut c_void,
            )
        }
    }

    pub fn target_data(&self) -> Result<TargetData, Error> {
        let x =
            unsafe { llvm::execution_engine::LLVMGetExecutionEngineTargetData(self.llvm()) };
        TargetData::from_inner(x)
    }
}
