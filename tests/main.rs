use opencl::{prelude::*, buffer::MemFlags};

#[test]
fn vec () -> Result<()> {
    let input = MemBuffer::new(&[1.0, 2.0, 3f32, 4.0, 5.0], MemFlags::READ_WRITE)?;
    
    Ok(())
}