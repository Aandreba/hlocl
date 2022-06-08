use core::ptr::NonNull;
use cl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clCreateBuffer, size_t};
use crate::prelude::Context;
use super::MemFlags;

#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UnsafeBuffer (cl_mem); 

impl UnsafeBuffer {
    pub unsafe fn new (ctx: &Context, size: size_t, flags: MemFlags, host_ptr: Option<NonNull<u8>>) -> Self {
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr().cast(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateBuffer(ctx.0, flags.bits(), size, host_ptr, &mut err)
        };

        // TODO finish
        todo!()
    }
}

impl Clone for UnsafeBuffer {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainMemObject(self.0));    
        }
        
        Self(self.0)
    }
}

impl Drop for UnsafeBuffer {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.0));
        }
    }
}

unsafe impl Send for UnsafeBuffer {}
unsafe impl Sync for UnsafeBuffer {}