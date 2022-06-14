use opencl::{prelude::*, buffer::MemFlag};

#[test]
fn vec () -> Result<()> {
    let buff = MemBuffer::<f32>::random(10, MemFlag::default())?;
    let ser = serde_json::to_string(&buff).unwrap();
    let de = serde_json::from_str::<MemBuffer<f64>>(&ser).unwrap();

    println!("{buff:?} -> {ser} -> {de:?}");
    Ok(())
}