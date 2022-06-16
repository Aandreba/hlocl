flat_mod!(ctx);
use core::marker::PhantomData;

#[repr(transparent)]
pub struct UsizeCheck<T> (PhantomData<T>);
pub trait IsUsize {}
impl IsUsize for UsizeCheck<usize> {}