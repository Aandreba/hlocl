use std::f32::consts::PI;

use opencl::{prelude::*, buffer::{ArrayBuffer, MemFlags}, vec::{Vector, VectorManager}, utils::ContextManager};

const TEST_KERNEL : &'static str = "void kernel add (const int n, __global const int* rhs, __global const int* in, __global int* out) {
    for (int id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn sum () {
    let device = Device::first().unwrap();
    let ctx = Context::new(None, core::slice::from_ref(device)).unwrap();
    let queue = CommandQueue::new(&ctx, device, None).unwrap();

    let left = ArrayBuffer::new(&ctx, None, &[1, 2, 3, 4, 5]).unwrap();
    let right = ArrayBuffer::new(&ctx, None, &[6, 7, 8, 9, 10]).unwrap();
    let result = unsafe { ArrayBuffer::<i32, 5>::uninit(&ctx, MemFlags::READ_ONLY).unwrap() };

    let program = Program::from_source(&ctx, TEST_KERNEL).unwrap();
    let mut kernel = Kernel::new(&program, "add").unwrap();

    unsafe {
        kernel.set_arg(0, 5i32).unwrap();
        kernel.set_mem_arg(1, &right).unwrap();
        kernel.set_mem_arg(2, &left).unwrap();
        kernel.set_mem_arg(3, &result).unwrap();
    }

    let sum = kernel.enqueue(&queue, &[5, 1, 1], None, []).unwrap();
    let read = result.to_array(&queue, [&sum]).unwrap();

    println!("{:?}", read.wait());
}

#[test]
fn vec () {
    let ctx = ContextManager::default();
    let manager = VectorManager::<f32>::new(Context::clone(&ctx)).unwrap();

    let alpha = Vector::new(&manager, None, &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    let beta = Vector::new(&manager, None, &[6.0, 7.0, 8.0, 9.0, 10.0]).unwrap();

    let sum = alpha.add(&beta, ctx.queue(), None, &manager, []);
    let mut sum = sum.unwrap().wait().unwrap();
    
    sum.set(ctx.queue(), 0, PI, []).unwrap().wait().unwrap();
    let sum = sum.to_vec(ctx.queue(), []).unwrap().wait().unwrap();
    
    println!("{:?}", sum)
}