use core::marker::PhantomData;
use opencl_sys::{cl_mem, clRetainMemObject};

#[repr(transparent)]
pub struct ReadSlice<'a, T: 'static + Copy + Unpin> (pub(crate) cl_mem, PhantomData<&'a T>);

impl<'a, T: Copy + Unpin> Clone for ReadSlice<'a, T> {
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainMemObject(self.0))
        }

        Self(self.0.clone(), PhantomData)
    }
}