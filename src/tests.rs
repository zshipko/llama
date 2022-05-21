use crate::*;

#[test]
fn codegen() -> Result<(), Error> {
    let jit = Jit::new("test_codegen", Some(2))?;

    let i32 = Type::of::<i32>(jit.context())?;

    let ft = FuncType::new(i32, &[i32, i32])?;
    jit.declare_function("testing", ft, |builder, f| {
        let params = f.params();
        let a = builder.add(params[0], params[1], "a")?;
        builder.ret(&a)
    })?;

    println!("{}", jit.module());

    jit.module().verify()?;

    let testing: extern "C" fn(i32, i32) -> i32 = unsafe { jit.engine().function("testing")? };

    let x: i32 = testing(1i32, 2i32);
    assert_eq!(x, 3);

    Codegen::new(jit.module(), &["testing"], true)?;

    Ok(())
}

#[test]
fn if_then_else() -> Result<(), Error> {
    let jit = Jit::new("test_if_then_else", None)?;

    let f32 = Type::float(jit.context())?;
    let ft = FuncType::new(f32, &[f32])?;

    jit.declare_function("testing", ft, |builder, f| {
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

        let ret = builder.ret(ite);
        f.verify()?;
        ret
    })?;

    println!("{}", jit.module());

    let testing: extern "C" fn(f32) -> f32 = unsafe { jit.engine().function("testing")? };
    let x = testing(11.0);
    let y = testing(9.0);

    assert_eq!(x, 2.0);
    assert_eq!(y, 1.0);

    Codegen::new(jit.module(), &["testing"], false)?;

    Ok(())
}

#[test]
fn for_loop() -> Result<(), Error> {
    let jit = Jit::new("test_for_loop", None)?;

    let i64 = Type::int(jit.context(), 64)?;

    let ft = FuncType::new(i64, &[i64])?;
    jit.declare_function("testing", ft, |build, f| {
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

    println!("{}", jit.module());

    let testing: extern "C" fn(i64) -> i64 = unsafe { jit.engine().function("testing")? };
    let x = testing(10);

    println!("{}", x);
    assert_eq!(x, 9);

    let x = testing(100);
    assert_eq!(x, 99);

    let (_ctx, _builder, engine) = jit.into_inner();
    Codegen::new(&engine.into_module()?, &["testing"], true)?;

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
    let jit = Jit::new("test_add_symbol", None)?;

    symbol!(testing123, testing1234);

    let i32 = Type::int(jit.context(), 32)?;

    let testing123_t = FuncType::new(i32, &[])?;
    let testing1234_t = FuncType::new(i32, &[])?;
    let testing123 = jit.define_function("testing123", testing123_t)?;
    let testing1234 = jit.define_function("testing1234", testing1234_t)?;

    jit.declare_function("testing", testing123.func_type()?, |build, _| {
        build.ret(build.call(testing123, &[], "call")?)
    })?;

    jit.declare_function("testing1", testing1234.func_type()?, |build, _| {
        build.ret(build.call(testing1234, &[], "call")?)
    })?;

    let testing: extern "C" fn() -> i32 = unsafe { jit.engine().function("testing")? };
    let testing1: extern "C" fn() -> i32 = unsafe { jit.engine().function("testing1")? };
    let x = testing();
    let y = testing1();

    println!("{}", x);
    assert_eq!(x, 123);

    println!("{}", y);
    assert_eq!(y, 1234);

    Ok(())
}

#[test]
fn test_rust_struct() -> Result<(), Error> {
    struct Test {
        a: i64,
        b: i64,
    }

    #[no_mangle]
    unsafe extern "C" fn test_add(t: *mut Test) -> i64 {
        let x = &*t;
        x.a + x.b
    }

    #[no_mangle]
    unsafe extern "C" fn test_free(t: *mut Test) {
        println!("FREE");
        drop(Box::from_raw(t))
    }

    #[no_mangle]
    unsafe extern "C" fn mk_test(a: i64, b: i64) -> *mut Test {
        Box::into_raw(Box::new(Test { a, b }))
    }

    let jit = Jit::new("test_rust_struct", None)?;
    let ctx = jit.context();
    let module = jit.module();

    symbol!(test_add, mk_test, test_free);

    let i8 = Type::int(ctx, 8)?;
    let ptr = i8.pointer(None)?;

    let i64 = Type::int(ctx, 64)?;

    let mk_test = jit.define_function("mk_test", FuncType::new(ptr, &[i64, i64])?)?;
    let test_add = jit.define_function("test_add", FuncType::new(i64, &[ptr])?)?;
    let test_free = jit.define_function("test_free", FuncType::new(Type::void(ctx)?, &[ptr])?)?;

    let run_test_t = FuncType::new(i64, &[i64, i64])?;

    jit.declare_function("run_test", run_test_t, |build, f| {
        let a = f.param(0)?;
        let b = f.param(1)?;

        let x = build.call(mk_test, &[a, b], "mk_test")?;
        let y = build.call(test_add, &[x.into()], "test_add")?;
        build.call(test_free, &[x.into()], "free")?;

        build.ret(y)
    })?;

    println!("{}", module);

    let run_test: extern "C" fn(i64, i64) -> i64 = unsafe { jit.engine().function("run_test")? };

    let x = run_test(10, 20);
    assert_eq!(30, x);

    Ok(())
}
