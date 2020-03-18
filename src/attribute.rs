use crate::*;

pub struct Attribute<'a>(NonNull<llvm::LLVMOpaqueAttributeRef>, PhantomData<&'a ()>);

llvm_inner_impl!(Attribute<'a>, llvm::LLVMOpaqueAttributeRef);

impl<'a> Attribute<'a> {
    pub(crate) fn from_inner(x: *mut llvm::LLVMOpaqueAttributeRef) -> Result<Attribute<'a>, Error> {
        Ok(Attribute(wrap_inner(x)?, PhantomData))
    }

    /// Create a string attribute
    pub fn new_string(
        ctx: &Context<'a>,
        k: impl AsRef<str>,
        v: impl AsRef<str>,
    ) -> Result<Attribute<'a>, Error> {
        let k = k.as_ref();
        let v = v.as_ref();
        unsafe {
            Attribute::from_inner(llvm::core::LLVMCreateStringAttribute(
                ctx.llvm_inner(),
                k.as_ptr() as *const c_char,
                k.len() as c_uint,
                v.as_ptr() as *const c_char,
                v.len() as c_uint,
            ))
        }
    }

    /// Create an enum attribute
    pub fn new_enum(ctx: &Context<'a>, k: u32, v: u64) -> Result<Attribute<'a>, Error> {
        unsafe {
            Attribute::from_inner(llvm::core::LLVMCreateEnumAttribute(ctx.llvm_inner(), k, v))
        }
    }

    pub fn is_enum(&self) -> bool {
        unsafe { llvm::core::LLVMIsEnumAttribute(self.llvm_inner()) == 1 }
    }

    pub fn is_string(&self) -> bool {
        unsafe { llvm::core::LLVMIsStringAttribute(self.llvm_inner()) == 1 }
    }

    pub fn string_kind(&self) -> Option<&str> {
        if !self.is_string() {
            return None;
        }

        let mut len = 0;
        let ptr = unsafe { llvm::core::LLVMGetStringAttributeKind(self.llvm_inner(), &mut len) };
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(slice) {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    pub fn string_value(&self) -> Option<&str> {
        if !self.is_string() {
            return None;
        }

        let mut len = 0;
        let ptr = unsafe { llvm::core::LLVMGetStringAttributeValue(self.llvm_inner(), &mut len) };
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
        match std::str::from_utf8(slice) {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    pub fn enum_kind(&self) -> Option<i32> {
        if !self.is_enum() {
            return None;
        }
        unsafe { Some(llvm::core::LLVMGetEnumAttributeKind(self.llvm_inner()) as i32) }
    }

    pub fn enum_value(&self) -> Option<u64> {
        if !self.is_enum() {
            return None;
        }
        unsafe { Some(llvm::core::LLVMGetEnumAttributeValue(self.llvm_inner())) }
    }
}
