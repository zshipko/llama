use crate::*;

pub struct ModulePassManager<'a>(NonNull<llvm::LLVMPassManager>, PhantomData<&'a ()>);
pub struct FunctionPassManager<'a>(NonNull<llvm::LLVMPassManager>, PhantomData<&'a ()>);

pub trait PassManager: LLVMInner<llvm::LLVMPassManager> {
    type Kind;

    fn run(&self, f: &Self::Kind) -> bool;

    fn add(&self, transforms: impl AsRef<[Transform]>) {
        for transform in transforms.as_ref().iter() {
            unsafe { transform(self.llvm_inner()) }
        }
    }
}

llvm_inner_impl!(ModulePassManager<'a>, llvm::LLVMPassManager);
llvm_inner_impl!(FunctionPassManager<'a>, llvm::LLVMPassManager);

pub type Transform = unsafe extern "C" fn(_: *mut llvm::LLVMPassManager);

pub use llvm::transforms;

impl<'a> Drop for ModulePassManager<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposePassManager(self.llvm_inner()) }
    }
}

impl<'a> Drop for FunctionPassManager<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposePassManager(self.llvm_inner()) }
    }
}

impl<'a> FunctionPassManager<'a> {
    pub fn new() -> Result<FunctionPassManager<'a>, Error> {
        let ptr = unsafe { llvm::core::LLVMCreatePassManager() };

        Ok(FunctionPassManager(wrap_inner(ptr)?, PhantomData))
    }
}

impl<'a> ModulePassManager<'a> {
    pub fn new(module: &Module<'a>) -> Result<ModulePassManager<'a>, Error> {
        let ptr =
            unsafe { llvm::core::LLVMCreateFunctionPassManagerForModule(module.llvm_inner()) };

        Ok(ModulePassManager(wrap_inner(ptr)?, PhantomData))
    }
}

impl<'a> PassManager for FunctionPassManager<'a> {
    type Kind = Function<'a>;

    fn run(&self, f: &Function<'a>) -> bool {
        unsafe {
            llvm::core::LLVMRunFunctionPassManager(self.llvm_inner(), f.as_ref().llvm_inner()) == 1
        }
    }
}

impl<'a> PassManager for ModulePassManager<'a> {
    type Kind = Module<'a>;

    fn run(&self, module: &Module<'a>) -> bool {
        unsafe { llvm::core::LLVMRunPassManager(self.llvm_inner(), module.llvm_inner()) == 1 }
    }
}
