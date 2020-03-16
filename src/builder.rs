use crate::*;

pub struct Builder<'a>(NonNull<llvm::LLVMBuilder>, &'a Context<'a>);

llvm_inner_impl!(Builder<'a>, llvm::LLVMBuilder);

impl<'a> Drop for Builder<'a> {
    fn drop(&mut self) {
        unsafe { llvm::core::LLVMDisposeBuilder(self.llvm_inner()) }
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

    instr!(ret(&self, a: impl AsRef<Value<'a>>) {
        llvm::core::LLVMBuildRet(
            self.llvm_inner(),
            a.as_ref().llvm_inner()
        )
    });

    pub fn add(
        &self,
        a: impl AsRef<Value<'a>>,
        b: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe {
            Value::from_inner(llvm::core::LLVMBuildAdd(
                self.llvm_inner(),
                a.as_ref().llvm_inner(),
                b.as_ref().llvm_inner(),
                name.as_ptr(),
            ))
        }
    }

    pub fn sub(
        &self,
        a: impl AsRef<Value<'a>>,
        b: impl AsRef<Value<'a>>,
        name: impl AsRef<str>,
    ) -> Result<Value<'a>, Error> {
        let name = cstr!(name.as_ref());
        unsafe {
            Value::from_inner(llvm::core::LLVMBuildSub(
                self.llvm_inner(),
                a.as_ref().llvm_inner(),
                b.as_ref().llvm_inner(),
                name.as_ptr(),
            ))
        }
    }
}
