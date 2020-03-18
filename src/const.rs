use crate::*;

pub struct Const<'a>(pub(crate) Value<'a>);
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

impl<'a> Const<'a> {
    const_func!(int(t: impl AsRef<Type<'a>>, i: i64, sign_extend: bool) {
        llvm::core::LLVMConstInt(
            t.as_ref().llvm_inner(),
            i as u64,
            sign_extend as c_int,
        )
    });

    const_func!(int_s(
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<str>,
        radix: u8,
    ) {
        let s = cstr!(i.as_ref());
        llvm::core::LLVMConstIntOfString(
            t.as_ref().llvm_inner(),
            s.as_ptr(),
            radix,
        )
    });

    const_func!(real(t: impl AsRef<Type<'a>>, i: f64) {
        llvm::core::LLVMConstReal(t.as_ref().llvm_inner(), i as f64)
    });

    const_func!(real_s(
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<str>,
    ) {
        let s = cstr!(i.as_ref());
        llvm::core::LLVMConstRealOfString(
            t.as_ref().llvm_inner(),
            s.as_ptr(),
        )
    });

    const_func!(undef(t: impl AsRef<Type<'a>>)  {
        llvm::core::LLVMGetUndef(t.as_ref().llvm_inner())
    });

    const_func!(pointer_null(t: impl AsRef<Type<'a>>){
        llvm::core::LLVMConstPointerNull(t.as_ref().llvm_inner())
    });

    const_func!(null(t: impl AsRef<Type<'a>>){
        llvm::core::LLVMConstNull(t.as_ref().llvm_inner())
    });

    const_func!(all_ones(t: impl AsRef<Type<'a>>){
        llvm::core::LLVMConstAllOnes(t.as_ref().llvm_inner())
    });

    const_func!(string(
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

    const_func!(get_element(&self, index: usize) {
            llvm::core::LLVMGetElementAsConstant(self.as_ref().llvm_inner(), index as c_uint)
    });

    pub fn crate_struct(
        ctx: &'a Context,
        vals: impl AsRef<[&'a Value<'a>]>,
        packed: bool,
    ) -> Result<Const<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstStructInContext(
                ctx.llvm_inner(),
                vals.as_mut_ptr(),
                len as c_uint,
                if packed { 1 } else { 0 },
            )
        };
        Value::from_inner(v)?.into_const()
    }

    pub fn create_named_struct(
        t: impl AsRef<Type<'a>>,
        vals: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Const<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstNamedStruct(
                t.as_ref().llvm_inner(),
                vals.as_mut_ptr(),
                len as c_uint,
            )
        };
        Value::from_inner(v)?.into_const()
    }

    pub fn create_array(
        t: impl AsRef<Type<'a>>,
        vals: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Const<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe {
            llvm::core::LLVMConstArray(t.as_ref().llvm_inner(), vals.as_mut_ptr(), len as c_uint)
        };
        Value::from_inner(v)?.into_const()
    }

    pub fn create_vector(vals: impl AsRef<[&'a Value<'a>]>) -> Result<Const<'a>, Error> {
        let len = vals.as_ref().len();
        let mut vals: Vec<*mut llvm::LLVMValue> =
            vals.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let v = unsafe { llvm::core::LLVMConstVector(vals.as_mut_ptr(), len as c_uint) };
        Value::from_inner(v)?.into_const()
    }

    pub fn opcode(&self) -> OpCode {
        unsafe { llvm::core::LLVMGetConstOpcode(self.as_ref().llvm_inner()) }
    }

    pub fn neg(&self) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNeg(self.as_ref().llvm_inner()))?.into_const()
        }
    }

    pub fn nsw_neg(&self) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWNeg(self.as_ref().llvm_inner()))?.into_const()
        }
    }

    pub fn nuw_neg(&self) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWNeg(self.as_ref().llvm_inner()))?.into_const()
        }
    }

    pub fn fneg(&self) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFNeg(self.as_ref().llvm_inner()))?.into_const()
        }
    }

    pub fn not(&self) -> Result<Const<'a>, Error> {
        unsafe { Value::from_inner(llvm::core::LLVMConstNot(self.as_ref().llvm_inner())) }?
            .into_const()
    }

    pub fn add(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nsw_add(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nuw_add(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fadd(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFAdd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn sub(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nsw_sub(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nuw_sub(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fsub(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFSub(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn mul(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nsw_mul(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNSWMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn nuw_mul(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstNUWMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fmul(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFMul(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn udiv(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstUDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn exact_udiv(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstExactUDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn sdiv(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn exact_sdiv(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstExactSDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fdiv(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFDiv(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn urem(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstURem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn srem(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSRem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn frem(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFRem(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn and(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAnd(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn or(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstOr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn xor(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstXor(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn icmp(&self, pred: ICmp, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstICmp(
                pred,
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fcmp(&self, pred: FCmp, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFCmp(
                pred,
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn shl(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstShl(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn lshr(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstLShr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn ashr(&self, other: &Const<'a>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAShr(
                self.as_ref().llvm_inner(),
                other.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn gep(&self, i: impl AsRef<[&'a Value<'a>]>) -> Result<Const<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> = i.as_ref().iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstGEP(
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))?
            .into_const()
        }
    }

    pub fn gep2(
        &self,
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Const<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> = i.as_ref().iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstGEP2(
                t.as_ref().llvm_inner(),
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))?
            .into_const()
        }
    }

    pub fn in_bounds_gep(&self, i: impl AsRef<[&'a Value<'a>]>) -> Result<Const<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> = i.as_ref().iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstInBoundsGEP(
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))?
            .into_const()
        }
    }

    pub fn in_bounds_gep2(
        &self,
        t: impl AsRef<Type<'a>>,
        i: impl AsRef<[&'a Value<'a>]>,
    ) -> Result<Const<'a>, Error> {
        let len = i.as_ref().len();
        let mut i: Vec<*mut llvm::LLVMValue> = i.as_ref().iter().map(|x| x.llvm_inner()).collect();
        unsafe {
            Value::from_inner(llvm::core::LLVMConstInBoundsGEP2(
                t.as_ref().llvm_inner(),
                self.as_ref().llvm_inner(),
                i.as_mut_ptr(),
                len as c_uint,
            ))?
            .into_const()
        }
    }

    pub fn trunc(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstTrunc(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn sext(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn zext(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstZExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fp_trunc(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPTrunc(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fp_ext(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPExt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn ui_to_fp(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstUIToFP(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn si_to_fp(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSIToFP(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fp_to_ui(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPToUI(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn fp_to_si(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstFPToSI(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn ptr_to_int(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstPtrToInt(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn int_to_ptr(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstIntToPtr(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn addr_space_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstAddrSpaceCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn zext_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstZExtOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn sext_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstSExtOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    pub fn trunc_or_bit_cast(&self, t: impl AsRef<Type<'a>>) -> Result<Const<'a>, Error> {
        unsafe {
            Value::from_inner(llvm::core::LLVMConstTruncOrBitCast(
                self.as_ref().llvm_inner(),
                t.as_ref().llvm_inner(),
            ))?
            .into_const()
        }
    }

    const_func!(pointer_cast(&self, t: impl AsRef<Type<'a>>) {
        llvm::core::LLVMConstPointerCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner())
    });

    const_func!(int_cast(&self, t: impl AsRef<Type<'a>>, signed: bool) {
        llvm::core::LLVMConstIntCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner(), if signed { 1 } else { 0 })
    });

    const_func!(fp_cast(&self, t: impl AsRef<Type<'a>>) {
        llvm::core::LLVMConstFPCast(self.as_ref().llvm_inner(), t.as_ref().llvm_inner())
    });

    const_func!(select(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstSelect(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    const_func!(extract_element(&self, index: &Const<'a>) {
        llvm::core::LLVMConstExtractElement(self.as_ref().llvm_inner(), index.as_ref().llvm_inner())
    });

    const_func!(insert_element(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstInsertElement(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    const_func!(shuffle_vector(&self, a: &Const<'a>, b: &Const<'a>) {
        llvm::core::LLVMConstShuffleVector(self.as_ref().llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner())
    });

    const_func!(extract_value(&self, idx: impl AsRef<[usize]>) {
        let num = idx.as_ref().len();
        let mut idx: Vec<c_uint> = idx.as_ref().iter().map(|x| *x as c_uint).collect();
        llvm::core::LLVMConstExtractValue(self.as_ref().llvm_inner(), idx.as_mut_ptr(), num as u32)
    });

    const_func!(insert_value(&self, idx: impl AsRef<[usize]>, x: &Const<'a>) {
        let num = idx.as_ref().len();
        let mut idx: Vec<c_uint> = idx.as_ref().iter().map(|x| *x as c_uint).collect();
        llvm::core::LLVMConstInsertValue(self.as_ref().llvm_inner(), x.as_ref().llvm_inner(), idx.as_mut_ptr(), num as u32)
    });
}
