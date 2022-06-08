use opencl::{prelude::*};

const TEST_KERNEL : &'static str = "void kernel add (const int n, __global const float* rhs, __global const float* in, __global float* out) {
    for (int id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

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

#[test]
fn context () {
    let context = Context::new(None, Device::all()).unwrap();
}

#[test]
fn program () {
    let (ctx, program) = Program::context_from_source(None, Device::all(), TEST_KERNEL).unwrap();
}