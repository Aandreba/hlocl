use opencl::prelude::*;

#[test]
fn platforms () {
    let platforms = Platform::all();
    println!("{:?}", platforms);
}

#[test]
fn devices () {
    let devices = Device::all();
    println!("{:?}", devices);
}