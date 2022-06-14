use opencl::{prelude::*, buffer::{MemFlag, FastRng}};

#[test]
fn vec () -> Result<()> {
    let dev = Device::all();
    println!("{dev:?}");
    
    //let ctx = Context::new(None, &Device::all())?;
    //let rng = FastRng::with_context(Context::default(), 10)?;
    //let buff = rng.random_u8_with_queue(CommandQueue::default(), 10, MemFlag::default(), EMPTY)?;
    //let buff = buff.wait()?;

    //println!("{buff:?}");
    Ok(())
}