use crate::*;

#[test]
fn codegen() -> Result<(), Error> {
    let context = Context::new()?;
    let module = Module::new(&context, "test_codegen")?;

    let builder = Builder::new(&context)?;

    let i32 = Type::int(&context, 32)?;

    let ft = FuncType::new(i32, &[i32, i32])?;
    module.declare_function(&builder, "testing", ft, |f| {
        let params = f.params();
        let a = builder.add(params[0], params[1], "a")?;
        builder.ret(&a)
    })?;

    println!("{}", module);

    let engine = ExecutionEngine::new_jit(module.clone(), 2)?;

    let testing: extern "C" fn(i32, i32) -> i32 = unsafe { engine.function("testing")? };

    let x: i32 = testing(1i32, 2i32);
    assert_eq!(x, 3);

    Codegen::new(&module, &["testing"], true)?;

    Ok(())
}

#[test]
fn if_then_else() -> Result<(), Error> {
    let ctx = Context::new()?;
    let module = Module::new(&ctx, "test_if_then_else")?;
    let builder = Builder::new(&ctx)?;

    let f32 = Type::float(&ctx)?;
    let ft = FuncType::new(f32, &[f32])?;
    module.declare_function(&builder, "testing", ft, |f| {
        let params = f.params();
        let cond = builder.fcmp(
            Fcmp::LLVMRealULT,
            &params[0],
            Const::real(f32, 10.0)?,
            "cond",
        )?;
        let a = Const::real(f32, 1.0)?;
        let b = Const::real(f32, 2.0)?;
        let ite = builder.if_then_else(cond, |_| Ok(a), |_| Ok(b))?;
        builder.ret(ite)
    })?;

    println!("{}", module);

    {
        let engine = ExecutionEngine::new(module.clone())?;
        let testing: extern "C" fn(f32) -> f32 = unsafe { engine.function("testing")? };
        let x = testing(11.0);
        let y = testing(9.0);

        assert_eq!(x, 2.0);
        assert_eq!(y, 1.0);
    }

    Codegen::new(&module, &["testing"], false)?;

    Ok(())
}

#[test]
fn for_loop() -> Result<(), Error> {
    let ctx = Context::new()?;
    let module = Module::new(&ctx, "test_for_loop")?;
    let build = Builder::new(&ctx)?;

    let i64 = Type::int(&ctx, 64)?;

    let ft = FuncType::new(i64, &[i64])?;
    module.declare_function(&build, "testing", ft, |f| {
        let params = f.params();
        let one = Const::int_sext(i64, 1)?;
        let f = build.for_loop(
            Const::int_sext(i64, 0)?,
            |x| build.icmp(Icmp::LLVMIntSLT, x, params[0], "cond"),
            |x| build.add(x, one, "add"),
            |x| Ok(*x),
        )?;
        build.ret(f)
    })?;

    println!("{}", module);

    {
        let engine = ExecutionEngine::new(module.clone())?;
        let testing: extern "C" fn(i64) -> i64 = unsafe { engine.function("testing")? };
        let x = testing(10);

        println!("{}", x);
        assert_eq!(x, 9);

        let x = testing(100);
        assert_eq!(x, 99);
    }

    Codegen::new(&module, &["testing"], true)?;

    Ok(())
}

extern "C" fn testing123() -> i32 {
    123
}

extern "C" fn testing1234() -> i32 {
    1234
}

#[test]
fn test_add_symbol() -> Result<(), Error> {
    let ctx = Context::new()?;
    let module = Module::new(&ctx, "test_add_symbol")?;
    let build = Builder::new(&ctx)?;

    symbol!(testing123; testing1234);

    let i32 = Type::int(&ctx, 32)?;

    let testing123 = symbol!(module.fn testing123() -> i32)?;
    let testing1234 = symbol!(module.fn testing1234() -> i32)?;

    module.declare_function(&build, "testing", testing123.func_type()?, |_| {
        build.ret(build.call(testing123, &[], "call")?)
    })?;

    module.declare_function(&build, "testing1", testing1234.func_type()?, |_| {
        build.ret(build.call(testing1234, &[], "call")?)
    })?;

    let engine = ExecutionEngine::new(module)?;
    let testing: extern "C" fn() -> i32 = unsafe { engine.function("testing")? };
    let testing1: extern "C" fn() -> i32 = unsafe { engine.function("testing1")? };
    let x = testing();
    let y = testing1();

    println!("{}", x);
    assert_eq!(x, 123);

    println!("{}", y);
    assert_eq!(y, 1234);

    for module in engine.modules().iter() {
        println!("{}", module);
    }

    Ok(())
}
