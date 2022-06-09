use opencl::{prelude::*, buffer::{UnsafeBuffer, ArrayBuffer}, event::Event};

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
    let device = Device::all().first().unwrap();
    let ctx = Context::new(None, core::slice::from_ref(device)).unwrap();
    let queue = CommandQueue::new(&ctx, device, None).unwrap();

    let buffer = UnsafeBuffer::<f32>::new(&ctx, 4, None).unwrap();
    unsafe {
        let write = buffer.write(&queue, false, 0, vec![1.0, 2.0, 3.0, 4.0], None).unwrap();
        let read = buffer.read(&queue, true, 0, 4, [write.borrow_base()]).unwrap().wait().unwrap();
        println!("Read: {read:?}");
    }

    let array = ArrayBuffer::new(&ctx, None, [1, 2]).unwrap();
    let read = array.get(&queue, 1, []).unwrap().unwrap().wait().unwrap();
    println!("Value @ 1 is {read}")
}

#[tokio::test]
async fn sync () {
    let device = Device::all().first().unwrap();
    let ctx = Context::new(None, core::slice::from_ref(device)).unwrap();
    let queue = CommandQueue::new(&ctx, device, None).unwrap();
    let buffer = UnsafeBuffer::<f32>::new(&ctx, 1000, None).unwrap();

    unsafe {
        let write = buffer.write(&queue, false, 0, vec![1.0; 1000], None).unwrap();
        let read = buffer.read(&queue, false, 0, 1000, [write.borrow_base()]).unwrap().await.unwrap();
        println!("Read: {:?}", &read[..5]);
    }
}