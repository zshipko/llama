use crate::*;

pub struct Attribute<'a>(NonNull<llvm::LLVMOpaqueAttributeRef>, PhantomData<&'a ()>);

llvm_inner_impl!(Attribute<'a>, llvm::LLVMOpaqueAttributeRef);

impl<'a> Attribute<'a> {
    pub(crate) fn from_raw(x: *mut llvm::LLVMOpaqueAttributeRef) -> Result<Attribute<'a>, Error> {
        Ok(Attribute(wrap_inner(x)?, PhantomData))
    }
}
