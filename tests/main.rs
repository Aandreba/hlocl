use opencl::{prelude::*, buffer::{MemFlag, FastRng}};

#[test]
fn vec () -> Result<()> {
    let rng = FastRng::with_context(Context::default(), 10)?;
    let buff = rng.random_u8_with_queue(CommandQueue::default(), 5, MemFlag::default(), EMPTY)?.wait()?;

    println!("{buff:?}");
    Ok(())
} 