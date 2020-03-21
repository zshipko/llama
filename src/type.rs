use crate::*;

#[derive(Copy)]
pub struct Type<'a>(NonNull<llvm::LLVMType>, PhantomData<&'a ()>);

llvm_inner_impl!(Type<'a>, llvm::LLVMType);

pub type TypeKind = llvm::LLVMTypeKind;

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

impl<'a> Type<'a> {
    pub(crate) fn from_inner(ptr: *mut llvm::LLVMType) -> Result<Type<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Type(t, PhantomData))
    }

    pub fn by_name(module: &Module, name: impl AsRef<str>) -> Result<Type<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe { Self::from_inner(llvm::core::LLVMGetTypeByName(module.llvm(), name.as_ptr())) }
    }

    pub fn int_width(&self) -> usize {
        unsafe { llvm::core::LLVMGetIntTypeWidth(self.llvm()) as usize }
    }

    pub fn context(&self) -> Result<Context, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetTypeContext(self.llvm()))? };
        Ok(Context(ctx, false, PhantomData))
    }

    pub(crate) fn into_context(self) -> Result<Context<'a>, Error> {
        let ctx = unsafe { wrap_inner(llvm::core::LLVMGetTypeContext(self.llvm()))? };
        Ok(Context(ctx, false, PhantomData))
    }

    pub fn int(ctx: &'a Context, bits: usize) -> Result<Type<'a>, Error> {
        unsafe {
            Self::from_inner(llvm::core::LLVMIntTypeInContext(
                ctx.llvm(),
                bits as std::os::raw::c_uint,
            ))
        }
    }

    pub fn i64(ctx: &'a Context) -> Result<Type<'a>, Error> {
        Self::int(ctx, 64)
    }

    pub fn i32(ctx: &'a Context) -> Result<Type<'a>, Error> {
        Self::int(ctx, 32)
    }

    pub fn i16(ctx: &'a Context) -> Result<Type<'a>, Error> {
        Self::int(ctx, 16)
    }

    pub fn i8(ctx: &'a Context) -> Result<Type<'a>, Error> {
        Self::int(ctx, 8)
    }

    pub fn i1(ctx: &'a Context) -> Result<Type<'a>, Error> {
        Self::int(ctx, 1)
    }

    pub fn is(&self, kind: TypeKind) -> bool {
        self.kind() == kind
    }

    pub fn void(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMVoidTypeInContext(ctx.llvm())) }
    }

    pub fn label(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMLabelTypeInContext(ctx.llvm())) }
    }

    pub fn token(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMTokenTypeInContext(ctx.llvm())) }
    }

    pub fn metadata(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMMetadataTypeInContext(ctx.llvm())) }
    }

    pub fn x86_mmx(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMX86MMXTypeInContext(ctx.llvm())) }
    }

    pub fn half(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMHalfTypeInContext(ctx.llvm())) }
    }

    pub fn float(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMFloatTypeInContext(ctx.llvm())) }
    }

    pub fn double(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMDoubleTypeInContext(ctx.llvm())) }
    }

    pub fn fp128(ctx: &'a Context) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMFP128TypeInContext(ctx.llvm())) }
    }

    pub fn element_type(&self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMGetElementType(self.llvm()) };
        Self::from_inner(t)
    }

    pub fn into_func_type(self) -> Result<FuncType<'a>, Error> {
        if !self.is(TypeKind::LLVMFunctionTypeKind) {
            return Err(Error::InvalidType);
        }

        Ok(FuncType(self))
    }

    pub fn into_struct_type(self) -> Result<StructType<'a>, Error> {
        if !self.is(TypeKind::LLVMStructTypeKind) {
            return Err(Error::InvalidType);
        }

        Ok(StructType(self))
    }

    pub fn pointer(&self, address_space: Option<usize>) -> Result<Type<'a>, Error> {
        let address_space = address_space.unwrap_or(0) as c_uint;
        unsafe { Self::from_inner(llvm::core::LLVMPointerType(self.llvm(), address_space)) }
    }

    pub fn vector(&self, count: usize) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMVectorType(self.llvm(), count as c_uint)) }
    }

    pub fn array(&self, count: usize) -> Result<Type<'a>, Error> {
        unsafe { Self::from_inner(llvm::core::LLVMArrayType(self.llvm(), count as c_uint)) }
    }

    pub fn kind(&self) -> TypeKind {
        unsafe { llvm::core::LLVMGetTypeKind(self.llvm()) }
    }

    pub fn is_sized(&self) -> bool {
        unsafe { llvm::core::LLVMTypeIsSized(self.llvm()) == 1 }
    }

    pub fn array_len(&self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    pub fn vector_len(&self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    pub fn pointer_address_space(&self) -> usize {
        unsafe { llvm::core::LLVMGetArrayLength(self.llvm()) as usize }
    }

    pub fn align_of(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMAlignOf(self.llvm())) }
    }

    pub fn size_of(self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMSizeOf(self.llvm())) }
    }
}

impl<'a> FuncType<'a> {
    pub fn new(
        return_type: impl AsRef<Type<'a>>,
        params: impl AsRef<[&'a Type<'a>]>,
        var_arg: bool,
    ) -> Result<FuncType<'a>, Error> {
        let mut params: Vec<*mut llvm::LLVMType> =
            params.as_ref().iter().map(|x| x.llvm()).collect();
        let len = params.len();
        let t = unsafe {
            llvm::core::LLVMFunctionType(
                return_type.as_ref().llvm(),
                params.as_mut_ptr(),
                len as u32,
                if var_arg { 1 } else { 0 },
            )
        };
        let x = Type::from_inner(t)?;
        x.into_func_type()
    }

    pub fn is_var_arg(&self) -> bool {
        unsafe { llvm::core::LLVMIsFunctionVarArg(self.as_ref().llvm()) == 1 }
    }

    pub fn return_type(&self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMGetReturnType(self.as_ref().llvm()) };
        Type::from_inner(t)
    }

    pub fn param_count(&self) -> usize {
        let n = unsafe { llvm::core::LLVMCountParamTypes(self.as_ref().llvm()) };
        n as usize
    }

    pub fn params(&self) -> Vec<Type<'a>> {
        let len = self.param_count();
        let mut data = vec![std::ptr::null_mut(); len];

        unsafe { llvm::core::LLVMGetParamTypes(self.as_ref().llvm(), data.as_mut_ptr()) }
        data.into_iter()
            .map(|x| Type::from_inner(x).unwrap())
            .collect()
    }
}

impl<'a> StructType<'a> {
    pub fn new(
        ctx: &'a Context,
        fields: impl AsRef<[&'a Type<'a>]>,
        packed: bool,
    ) -> Result<StructType<'a>, Error> {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        let t = unsafe {
            llvm::core::LLVMStructTypeInContext(
                ctx.llvm(),
                fields.as_mut_ptr(),
                len as u32,
                if packed { 1 } else { 0 },
            )
        };
        Type::from_inner(t)?.into_struct_type()
    }

    pub fn new_named(ctx: &'a Context, name: impl AsRef<str>) -> Result<StructType<'a>, Error> {
        let name = cstr!(name.as_ref());
        let t = unsafe { llvm::core::LLVMStructCreateNamed(ctx.llvm(), name.as_ptr()) };
        Type::from_inner(t)?.into_struct_type()
    }

    pub fn name(&self) -> Result<&str, Error> {
        unsafe {
            let s = llvm::core::LLVMGetStructName(self.as_ref().llvm());
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    pub fn set_body<'b>(&'b mut self, fields: impl AsRef<[&'b Type<'a>]>, packed: bool) {
        let mut fields: Vec<*mut llvm::LLVMType> =
            fields.as_ref().iter().map(|x| x.llvm()).collect();
        let len = fields.len();
        unsafe {
            llvm::core::LLVMStructSetBody(
                self.as_ref().llvm(),
                fields.as_mut_ptr(),
                len as u32,
                if packed { 1 } else { 0 },
            )
        };
    }

    pub fn field_count(&self) -> usize {
        let n = unsafe { llvm::core::LLVMCountStructElementTypes(self.as_ref().llvm()) };
        n as usize
    }

    pub fn fields(&self) -> Vec<Type<'a>> {
        let len = self.field_count();
        let mut ptr = std::ptr::null_mut();

        unsafe { llvm::core::LLVMGetStructElementTypes(self.as_ref().llvm(), &mut ptr) }
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        slice
            .iter_mut()
            .map(|x| Type::from_inner(x).unwrap())
            .collect()
    }

    pub fn field(&self, index: usize) -> Result<Type<'a>, Error> {
        let t =
            unsafe { llvm::core::LLVMStructGetTypeAtIndex(self.as_ref().llvm(), index as c_uint) };

        Type::from_inner(t)
    }

    pub fn is_packed(&self) -> bool {
        unsafe { llvm::core::LLVMIsPackedStruct(self.as_ref().llvm()) == 1 }
    }

    pub fn is_opaque(&self) -> bool {
        unsafe { llvm::core::LLVMIsOpaqueStruct(self.as_ref().llvm()) == 1 }
    }

    pub fn is_literal(&self) -> bool {
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
