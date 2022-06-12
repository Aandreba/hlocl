use opencl::{vec::{Vector}};

#[test]
fn vec () {
    let alpha = Vector::<i32>::new(&[1, 2, 3, 4]).unwrap();
    //llet beta = Vector::<i32>::new(&[5, 6, 7, 8]).unwrap();

    let gamma = alpha.slice(0..2).unwrap();
    println!("{:?}", gamma);

    let gamma = unsafe { Vector::from_buffer(gamma) };
    println!("{:?}", alpha.sum().unwrap());
}