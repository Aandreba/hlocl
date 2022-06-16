use opencl::{prelude::*, buffer::{FastRng, MemFlag}};

#[test]
fn vec () -> Result<()> {
    let device = Device::first().unwrap();
    println!("{} & {}", device.version()?, device.driver_version()?);
    
    let svm = FastRng::random_f64(0.0, 1.0, 10_000, MemFlag::default(), EMPTY)?.wait()?;
    println!("{svm:?}");
    Ok(())
} 