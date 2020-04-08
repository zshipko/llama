use crate::*;

/// LLVMType wrapper
#[derive(Copy)]
pub struct Type<'a>(NonNull<llvm::LLVMType>, PhantomData<&'a ()>);

llvm_inner_impl!(Type<'a>, llvm::LLVMType);

/// Enumerates all possible kinds of types
pub type TypeKind = llvm::LLVMTypeKind;

/// Function type
#[derive(Clone, Copy)]
pub struct FuncType<'a>(pub(crate) Type<'a>);

impl<'a> Clone for Type<'a> {
    fn clone(&self) -> Type<'a> {
        Type(self.0, PhantomData)
    }
}

impl<'a> AsRef<Type<'a>> for Type<'a> {
    fn as_ref(&self) -> &Type<'a> {
        &self
    }
}

impl<'a> AsRef<Type<'a>> for FuncType<'a> {
    fn as_ref(&self) -> &Type<'a> {
        &self.0
    }
}

impl<'a> From<FuncType<'a>> for Type<'a> {
    fn from(x: FuncType<'a>) -> Type<'a> {
        x.0
    }
}

/// Struct type
#[derive(Clone, Copy)]
pub struct StructType<'a>(pub(crate) Type<'a>);

impl<'a> AsRef<Type<'a>> for StructType<'a> {
    fn as_ref(&self) -> &Type<'a> {
        &self.0
    }
}

impl<'a> From<StructType<'a>> for Type<'a> {
    fn from(x: StructType<'a>) -> Type<'a> {
        x.0
    }
}

pub trait LLVMType<'a> {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error>;
}

impl<'a> Type<'a> {
    /// Wrap an LLVMType pointer
    pub fn from_inner(ptr: *mut llvm::LLVMType) -> Result<Type<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Type(t, PhantomData))
    }

    /// Allows for conversion between Rust/LLVM types
    pub fn of<T: LLVMType<'a>>(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        T::llvm_type(ctx)
    }

    /// Get type by name
    pub fn by_name(module: &Module, name: impl AsRef<str>) -> Result<Type<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe { Self::from_inner(llvm::core::LLVMGetTypeByName(module.llvm(), name.as_ptr())) }
    }

    /// Get width of integer type
    pub fn int_width(self) -> usize {
        unsafe { llvm::core::LLVMGetIntTypeWidth(self.llvm()) as usize }
    }

    /// Get associated context
    pub fn context(self) -> Result<Context<'a>, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetTypeContext(self.llvm()))? };
        Ok(Context(ctx, false, PhantomData))
    }

    pub(crate) fn into_context(self) -> Result<Context<'a>, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetTypeContext(self.llvm()))? };
        Ok(Context(ctx, false, PhantomData))
    }

    /// Create int type
    pub fn int(ctx: &Context<'a>, bits: usize) -> Result<Type<'a>, Error> {
        unsafe {
            Self::from_inner(llvm::core::LLVMIntTypeInContext(
                ctx.llvm(),
                bits as std::os::raw::c_uint,
            ))
        }
    }

    /// Create i64 type
    pub fn i64(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Self::int(ctx, 64)
    }

    /// Create i32 type
    pub fn i32(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Self::int(ctx, 32)
    }

    /// Create i16 type
    pub fn i16(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Self::int(ctx, 16)
    }

    /// Create i8 type
    pub fn i8(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Self::int(ctx, 8)
    }

    /// Create i1 type
    pub fn i1(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Self::int(ctx, 1)
    }

    /// Returns true if the types kind matches `kind`
    pub fn is(self, kind: TypeKind) -> bool {
        self.kind() == kind
    }

    /// Create void type
    pub fn void(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMVoidTypeInContext(ctx.llvm())) }
    }

    /// Create label type
    pub fn label(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMLabelTypeInContext(ctx.llvm())) }
    }

    /// Create token type
    pub fn token(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMTokenTypeInContext(ctx.llvm())) }
    }

    /// Create metadata type
    pub fn metadata(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMMetadataTypeInContext(ctx.llvm())) }
    }

    /// Create x86MMX type
    pub fn x86_mmx(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMX86MMXTypeInContext(ctx.llvm())) }
    }

    /// Create half/float16 type
    pub fn half(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMHalfTypeInContext(ctx.llvm())) }
    }

    /// Create float type
    pub fn float(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMFloatTypeInContext(ctx.llvm())) }
    }

    /// Create double type
    pub fn double(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMDoubleTypeInContext(ctx.llvm())) }
    }

    /// Create FP128 type
    pub fn fp128(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMFP128TypeInContext(ctx.llvm())) }
    }

    /// Get type of element
    pub fn element_type(self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMGetElementType(self.llvm()) };
        Self::from_inner(t)
    }

    /// Convert to function type
    pub fn to_func_type(self) -> Result<FuncType<'a>, Error> {
        if self.is(TypeKind::LLVMPointerTypeKind) {
            return self.element_type()?.to_func_type();
        }

        if !self.is(TypeKind::LLVMFunctionTypeKind) {
            println!("TYPE: {:?}", self.element_type()?.kind());
            return Err(Error::InvalidType);
        }

        Ok(FuncType(self))
    }

    /// Convert to struct type
    pub fn to_struct_type(self) -> Result<StructType<'a>, Error> {
        if self.is(TypeKind::LLVMPointerTypeKind) {
            return self.element_type()?.to_struct_type();
        }

        if !self.is(TypeKind::LLVMStructTypeKind) {
            return Err(Error::InvalidType);
        }

        Ok(StructType(self))
    }

    /// Make pointer type
    pub fn pointer(self, address_space: Option<usize>) -> Result<Type<'a>, Error> {
        let address_space = address_space.unwrap_or(0) as c_uint;
        unsafe { Self::from_inner(llvm::core::LLVMPointerType(self.llvm(), address_space)) }
    }

    /// Make vector type
    pub fn vector(self, count: usize) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMVectorType(self.llvm(), count as c_uint)) }
    }

    /// Make array type
    pub fn array(self, count: usize) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMArrayType(self.llvm(), count as c_uint)) }
    }

    /// Get type kind
    pub fn kind(self) -> TypeKind {
        unsafe { llvm::core::LLVMGetTypeKind(self.llvm()) }
    }

    /// Returns true if a type is sized
    pub fn is_sized(self) -> bool {
        unsafe { llvm::core::LLVMTypeIsSized(self.llvm()) == 1 }
    }

    /// Get array length
    pub fn array_len(self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    /// Get vector length
    pub fn vector_len(self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    /// Get pointer address space
    pub fn pointer_address_space(self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    /// Get alignment of type
    pub fn align_of(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMAlignOf(self.llvm())) }
    }

    /// Get size of type
    pub fn size_of(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMSizeOf(self.llvm())) }
    }
}

impl<'a> FuncType<'a> {
    /// Create a new function type with the given return type and parameter types
    pub fn new(
        return_type: impl AsRef<Type<'a>>,
        params: impl AsRef<[Type<'a>]>,
    ) -> Result<FuncType<'a>, Error> {
        let mut params: Vec<*mut llvm::LLVMType> =
            params.as_ref().iter().map(|x| x.llvm()).collect();
        let len = params.len();
        let t = unsafe {
            llvm::core::LLVMFunctionType(
                return_type.as_ref().llvm(),
                params.as_mut_ptr(),
                len as u32,
                0,
            )
        };
        let x = Type::from_inner(t)?;
        x.to_func_type()
    }

    /// Create a new var arg function type with the given return type and parameter types
    pub fn new_var_arg(
        return_type: impl AsRef<Type<'a>>,
        params: impl AsRef<[Type<'a>]>,
    ) -> Result<FuncType<'a>, Error> {
        let mut params: Vec<*mut llvm::LLVMType> =
            params.as_ref().iter().map(|x| x.llvm()).collect();
        let len = params.len();
        let t = unsafe {
            llvm::core::LLVMFunctionType(
                return_type.as_ref().llvm(),
                params.as_mut_ptr(),
                len as u32,
                1,
            )
        };
        let x = Type::from_inner(t)?;
        x.to_func_type()
    }

    /// Returns true if the function type has a variable number of arguments
    pub fn is_var_arg(self) -> bool {
        unsafe { llvm::core::LLVMIsFunctionVarArg(self.as_ref().llvm()) == 1 }
    }

    /// Get the return type
    pub fn return_type(self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMGetReturnType(self.as_ref().llvm()) };
        Type::from_inner(t)
    }

    /// Get the number of parameters
    pub fn param_count(self) -> usize {
        let n = unsafe { llvm::core::LLVMCountParamTypes(self.as_ref().llvm()) };
        n as usize
    }

    /// Get all parameter types
    pub fn params(self) -> Vec<Type<'a>> {
        let len = self.param_count();
        let mut data = vec![std::ptr::null_mut(); len];

        unsafe { llvm::core::LLVMGetParamTypes(self.as_ref().llvm(), data.as_mut_ptr()) }
        data.into_iter()
            .map(|x| Type::from_inner(x).unwrap())
            .collect()
    }
}

impl<'a> StructType<'a> {
    /// Create a new struct with the given fields
    pub fn new(ctx: &Context<'a>, fields: impl AsRef<[Type<'a>]>) -> Result<StructType<'a>, Error> {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        let t = unsafe {
            llvm::core::LLVMStructTypeInContext(ctx.llvm(), fields.as_mut_ptr(), len as u32, 0)
        };
        Type::from_inner(t)?.to_struct_type()
    }

    /// Create a new packed struct with the given fields
    pub fn new_packed(
        ctx: &Context<'a>,
        fields: impl AsRef<[Type<'a>]>,
    ) -> Result<StructType<'a>, Error> {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        let t = unsafe {
            llvm::core::LLVMStructTypeInContext(ctx.llvm(), fields.as_mut_ptr(), len as u32, 1)
        };
        Type::from_inner(t)?.to_struct_type()
    }

    /// Create a new named struct
    pub fn new_with_name(
        ctx: &Context<'a>,
        name: impl AsRef<str>,
    ) -> Result<StructType<'a>, Error> {
        let name = cstr!(name.as_ref());
        let t = unsafe { llvm::core::LLVMStructCreateNamed(ctx.llvm(), name.as_ptr()) };
        Type::from_inner(t)?.to_struct_type()
    }

    /// Get struct name
    pub fn name(self) -> Result<Option<&'a str>, Error> {
        unsafe {
            let s = llvm::core::LLVMGetStructName(self.as_ref().llvm());
            if s.is_null() {
                return Ok(None);
            }
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8(s)?;
            Ok(Some(s))
        }
    }

    /// Set named struct body
    pub fn set_body(&mut self, fields: impl AsRef<[Type<'a>]>) {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        unsafe {
            llvm::core::LLVMStructSetBody(self.as_ref().llvm(), fields.as_mut_ptr(), len as u32, 0)
        };
    }

    /// Set packed, named struct's body
    pub fn set_body_packed(&mut self, fields: impl AsRef<[Type<'a>]>) {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        unsafe {
            llvm::core::LLVMStructSetBody(self.as_ref().llvm(), fields.as_mut_ptr(), len as u32, 1)
        };
    }

    /// Get number of fields
    pub fn field_count(self) -> usize {
        let n = unsafe { llvm::core::LLVMCountStructElementTypes(self.as_ref().llvm()) };
        n as usize
    }

    /// Get all fields
    pub fn fields(self) -> Vec<Type<'a>> {
        let len = self.field_count();
        let mut ptr = std::ptr::null_mut();

        unsafe { llvm::core::LLVMGetStructElementTypes(self.as_ref().llvm(), &mut ptr) }
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        slice
            .iter_mut()
            .map(|x| Type::from_inner(x).unwrap())
            .collect()
    }

    /// Get a single field by index
    pub fn field(self, index: usize) -> Result<Type<'a>, Error> {
        let t =
            unsafe { llvm::core::LLVMStructGetTypeAtIndex(self.as_ref().llvm(), index as c_uint) };

        Type::from_inner(t)
    }

    /// Returns true if the struct is packed
    pub fn is_packed(self) -> bool {
        unsafe { llvm::core::LLVMIsPackedStruct(self.as_ref().llvm()) == 1 }
    }

    /// Returns true if the struct is an opaque type
    pub fn is_opaque(self) -> bool {
        unsafe { llvm::core::LLVMIsOpaqueStruct(self.as_ref().llvm()) == 1 }
    }

    /// Returns true if the struct is a literal
    pub fn is_literal(self) -> bool {
        unsafe { llvm::core::LLVMIsLiteralStruct(self.as_ref().llvm()) == 1 }
    }
}

impl<'a> std::fmt::Display for Type<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let s = Message::from_raw(llvm::core::LLVMPrintTypeToString(self.llvm()));
            write!(fmt, "{}", s.as_ref())
        }
    }
}

impl<'a> LLVMType<'a> for u8 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 8)
    }
}

impl<'a> LLVMType<'a> for i8 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 8)
    }
}

impl<'a> LLVMType<'a> for u16 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 16)
    }
}

impl<'a> LLVMType<'a> for i16 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 16)
    }
}

impl<'a> LLVMType<'a> for u32 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 32)
    }
}

impl<'a> LLVMType<'a> for i32 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 32)
    }
}

impl<'a> LLVMType<'a> for u64 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 64)
    }
}

impl<'a> LLVMType<'a> for i64 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 64)
    }
}

impl<'a> LLVMType<'a> for u128 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 128)
    }
}

impl<'a> LLVMType<'a> for i128 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::int(ctx, 128)
    }
}

impl<'a> LLVMType<'a> for f32 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::float(ctx)
    }
}

impl<'a> LLVMType<'a> for f64 {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::double(ctx)
    }
}

impl<'a> LLVMType<'a> for () {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        Type::void(ctx)
    }
}

impl<'a, T: LLVMType<'a>> LLVMType<'a> for *const T {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        T::llvm_type(ctx)?.pointer(None)
    }
}

impl<'a, T: LLVMType<'a>> LLVMType<'a> for *mut T {
    fn llvm_type(ctx: &Context<'a>) -> Result<Type<'a>, Error> {
        T::llvm_type(ctx)?.pointer(None)
    }
}
