use crate::*;

/// PassManager for module optimizations
pub struct ModulePassManager<'a>(NonNull<llvm::LLVMPassManager>, PhantomData<&'a ()>);

/// PassManager for function optimizations
pub struct FuncPassManager<'a>(NonNull<llvm::LLVMPassManager>, PhantomData<&'a ()>);

/// PassManager trait is used to define common functionality between the two types of PassManagers
pub trait PassManager: LLVM<llvm::LLVMPassManager> {
    /// Kind is used to designate the kind of value that can be optimized using this PassManager
    type Kind;

    /// Run configured optimization passes
    unsafe fn run(&self, f: &Self::Kind) -> bool;

    /// Add optimization passes
    fn add(&self, transforms: impl AsRef<[Transform]>) {
        for transform in transforms.as_ref().iter() {
            unsafe { transform(self.llvm()) }
        }
    }
}

llvm_inner_impl!(ModulePassManager<'a>, llvm::LLVMPassManager);
llvm_inner_impl!(FuncPassManager<'a>, llvm::LLVMPassManager);

/// An optimization pass
pub type Transform = unsafe extern "C" fn(_: *mut llvm::LLVMPassManager);

pub use llvm::transforms;

impl<'a> Drop for ModulePassManager<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposePassManager(self.llvm()) }
    }
}

impl<'a> Drop for FuncPassManager<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposePassManager(self.llvm()) }
    }
}

impl<'a> FuncPassManager<'a> {
    /// Create new function pass manager
    pub fn new(module: &Module<'a>) -> Result<FuncPassManager<'a>, Error> {
        let ptr = unsafe { llvm::core::LLVMCreateFunctionPassManagerForModule(module.llvm()) };

        Ok(FuncPassManager(wrap_inner(ptr)?, PhantomData))
    }
}

impl<'a> ModulePassManager<'a> {
    /// Create new module pass manager
    pub fn new(module: &Module<'a>) -> Result<ModulePassManager<'a>, Error> {
        let ptr = unsafe { llvm::core::LLVMCreateFunctionPassManagerForModule(module.llvm()) };

        Ok(ModulePassManager(wrap_inner(ptr)?, PhantomData))
    }
}

impl<'a> PassManager for FuncPassManager<'a> {
    type Kind = Func<'a>;

    unsafe fn run(&self, f: &Func<'a>) -> bool {
        llvm::core::LLVMRunFunctionPassManager(self.llvm(), f.as_ref().llvm()) == 1
    }
}

impl<'a> PassManager for ModulePassManager<'a> {
    type Kind = Module<'a>;

    unsafe fn run(&self, module: &Module<'a>) -> bool {
        llvm::core::LLVMRunPassManager(self.llvm(), module.llvm()) == 1
    }
}
