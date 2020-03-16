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
        unsafe { llvm::execution_engine::LLVMDisposeExecutionEngine(self.llvm_inner()) }
    }
}

impl<'a> ExecutionEngine<'a> {
    pub fn new(module: &'a Module) -> Result<ExecutionEngine<'a>, Error> {
        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateExecutionEngineForModule(
                &mut engine,
                module.llvm_inner(),
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if !r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn new_jit(module: &'a Module, opt: usize) -> Result<ExecutionEngine<'a>, Error> {
        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateJITCompilerForModule(
                &mut engine,
                module.llvm_inner(),
                opt as u32,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if !r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn new_mcjit(module: &'a Module) -> Result<ExecutionEngine<'a>, Error> {
        let mut engine = std::ptr::null_mut();
        let mut message = std::ptr::null_mut();
        let r = unsafe {
            llvm::execution_engine::LLVMCreateMCJITCompilerForModule(
                &mut engine,
                module.llvm_inner(),
                std::ptr::null_mut(),
                0,
                &mut message,
            ) == 1
        };

        let message = Message::from_raw(message);
        if !r {
            return Err(Error::Message(message));
        }

        Ok(ExecutionEngine(wrap_inner(engine)?, PhantomData))
    }

    pub fn function<T>(&self, name: impl AsRef<str>) -> Result<T, Error> {
        let name = cstr!(name.as_ref());
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetFunctionAddress(self.llvm_inner(), name.as_ptr())
        };

        unsafe { Ok(std::mem::transmute_copy(&(ptr as *mut c_void))) }
    }

    pub fn global_value<T>(&self, name: impl AsRef<str>) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetGlobalValueAddress(self.llvm_inner(), name.as_ptr())
        } as *mut llvm::LLVMValue;
        Value::from_inner(ptr)
    }

    pub fn global<T>(&self, global: impl AsRef<Value<'a>>) -> Result<T, Error> {
        let ptr = unsafe {
            llvm::execution_engine::LLVMGetPointerToGlobal(
                self.llvm_inner(),
                global.as_ref().llvm_inner(),
            )
        };

        unsafe { Ok(std::mem::transmute_copy(&(ptr as *mut c_void))) }
    }

    pub fn run_static_constructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticConstructors(self.llvm_inner()) }
    }

    pub fn run_static_destructors(&self) {
        unsafe { llvm::execution_engine::LLVMRunStaticDestructors(self.llvm_inner()) }
    }

    pub fn add_module(&mut self, module: &Module<'a>) {
        unsafe { llvm::execution_engine::LLVMAddModule(self.llvm_inner(), module.llvm_inner()) }
    }

    pub fn add_global_mapping<T>(&mut self, global: impl AsRef<Value<'a>>, data: &'a T) {
        unsafe {
            llvm::execution_engine::LLVMAddGlobalMapping(
                self.llvm_inner(),
                global.as_ref().llvm_inner(),
                data as *const T as *mut c_void,
            )
        }
    }
}
