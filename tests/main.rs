use hlocl::{prelude::*, buffer::{MemFlag, FastRng}, event::various::Swap};

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn main () -> Result<()> {
    let alpha = FastRng::random_f32(0., 1., 10, MemFlag::default(), EMPTY)?;
    let beta = FastRng::random_f32(0., 1., 10, MemFlag::default(), EMPTY)?;
    let gamma = unsafe { MemBuffer::<f32>::uninit(10, MemFlag::WRITE_ONLY) }?;

    let [alpha, beta] = Swap::wait_all_array([alpha, beta])?;
    println!("{alpha:?} + {beta:?}");

    let prog = Program::from_source(PROGRAM)?;
    let mut kernel = unsafe { Kernel::new_unchecked(&prog, "add")? };

    kernel.set_arg(0, 10u64)?;
    kernel.set_mem_arg(1, &alpha)?;
    kernel.set_mem_arg(2, &beta)?;
    kernel.set_mem_arg(3, &gamma)?;

    let evt = kernel.enqueue(&[10, 1, 1], None, EMPTY)?;
    let gamma = gamma.to_vec([evt])?.wait()?;
    println!("{gamma:?}");

    Ok(())
}