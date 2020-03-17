use crate::*;

pub struct Attribute<'a>(NonNull<llvm::LLVMOpaqueAttributeRef>, PhantomData<&'a ()>);

llvm_inner_impl!(Attribute<'a>, llvm::LLVMOpaqueAttributeRef);

impl<'a> Attribute<'a> {
    pub(crate) fn from_inner(x: *mut llvm::LLVMOpaqueAttributeRef) -> Result<Attribute<'a>, Error> {
        Ok(Attribute(wrap_inner(x)?, PhantomData))
    }

    /// Create a string attribute
    pub fn new_string(ctx: &Context<'a>, k: &str, v: &str) -> Result<Attribute<'a>, Error> {
        unsafe {
            Attribute::from_inner(llvm::core::LLVMCreateStringAttribute(
                ctx.llvm_inner(),
                k.as_ptr() as *const i8,
                k.len() as u32,
                v.as_ptr() as *const i8,
                v.len() as u32,
            ))
        }
    }

    /// Create an enum attribute
    pub fn new_enum(ctx: &Context<'a>, k: u32, v: u64) -> Result<Attribute<'a>, Error> {
        unsafe {
            Attribute::from_inner(llvm::core::LLVMCreateEnumAttribute(ctx.llvm_inner(), k, v))
        }
    }
}
