use std::{f32::consts::TAU, time::Instant};
use opencl::{prelude::*, buffer::{MemFlag, FastRng}};

#[test]
fn vec () -> Result<()> {
    let first = FastRng::random_f32(-1f32, 1f32, 1000, MemFlag::default(), EMPTY)?.wait()?;
    let last = FastRng::random_f32(0f32, TAU, 25, MemFlag::default(), EMPTY)?.wait()?;

    println!("First: {first_time:?}");
    println!("Last: {last_time:?}");
    Ok(())
} 