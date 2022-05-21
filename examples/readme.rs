use llama::*;

// Convenience type alias for the `sum` function.
//
// Calling this is innately `unsafe` because there's no guarantee it doesn't
// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

fn compile_sum(jit: &mut Jit) -> Result<SumFunc, Error> {
    let i64 = Type::i64(jit.context())?;
    let sum_t = FuncType::new(i64, [i64, i64, i64])?;
    jit.declare_function("sum", sum_t, |build, f| {
        let params = f.params();
        let x = params[0];
        let y = params[1];
        let z = params[2];

        let sum = build.add(x, y, "sum")?;
        let sum = build.add(sum, z, "sum")?;
        build.ret(sum)
    })?;

    unsafe { jit.engine().function("sum") }
}

fn main() -> Result<(), Error> {
    let mut jit = Jit::new("sum", None)?;

    let sum = compile_sum(&mut jit)?;

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum(x, y, z));
        assert_eq!(sum(x, y, z), x + y + z);
    }

    Ok(())
}
