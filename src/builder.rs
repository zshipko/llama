use crate::*;

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
    pub fn new(ctx: &'a Context) -> Result<Builder<'a>, Error> {
        let b = unsafe { wrap_inner(llvm::core::LLVMCreateBuilderInContext(ctx.llvm_inner()))? };
        Ok(Builder(b, ctx))
    }

    pub fn context(&self) -> &'a Context<'a> {
        self.1
    }

    pub fn position_at_end(&self, block: &BasicBlock<'a>) {
        unsafe { llvm::core::LLVMPositionBuilderAtEnd(self.llvm_inner(), block.llvm_inner()) }
    }

    pub fn position_before(&self, value: &Value<'a>) {
        unsafe { llvm::core::LLVMPositionBuilderBefore(self.llvm_inner(), value.llvm_inner()) }
    }

    pub fn define_function<F: FnOnce(&Self, BasicBlock<'a>) -> Result<Value<'a>, Error>>(
        &self,
        f: &Function<'a>,
        def: F,
    ) -> Result<Value, Error> {
        let entry = BasicBlock::append(self.context(), f.as_ref(), "entry")?;
        self.position_at_end(&entry);
        let v = def(self, entry)?;
        f.verify()?;
        Ok(v)
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
        let mut values: Vec<*mut llvm::LLVMValue> = values.into_iter().map(|x| x.llvm_inner()).collect();
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

    // Next: BuildSwitch

    op!(2: add, LLVMBuildAdd);
    op!(2: fadd, LLVMBuildFAdd);
    op!(2: sub, LLVMBuildSub);
    op!(2: fsub, LLVMBuildFSub);
    op!(2: mul, LLVMBuildMul);
    op!(2: fmul, LLVMBuildFMul);
}
