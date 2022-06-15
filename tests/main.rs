use opencl::{prelude::*, buffer::{MemFlag, FastRng}};

#[test]
fn vec () -> Result<()> {
    let rng = FastRng::with_context(Context::default(), 10)?;
    let u = rng.random_f32_with_queue(CommandQueue::default(), 5, MemFlag::default(), EMPTY)?.wait()?;
    let s = rng.random_f64_with_queue(CommandQueue::default(), 5, MemFlag::default(), EMPTY)?.wait()?;

    println!("{u:?} - {s:?}");
    Ok(())
} 