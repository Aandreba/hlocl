use opencl::{vec::{Vector}};

#[test]
fn vec () {
    let alpha = Vector::<i32>::new(&[1, 2, 3, 4, 5]).unwrap();
    println!("sum {alpha:?} = {:?}", alpha.sum().unwrap());
}