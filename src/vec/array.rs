use core::ops::DerefMut;
use crate::{prelude::{ArrayBuffer, Context, ErrorCL}, buffer::MemFlags};

#[derive(Clone)]
#[repr(transparent)]
pub struct ArrayVector<T: Copy + Unpin, const N: usize> (ArrayBuffer<T, N>);

impl<T: Copy + Unpin, const N: usize> ArrayVector<T, N> {
    #[inline(always)]
    pub fn new (ctx: &Context, flags: impl Into<Option<MemFlags>>, v: &[T;N]) -> Result<Self, ErrorCL> {
        ArrayBuffer::new(ctx, flags, v).map(Self)
    }

    #[inline(always)]
    pub fn from_buffer (v: ArrayBuffer<T, N>) -> Self {
        Self(v)
    }
}

impl<T: Copy + Unpin, const N: usize> core::ops::Deref for ArrayVector<T, N> {
    type Target = ArrayBuffer<T, N>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Copy + Unpin, const N: usize> DerefMut for ArrayVector<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}