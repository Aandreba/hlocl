use opencl::{prelude::*, buffer::{MemFlag, FastRng}, svm::{SvmBuffer, SvmFlag}};

#[test]
fn vec () -> Result<()> {
    let dev = Device::first().unwrap();
    println!("{:?}", dev.version());

    Ok(())
}