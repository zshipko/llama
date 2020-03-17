use crate::*;

/// A `Builder` is used to create `Instruction`s
pub struct Builder<'a>(NonNull<llvm::LLVMBuilder>, &'a Context<'a>);

llvm_inner_impl!(Builder<'a>, llvm::LLVMBuilder);

impl<'a> Drop for Builder<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeBuilder(self.llvm_inner()) }
    }
}

macro_rules! op {
    (3 : $name:ident, $f:ident) => {
        instr!($name(&self, a: impl AsRef<Value<'a>>, b: impl AsRef<Value<'a>>, c: impl AsRef<Value<'a>>, name: impl AsRef<str>) {
            let name = cstr!(name.as_ref());
            llvm::core::$f(self.llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner(), c.as_ref().llvm_inner(), name.as_ptr())
        });
    };
    (2 : $name:ident, $f:ident) => {
        instr!($name(&self, a: impl AsRef<Value<'a>>, b: impl AsRef<Value<'a>>, name: impl AsRef<str>) {
            let name = cstr!(name.as_ref());
            llvm::core::$f(self.llvm_inner(), a.as_ref().llvm_inner(), b.as_ref().llvm_inner(), name.as_ptr())
        });
    };
    (1 : $name:ident, $f:ident) => {
        instr!($name(&self, a: impl AsRef<Value<'a>>, name: impl AsRef<str>) {
            let name = cstr!(name.as_ref());
            llvm::core::$f(self.llvm_inner(), a.as_ref().llvm_inner(), name.as_ptr())
        });
    };
    (0 : $name:ident, $f:ident) => {
        instr!($name(&self, a: impl AsRef<Value<'a>>, name: impl AsRef<str>) {
            let name = cstr!(name.as_ref());
            llvm::core::$f(self.llvm_inner(), name.as_ptr())
        });
    }

}

impl<'a> Builder<'a> {
    /// Create a new builder
    pub fn new(ctx: &'a Context) -> Result<Builder<'a>, Error> {
        let b = unsafe { wrap_inner(llvm::core::LLVMCreateBuilderInContext(ctx.llvm_inner()))? };
        Ok(Builder(b, ctx))
    }

    /// Get the builder's context
    pub fn context(&self) -> &'a Context<'a> {
        self.1
    }

    pub fn position_at_end(&self, block: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMPositionBuilderAtEnd(self.llvm_inner(), block.llvm_inner()) }
    }

    pub fn position_before(&self, value: &Value<'a>) {
        unsafe { llvm::core::LLVMPositionBuilderBefore(self.llvm_inner(), value.llvm_inner()) }
    }

    /// Clear insertion position
    pub fn clear_insertion_position(&self) {
        unsafe { llvm::core::LLVMClearInsertionPosition(self.llvm_inner()) }
    }

    /// Get the insertion block
    pub fn insertion_block(&self) -> Result<BasicBlock<'a>, Error> {
        unsafe { BasicBlock::from_inner(llvm::core::LLVMGetInsertBlock(self.llvm_inner())) }
    }

    pub fn define_function<
        T: Into<Value<'a>>,
        F: FnOnce(&Self, BasicBlock<'a>) -> Result<T, Error>,
    >(
        &self,
        f: &Function<'a>,
        def: F,
    ) -> Result<Instruction<'a>, Error> {
        let entry = BasicBlock::append(self.context(), f.as_ref(), "entry")?;
        self.position_at_end(&entry);
        let v = def(self, entry)?;
        f.verify()?;
        Ok(Instruction(v.into()))
    }

    instr!(ret_void(&self) {
        llvm::core::LLVMBuildRetVoid(
            self.llvm_inner(),
        )
    });

    instr!(ret(&self, a: impl AsRef<Value<'a>>) {
        llvm::core::LLVMBuildRet(
            self.llvm_inner(),
            a.as_ref().llvm_inner()
        )
    });

    instr!(aggregate_ret(&self, vals: impl AsRef<[&'a Value<'a>]>) {
        let values = vals.as_ref();
        let mut values: Vec<*mut llvm::LLVMValue> = values.iter().map(|x| x.llvm_inner()).collect();
        let ptr = values.as_mut_ptr();
        let len = values.len();
        llvm::core::LLVMBuildAggregateRet(self.llvm_inner(), ptr, len as u32)
    });

    instr!(br(&self, bb: &BasicBlock<'a>) {
        llvm::core::LLVMBuildBr(self.llvm_inner(), bb.llvm_inner())
    });

    instr!(cond_br(&self, if_: impl AsRef<Value<'a>>,  then_: &BasicBlock<'a>, else_: &BasicBlock<'a>) {
        llvm::core::LLVMBuildCondBr(self.llvm_inner(), if_.as_ref().llvm_inner(), then_.llvm_inner(), else_.llvm_inner())
    });

    instr!(switch(&self, v: impl AsRef<Value<'a>>, bb: &BasicBlock<'a>, num_cases: usize) {
        llvm::core::LLVMBuildSwitch(self.llvm_inner(), v.as_ref().llvm_inner(), bb.llvm_inner(), num_cases as c_uint)
    });

    instr!(indirect_br(&self, addr: impl AsRef<Value<'a>>, num_dests: usize) {
        llvm::core::LLVMBuildIndirectBr(self.llvm_inner(), addr.as_ref().llvm_inner(), num_dests as c_uint)
    });

    instr!(invoke(&self, f: impl AsRef<Value<'a>>, args: impl AsRef<[&'a Value<'a>]>, then: &BasicBlock<'a>, catch: &BasicBlock<'a>, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        let mut args: Vec<*mut llvm::LLVMValue> = args.as_ref().iter().map(|x| x.llvm_inner()).collect();
        llvm::core::LLVMBuildInvoke(self.llvm_inner(), f.as_ref().llvm_inner(), args.as_mut_ptr(), args.len() as c_uint, then.llvm_inner(), catch.llvm_inner(), name.as_ptr())
    });

    instr!(invoke2(&self, t: impl AsRef<Type<'a>>, f: impl AsRef<Value<'a>>, args: impl AsRef<[&'a Value<'a>]>, then: &BasicBlock<'a>, catch: &BasicBlock<'a>, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        let mut args: Vec<*mut llvm::LLVMValue> = args.as_ref().iter().map(|x| x.llvm_inner()).collect();
        llvm::core::LLVMBuildInvoke2(self.llvm_inner(), t.as_ref().llvm_inner(), f.as_ref().llvm_inner(), args.as_mut_ptr(), args.len() as c_uint, then.llvm_inner(), catch.llvm_inner(), name.as_ptr())
    });

    instr!(unreachable(&self) {
        llvm::core::LLVMBuildUnreachable(self.llvm_inner())
    });

    instr!(resume(&self, exn: impl AsRef<Value<'a>>) {
        llvm::core::LLVMBuildResume(self.llvm_inner(), exn.as_ref().llvm_inner())
    });

    op!(2: add, LLVMBuildAdd);
    op!(2: nsw_add, LLVMBuildNSWAdd);
    op!(2: nuw_add, LLVMBuildNUWAdd);
    op!(2: fadd, LLVMBuildFAdd);
    op!(2: sub, LLVMBuildSub);
    op!(2: nsw_sub, LLVMBuildNSWSub);
    op!(2: nuw_sub, LLVMBuildNUWSub);
    op!(2: fsub, LLVMBuildFSub);
    op!(2: mul, LLVMBuildMul);
    op!(2: nsw_mul, LLVMBuildNSWMul);
    op!(2: nuw_mul, LLVMBuildNUWMul);
    op!(2: fmul, LLVMBuildFMul);
    op!(2: udiv, LLVMBuildUDiv);
    op!(2: exact_udiv, LLVMBuildExactUDiv);
    op!(2: sdiv, LLVMBuildSDiv);
    op!(2: exact_sdiv, LLVMBuildExactSDiv);
    op!(2: fdiv, LLVMBuildFDiv);
    op!(2: urem, LLVMBuildURem);
    op!(2: srem, LLVMBuildSRem);
    op!(2: frem, LLVMBuildFRem);
    op!(2: shl, LLVMBuildShl);
    op!(2: lshr, LLVMBuildLShr);
    op!(2: ashr, LLVMBuildAShr);
    op!(2: and, LLVMBuildAnd);
    op!(2: or, LLVMBuildOr);
    op!(2: xor, LLVMBuildXor);

    pub fn bin_op(
        &self,
        op: OpCode,
        lhs: impl AsRef<Value<'a>>,
        rhs: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) -> Result<Instruction<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe {
            Ok(Instruction(Value::from_inner(llvm::core::LLVMBuildBinOp(
                self.llvm_inner(),
                op,
                lhs.as_ref().llvm_inner(),
                rhs.as_ref().llvm_inner(),
                name.as_ptr(),
            ))?))
        }
    }

    op!(1: neg, LLVMBuildNeg);
    op!(1: nsw_neg, LLVMBuildNSWNeg);
    op!(1: nuw_neg, LLVMBuildNUWNeg);
    op!(1: fneg, LLVMBuildFNeg);
    op!(1: not, LLVMBuildNot);

    instr!(malloc(
        &self,
        t: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildMalloc(
            self.llvm_inner(),
            t.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(array_malloc(
        &self,
        t: impl AsRef<Type<'a>>,
        v: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ){
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildArrayMalloc(
            self.llvm_inner(),
            t.as_ref().llvm_inner(),
            v.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(memset(
        &self,
        ptr: impl AsRef<Value<'a>>,
        val: impl AsRef<Value<'a>>,
        len: impl AsRef<Value<'a>>,
        align: usize,
    ) {
        llvm::core::LLVMBuildMemSet(
            self.llvm_inner(),
            ptr.as_ref().llvm_inner(),
            val.as_ref().llvm_inner(),
            len.as_ref().llvm_inner(),
            align as c_uint,
        )
    });

    instr!(memcpy(
        &self,
        dst: impl AsRef<Value<'a>>,
        dst_align: usize,
        src: impl AsRef<Value<'a>>,
        src_align: usize,
        len: impl AsRef<Value<'a>>,
    ) {
        llvm::core::LLVMBuildMemCpy(
            self.llvm_inner(),
            dst.as_ref().llvm_inner(),
            dst_align as c_uint,
            src.as_ref().llvm_inner(),
            src_align as c_uint,
            len.as_ref().llvm_inner(),
        )
    });

    instr!(memmove(
        &self,
        dst: impl AsRef<Value<'a>>,
        dst_align: usize,
        src: impl AsRef<Value<'a>>,
        src_align: usize,
        len: impl AsRef<Value<'a>>,
    ) {
        llvm::core::LLVMBuildMemMove(
            self.llvm_inner(),
            dst.as_ref().llvm_inner(),
            dst_align as c_uint,
            src.as_ref().llvm_inner(),
            src_align as c_uint,
            len.as_ref().llvm_inner(),
        )
    });

    instr!(alloca(&self, t: impl AsRef<Type<'a>>, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildAlloca(
            self.llvm_inner(),
            t.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(array_alloca(
        &self,
        t: impl AsRef<Type<'a>>,
        v: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildArrayAlloca(
            self.llvm_inner(),
            t.as_ref().llvm_inner(),
            v.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(free(&self, val: impl AsRef<Value<'a>>) {
        llvm::core::LLVMBuildFree(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
        )
    });

    op!(1: load, LLVMBuildLoad);

    instr!(load2(&self, t: impl AsRef<Type<'a>>, v: impl AsRef<Value<'a>>, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildLoad2(
            self.llvm_inner(),
            t.as_ref().llvm_inner(),
            v.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(store(&self, val: impl AsRef<Value<'a>>, ptr: impl AsRef<Value<'a>>) {
        llvm::core::LLVMBuildStore(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ptr.as_ref().llvm_inner(),
        )
    });

    instr!(gep(
        &self,
        ptr: impl AsRef<Value<'a>>,
        indices: impl AsRef<[&'a Value<'a>]>,
        name: impl AsRef<str>,
    ) {
        let mut v: Vec<*mut llvm::LLVMValue> =
            indices.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let len = indices.as_ref().len();
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildGEP(
            self.llvm_inner(),
            ptr.as_ref().llvm_inner(),
            v.as_mut_ptr(),
            len as c_uint,
            name.as_ptr(),
        )
    });

    instr!(in_bounds_gep(
        &self,
        ptr: impl AsRef<Value<'a>>,
        indices: impl AsRef<[&'a Value<'a>]>,
        name: impl AsRef<str>,
    ) {
        let mut v: Vec<*mut llvm::LLVMValue> =
            indices.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let len = indices.as_ref().len();
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildInBoundsGEP(
            self.llvm_inner(),
            ptr.as_ref().llvm_inner(),
            v.as_mut_ptr(),
            len as c_uint,
            name.as_ptr(),
        )
    });

    instr!(in_struct_gep(&self, ptr: impl AsRef<Value<'a>>, index: usize, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildStructGEP(
            self.llvm_inner(),
            ptr.as_ref().llvm_inner(),
            index as c_uint,
            name.as_ptr(),
        )
    });

    instr!(gep2(
        &self,
        ty: impl AsRef<Type<'a>>,
        ptr: impl AsRef<Value<'a>>,
        indices: impl AsRef<[&'a Value<'a>]>,
        name: impl AsRef<str>,
    ) {
        let mut v: Vec<*mut llvm::LLVMValue> =
            indices.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let len = indices.as_ref().len();
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildGEP2(
            self.llvm_inner(),
            ty.as_ref().llvm_inner(),
            ptr.as_ref().llvm_inner(),
            v.as_mut_ptr(),
            len as c_uint,
            name.as_ptr(),
        )
    });

    instr!(in_bounds_gep2(
        &self,
        ty: impl AsRef<Type<'a>>,
        ptr: impl AsRef<Value<'a>>,
        indices: impl AsRef<[&'a Value<'a>]>,
        name: impl AsRef<str>,
    ) {
        let mut v: Vec<*mut llvm::LLVMValue> =
            indices.as_ref().iter().map(|x| x.llvm_inner()).collect();
        let len = indices.as_ref().len();
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildInBoundsGEP2(
            self.llvm_inner(),
            ty.as_ref().llvm_inner(),
            ptr.as_ref().llvm_inner(),
            v.as_mut_ptr(),
            len as c_uint,
            name.as_ptr(),
        )
    });

    instr!(in_struct_gep2(
        &self,
        ty: impl AsRef<Type<'a>>,
        ptr: impl AsRef<Value<'a>>,
        index: usize,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildStructGEP2(
            self.llvm_inner(),
            ty.as_ref().llvm_inner(),
            ptr.as_ref().llvm_inner(),
            index as c_uint,
            name.as_ptr(),
        )
    });

    instr!(global_string(&self, s: impl AsRef<str>, name: impl AsRef<str>) {
        let s = cstr!(s.as_ref());
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildGlobalString(
            self.llvm_inner(),
            s.as_ptr(),
            name.as_ptr(),
        )
    });

    instr!(global_string_ptr(&self, s: impl AsRef<str>, name: impl AsRef<str>) {
        let s = cstr!(s.as_ref());
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildGlobalStringPtr(
            self.llvm_inner(),
            s.as_ptr(),
            name.as_ptr(),
        )
    });

    instr!(trunc(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildTrunc(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(zext(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildZExt(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(sext(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildSExt(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(fp_to_ui(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFPToUI(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(fp_to_si(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFPToSI(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(ui_to_fp(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildUIToFP(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(si_to_fp(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildSIToFP(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(fp_trunc(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFPTrunc(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(fp_ext(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFPExt(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(ptr_to_int(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildPtrToInt(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(int_to_ptr(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildIntToPtr(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(bit_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildBitCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(addr_space_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildAddrSpaceCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(zext_or_bit_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildZExtOrBitCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(sext_or_bit_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildSExtOrBitCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(trunc_or_bit_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildTruncOrBitCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(pointer_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildPointerCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(int_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
        signed: bool,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildIntCast2(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            if signed { 1 } else { 0 },
            name.as_ptr(),
        )
    });

    instr!(fp_cast(
        &self,
        val: impl AsRef<Value<'a>>,
        ty: impl AsRef<Type<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFPCast(
            self.llvm_inner(),
            val.as_ref().llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(icmp(
        &self,
        op: ICmp,
        lhs: impl AsRef<Value<'a>>,
        rhs: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildICmp(
            self.llvm_inner(),
            op,
            lhs.as_ref().llvm_inner(),
            rhs.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(fcmp(
        &self,
        op: FCmp,
        lhs: impl AsRef<Value<'a>>,
        rhs: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildFCmp(
            self.llvm_inner(),
            op,
            lhs.as_ref().llvm_inner(),
            rhs.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });

    instr!(phi(&self, ty: impl AsRef<Type<'a>>, name: impl AsRef<str>) {
        let name = cstr!(name.as_ref());
        llvm::core::LLVMBuildPhi(
            self.llvm_inner(),
            ty.as_ref().llvm_inner(),
            name.as_ptr(),
        )
    });
}
