use hlocl::{prelude::*, buffer::{MemFlag, FastRng}, event::various::Swap};

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn main () -> Result<()> {
    let alpha = FastRng

    let prog = Program::from_source_with_context(&ctx, PROGRAM)?;

    //panic!("{:?}", ctx.reference_count());
    Ok(())
}