use crate::*;

pub struct Attribute<'a>(NonNull<llvm::LLVMOpaqueAttributeRef>, PhantomData<&'a ()>);
