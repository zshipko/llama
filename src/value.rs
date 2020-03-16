use crate::*;

pub struct Value<'a>(NonNull<llvm::LLVMValue>, PhantomData<&'a ()>);

llvm_inner_impl!(Value<'a>, llvm::LLVMValue);

pub type ValueKind = llvm::LLVMValueKind;

pub struct Const<'a>(Value<'a>);

impl<'a> AsRef<Value<'a>> for Value<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self
    }
}

impl<'a> AsRef<Value<'a>> for Const<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Const<'a>> for Value<'a> {
    fn from(x: Const<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Value<'a> {
    pub(crate) fn from_inner(ptr: *mut llvm::LLVMValue) -> Result<Value<'a>, Error> {
        let t = wrap_inner(ptr)?;
        Ok(Value(t, PhantomData))
    }

    pub fn is_basic_block(&self) -> bool {
        unsafe { llvm::core::LLVMValueIsBasicBlock(self.llvm_inner()) == 0 }
    }

    pub fn kind(&self) -> ValueKind {
        unsafe { llvm::core::LLVMGetValueKind(self.llvm_inner()) }
    }

    pub fn is(&self, kind: ValueKind) -> bool {
        self.kind() == kind
    }

    pub fn type_of(&self) -> Result<Type<'a>, Error> {
        let t = unsafe { llvm::core::LLVMTypeOf(self.llvm_inner()) };
        Type::from_inner(t)
    }

    pub(crate) fn into_context(self) -> Result<Context<'a>, Error> {
        self.type_of()?.into_context()
    }

    pub fn context(&self) -> Result<Context, Error> {
        self.type_of()?.into_context()
    }

    pub fn name(&self) -> Result<&str, Error> {
        let mut size = 0;
        unsafe {
            let s = llvm::core::LLVMGetValueName2(self.llvm_inner(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, size);
            let s = std::str::from_utf8(s)?;
            Ok(s)
        }
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        let len = name.as_ref().len();
        let name = cstr!(name.as_ref());
        unsafe { llvm::core::LLVMSetValueName2(self.llvm_inner(), name.as_ptr(), len) }
    }

    pub fn replace_all_uses_with(&self, other: impl AsRef<Value<'a>>) {
        unsafe {
            llvm::core::LLVMReplaceAllUsesWith(self.llvm_inner(), other.as_ref().llvm_inner())
        }
    }

    pub fn is_const(&self) -> bool {
        unsafe { llvm::core::LLVMIsConstant(self.llvm_inner()) == 1 }
    }

    pub fn into_const(self) -> Result<Const<'a>, Error> {
        if !self.is_const() {
            return Err(Error::InvalidConst);
        }

        Ok(Const(self))
    }

    pub fn to_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        if !self.is_basic_block() {
            return Err(Error::InvalidBasicBlock);
        }

        let ptr = unsafe { llvm::core::LLVMValueAsBasicBlock(self.llvm_inner()) };
        BasicBlock::from_inner(ptr)
    }

    pub fn is_undef(&self) -> bool {
        unsafe { llvm::core::LLVMIsUndef(self.llvm_inner()) == 1 }
    }

    pub fn is_null(&self) -> bool {
        unsafe { llvm::core::LLVMIsNull(self.llvm_inner()) == 1 }
    }

    pub fn is_constant_string(&self) -> bool {
        unsafe { llvm::core::LLVMIsConstantString(self.llvm_inner()) == 1 }
    }

    pub fn count_basic_blocks(&self) -> usize {
        unsafe { llvm::core::LLVMCountBasicBlocks(self.llvm_inner()) as usize }
    }

    pub fn basic_blocks(&self) -> Vec<BasicBlock<'a>> {
        let count = self.count_basic_blocks();
        let ptr = std::ptr::null_mut();
        unsafe { llvm::core::LLVMGetBasicBlocks(self.llvm_inner(), ptr) }
        let slice = unsafe { std::slice::from_raw_parts(ptr, count) };
        slice
            .into_iter()
            .map(|x| BasicBlock::from_inner(*x).unwrap())
            .collect()
    }

    pub fn first_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetFirstBasicBlock(self.llvm_inner()) })
    }

    pub fn last_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetLastBasicBlock(self.llvm_inner()) })
    }

    pub fn entry_basic_block(&self) -> Result<BasicBlock<'a>, Error> {
        BasicBlock::from_inner(unsafe { llvm::core::LLVMGetEntryBasicBlock(self.llvm_inner()) })
    }

    pub fn append_basic_block(&self, bb: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMAppendExistingBasicBlock(self.llvm_inner(), bb.llvm_inner()) }
    }
}

impl<'a> Const<'a> {
    instr!(int(t: impl AsRef<&'a Type<'a>>, i: i64, sign_extend: bool) {
        llvm::core::LLVMConstInt(
            t.as_ref().llvm_inner(),
            i as u64,
            sign_extend as c_int,
        )
    });

    instr!(int_s(
        t: impl AsRef<&'a Type<'a>>,
        i: impl AsRef<str>,
        radix: u8,
    ) {
        let s =cstr!(i.as_ref());
        llvm::core::LLVMConstIntOfString(
            t.as_ref().llvm_inner(),
            s.as_ptr(),
            radix,
        )
    });

    instr!(real(t: impl AsRef<&'a Type<'a>>, i: f64) {
        llvm::core::LLVMConstReal(t.as_ref().llvm_inner(), i as f64)
    });

    instr!(real_s(
        t: impl AsRef<&'a Type<'a>>,
        i: impl AsRef<str>,
    ) {
        let s = cstr!(i.as_ref());
        llvm::core::LLVMConstRealOfString(
            t.as_ref().llvm_inner(),
            s.as_ptr(),
        )
    });

    instr!(undef(t: impl AsRef<&'a Type<'a>>)  {
        llvm::core::LLVMGetUndef(t.as_ref().llvm_inner())
    });

    instr!(pointer_null(t: impl AsRef<&'a Type<'a>>){
        llvm::core::LLVMConstPointerNull(t.as_ref().llvm_inner())
    });

    instr!(null(t: impl AsRef<&'a Type<'a>>){
        llvm::core::LLVMConstNull(t.as_ref().llvm_inner())
    });

    instr!(all_ones(t: impl AsRef<&'a Type<'a>>){
        llvm::core::LLVMConstAllOnes(t.as_ref().llvm_inner())
    });

    instr!(string(
        ctx: &'a Context,
        s: impl AsRef<str>,
        no_null_terminator: bool,
    ) {
        let len = s.as_ref().len();
        let s = cstr!(s.as_ref());
        llvm::core::LLVMConstStringInContext(
            ctx.llvm_inner(),
            s.as_ptr(),
            len as c_uint,
            if no_null_terminator { 1 } else { 0 },
        )
    });

    pub fn get_unsigned_int(&self) -> Option<u64> {
        if !self.as_ref().is(ValueKind::LLVMConstantIntValueKind) {
            return None;
        }

        unsafe {
            Some(llvm::core::LLVMConstIntGetZExtValue(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn get_signed_int(&self) -> Option<i64> {
        if !self.as_ref().is(ValueKind::LLVMConstantIntValueKind) {
            return None;
        }

        unsafe {
            Some(llvm::core::LLVMConstIntGetSExtValue(
                self.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn get_double(&self) -> Option<f64> {
        if !self.as_ref().is(ValueKind::LLVMConstantFPValueKind) {
            return None;
        }

        let mut _l = 0;
        unsafe {
            Some(llvm::core::LLVMConstRealGetDouble(
                self.as_ref().llvm_inner(),
                &mut _l,
            ))
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        let mut size = 0;
        unsafe {
            let s = llvm::core::LLVMGetAsString(self.as_ref().llvm_inner(), &mut size);
            let s = std::slice::from_raw_parts(s as *const u8, strlen(s));
            match std::str::from_utf8(s) {
                Ok(x) => Some(x),
                Err(_) => None,
            }
        }
    }

    pub fn get_element(&self, index: usize) -> Result<Value<'a>, Error> {
        let v = unsafe {
            llvm::core::LLVMGetElementAsConstant(self.as_ref().llvm_inner(), index as c_uint)
        };
        Value::from_inner(v)
    }

    pub fn crate_struct(
        ctx: &'a Context,
        vals: impl AsRef<[&'a Value<'a>]>,
        packed: bool,
    ) -> Result<Value<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstStructInContext(
                ctx.llvm_inner(),
                vals.as_mut_ptr(),
                len as c_uint,
                if packed { 1 } else { 0 },
            )
        };
        Value::from_inner(v)
    }

    pub fn create_named_struct(
        t: impl AsRef<Type<'a>>,
        vals: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Value<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstNamedStruct(
                t.as_ref().llvm_inner(),
                vals.as_mut_ptr(),
                len as c_uint,
            )
        };
        Value::from_inner(v)
    }

    pub fn create_array(
        t: impl AsRef<Type<'a>>,
        vals: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Value<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstArray(t.as_ref().llvm_inner(), vals.as_mut_ptr(), len as c_uint)
        };
        Value::from_inner(v)
    }

    pub fn create_vector(vals: impl AsRef<[&'a Value<'a>]>) -> Result<Value<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe { llvm::core::LLVMConstVector(vals.as_mut_ptr(), len as c_uint) };
        Value::from_inner(v)
    }

    pub fn opcode(&self) -> OpCode {
        unsafe { llvm::core::LLVMGetConstOpcode(self.as_ref().llvm_inner()) }
    }

    pub fn neg(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstNeg(self.as_ref().llvm_inner())) }
    }

    pub fn nsw_neg(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstNSWNeg(self.as_ref().llvm_inner())) }
    }

    pub fn nuw_neg(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstNUWNeg(self.as_ref().llvm_inner())) }
    }

    pub fn fneg(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstFNeg(self.as_ref().llvm_inner())) }
    }

    pub fn not(&self) -> Result<Value<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstNot(self.as_ref().llvm_inner())) }
    }

    pub fn add(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nsw_add(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nuw_add(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fadd(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn sub(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nsw_sub(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nuw_sub(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fsub(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn mul(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nsw_mul(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn nuw_mul(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fmul(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn udiv(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstUDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn exact_udiv(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstExactUDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn sdiv(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn exact_sdiv(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstExactSDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fdiv(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn urem(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstURem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn srem(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSRem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn frem(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFRem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn and(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAnd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn or(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstOr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn xor(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstXor(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn icmp(&self, pred: IntPredicate, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstICmp(
                pred,
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fcmp(&self, pred: RealPredicate, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFCmp(
                pred,
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn shl(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstShl(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn lshr(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstLShr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn ashr(&self, other: &Const<'a>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAShr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn gep(&self, i: impl AsRef<[&'a Value<'a>]>) -> Result<Value<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> =
            i.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstGEP(
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))
        }
    }

    pub fn gep2(
        &self,
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Value<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> =
            i.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstGEP2(
                t.as_ref().llvm_inner(),
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))
        }
    }

    pub fn in_bounds_gep(&self, i: impl AsRef<[&'a Value<'a>]>) -> Result<Value<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> =
            i.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstInBoundsGEP(
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))
        }
    }

    pub fn in_bounds_gep2(
        &self,
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Value<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> =
            i.as_ref().into_iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstInBoundsGEP2(
                t.as_ref().llvm_inner(),
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))
        }
    }

    pub fn trunc(&self, t: impl AsRef<&'a Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstTrunc(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn sext(&self, t: impl AsRef<&'a Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn zext(&self, t: impl AsRef<&'a Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstZExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fp_trunc(&self, t: impl AsRef<&'a Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPTrunc(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fp_ext(&self, t: impl AsRef<&'a Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn ui_to_fp(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstUIToFP(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn si_to_fp(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSIToFP(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fp_to_ui(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPToUI(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn fp_to_si(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPToSI(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn ptr_to_int(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstPtrToInt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn int_to_ptr(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstIntToPtr(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn addr_space_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAddrSpaceCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn zext_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstZExtOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn sext_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSExtOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    pub fn trunc_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstTruncOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))
        }
    }

    instr!(pointer_cast(&self, t: impl AsRef<Type<'a>>) {
        llvm::core::LLVMConstPointerCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner())
    });

    instr!(int_cast(&self, t: impl AsRef<Type<'a>>, signed: bool) {
        llvm::core::LLVMConstIntCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner(), if signed { 1 } else { 0 })
    });

    instr!(fp_cast(&self, t: impl AsRef<Type<'a>>) {
        llvm::core::LLVMConstFPCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner())
    });

    instr!(select(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstSelect(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    instr!(extract_element(&self, index: &Const<'a>) {
        llvm::core::LLVMConstExtractElement(self.as_ref().llvm_inner(), index.as_ref().llvm_inner())
    });

    instr!(insert_element(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstInsertElement(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    instr!(shuffle_vector(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstShuffleVector(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    instr!(extract_value(&self, idx: impl AsRef<[usize]>) {
        let num = idx.as_ref().len();
        let mut idx: Vec<c_uint> = idx.as_ref().into_iter().map(|x| *x as c_uint).collect();
        llvm::core::LLVMConstExtractValue(self.as_ref().llvm_inner(), idx.as_mut_ptr(), num as u32)
    });

    instr!(insert_value(&self, idx: impl AsRef<[usize]>, x: &Const<'a>) {
        let num = idx.as_ref().len();
        let mut idx: Vec<c_uint> = idx.as_ref().into_iter().map(|x| *x as c_uint).collect();
        llvm::core::LLVMConstInsertValue(self.as_ref().llvm_inner(), x.as_ref().llvm_inner(), idx.as_mut_ptr(), num as u32)
    });
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let message =
            unsafe { Message::from_raw(llvm::core::LLVMPrintValueToString(self.llvm_inner())) };
        write!(fmt, "{}", message.as_ref())
    }
}

pub struct Function<'a>(pub(crate) Value<'a>);

impl<'a> AsRef<Value<'a>> for Function<'a> {
    fn as_ref(&self) -> &Value<'a> {
        &self.0
    }
}

impl<'a> From<Function<'a>> for Value<'a> {
    fn from(x: Function<'a>) -> Value<'a> {
        x.0
    }
}

impl<'a> Function<'a> {
    pub fn param_count(&self) -> usize {
        let n = unsafe { llvm::core::LLVMCountParams(self.as_ref().llvm_inner()) };
        n as usize
    }

    pub fn param(&self, i: usize) -> Result<Value<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMGetParam(
                self.as_ref().llvm_inner(),
                i as u32,
            ))
        }
    }

    pub fn params(&self) -> Vec<Value<'a>> {
        let len = self.param_count();
        let mut data = vec![std::ptr::null_mut(); len];

        unsafe { llvm::core::LLVMGetParams(self.as_ref().llvm_inner(), data.as_mut_ptr()) }
        data.into_iter()
            .map(|x| Value::from_inner(x).unwrap())
            .collect()
    }

    /// Verify the function, returning an error on failure
    pub fn verify(&self) -> Result<(), Error> {
        let ok = unsafe {
            llvm::analysis::LLVMVerifyFunction(
                self.as_ref().llvm_inner(),
                llvm::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            ) == 0
        };

        if !ok {
            return Err(Error::InvalidFunction);
        }

        Ok(())
    }

    pub fn next_function(&self) -> Result<Function<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetNextFunction(self.as_ref().llvm_inner()) };
        Value::from_inner(v).map(|x| Function(x))
    }

    pub fn prev_function(&self) -> Result<Function<'a>, Error> {
        let v = unsafe { llvm::core::LLVMGetPreviousFunction(self.as_ref().llvm_inner()) };
        Value::from_inner(v).map(|x| Function(x))
    }
}
