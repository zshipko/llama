use crate::*;

/// Jit bundles LLVMContext, LLVMBuilder and LLVMExecutionEngine
pub struct Jit<'ctx> {
    engine: ExecutionEngine<'ctx>,
    build: Builder<'ctx>,
    context: Context<'ctx>,
}

impl<'ctx> Jit<'ctx> {
    /// Create new Jit instance
    pub fn new(name: impl AsRef<str>, opt: Option<usize>) -> Result<Jit<'ctx>, Error> {
        let context = Context::new()?;
        let module = Module::new(&context, name)?;
        let build = Builder::new(&context)?;
        let engine = ExecutionEngine::new_jit(module, opt.unwrap_or_default())?;
        Ok(Jit {
            context,
            build,
            engine,
        })
    }

    /// Destruct
    pub fn into_inner(self) -> (Context<'ctx>, Builder<'ctx>, ExecutionEngine<'ctx>) {
        (self.context, self.build, self.engine)
    }

    /// Access ExecutionEngine
    pub fn engine(&self) -> &ExecutionEngine<'ctx> {
        &self.engine
    }

    /// Mutable access to ExecutionEngine
    pub fn engine_mut(&mut self) -> &mut ExecutionEngine<'ctx> {
        &mut self.engine
    }

    /// Access Builder
    pub fn build(&self) -> &Builder<'ctx> {
        &self.build
    }

    /// Mutable access to Builder
    pub fn build_mut(&mut self) -> &mut Builder<'ctx> {
        &mut self.build
    }

    /// Access Context
    pub fn context(&self) -> &Context<'ctx> {
        &self.context
    }

    /// Mutable access to Context
    pub fn context_mut(&mut self) -> &mut Context<'ctx> {
        &mut self.context
    }

    /// Access Module
    pub fn module(&self) -> &Module<'ctx> {
        self.engine.module()
    }

    /// Mutable access to Module
    pub fn module_mut(&mut self) -> &mut Module<'ctx> {
        self.engine.module_mut()
    }

    /// Declare a new function with function body
    pub fn declare_function<
        T: Into<Value<'ctx>>,
        F: FnOnce(&Builder<'ctx>, Func<'ctx>) -> Result<T, Error>,
    >(
        &self,
        name: impl AsRef<str>,
        ft: FuncType,
        def: F,
    ) -> Result<Func<'ctx>, Error> {
        self.module()
            .declare_function(&self.build, name, ft, |x| def(&self.build, x))
    }

    /// Define a new function without declaring a function body
    pub fn define_function(&self, name: impl AsRef<str>, t: FuncType) -> Result<Func<'ctx>, Error> {
        self.module().define_function(name, t)
    }

    /// Create a new global with an empty initializer
    pub fn define_global(
        &self,
        name: impl AsRef<str>,
        ty: impl AsRef<Type<'ctx>>,
    ) -> Result<Value<'ctx>, Error> {
        self.module().define_global(name, ty)
    }

    /// Create a new global with the specified initializer
    pub fn declare_global(
        &self,
        name: impl AsRef<str>,
        t: impl AsRef<Value<'ctx>>,
    ) -> Result<Value<'ctx>, Error> {
        self.module().declare_global(name, t)
    }
}
