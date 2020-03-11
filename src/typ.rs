use crate::*;

pub struct Type<'a>(NonNull<llvm::LLVMType>, PhantomData<&'a ()>);

llvm_inner_impl!(Type<'a>, llvm::LLVMType);

pub type TypeKind = llvm::LLVMTypeKind;

impl<'a> Type<'a> {
    pub(crate) fn from_inner(ptr: *mut llvm::LLVMType) -> Result<Type<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Type(t, PhantomData))
    }

    pub fn int(ctx: &'a Context, bits: usize) -> Result<Type<'a>, Error> {
        unsafe {
            Self::from_inner(llvm::core::LLVMIntTypeInContext(
                ctx.llvm_inner(),
                bits as std::os::raw::c_uint,
            ))
        }
    }

    pub fn void(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMVoidTypeInContext(ctx.llvm_inner())) }
    }

    pub fn float(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMFloatTypeInContext(ctx.llvm_inner())) }
    }

    pub fn double(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMDoubleTypeInContext(ctx.llvm_inner())) }
    }

    pub fn pointer(&self, address_space: Option<usize>) -> Result<Type<'a>, Error> {
        let address_space = address_space.unwrap_or(0) as c_uint;
        unsafe {
            Self::from_inner(llvm::core::LLVMPointerType(
                self.llvm_inner(),
                address_space,
            ))
        }
    }

    pub fn vector(&self, count: usize) -> Result<Type<'a>, Error> {
        unsafe {
            Self::from_inner(llvm::core::LLVMVectorType(
                self.llvm_inner(),
                count as c_uint,
            ))
        }
    }

    pub fn array(&self, count: usize) -> Result<Type<'a>, Error> {
        unsafe {
            Self::from_inner(llvm::core::LLVMArrayType(
                self.llvm_inner(),
                count as c_uint,
            ))
        }
    }

    pub fn kind(&self) -> TypeKind {
        unsafe { llvm::core::LLVMGetTypeKind(self.llvm_inner()) }
    }

    pub fn is_sized(&self) -> bool {
        unsafe { llvm::core::LLVMTypeIsSized(self.llvm_inner()) == 1 }
    }
}

impl<'a> std::fmt::Display for Type<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let s = Message::from_raw(llvm::core::LLVMPrintTypeToString(self.llvm_inner()));
            write!(fmt, "{}", s.as_ref())
        }
    }
}
