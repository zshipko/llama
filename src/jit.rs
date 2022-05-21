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
}
