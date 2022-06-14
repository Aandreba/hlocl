use opencl::{prelude::*, buffer::{MemFlag, FastRng}};

#[test]
fn vec () -> Result<()> {
    let rng = FastRng::with_context(Context::default())?;
    let buff = rng.random_u8_with_queue(CommandQueue::default(), 10, MemFlag::default(), EMPTY)?;
    let buff = buff.wait()?;

    println!("{buff:?}");
    Ok(())
}