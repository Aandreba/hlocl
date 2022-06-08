use core::{ptr::NonNull, marker::PhantomData};
use alloc::{vec::Vec, borrow::Cow};
use cl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clCreateBuffer, size_t};
use crate::{prelude::{Context, ErrorCL, CommandQueue}, event::{ReadBuffer, BaseEvent, WriteBuffer, Event, CopyBuffer}};
use super::MemFlags;

#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UnsafeBuffer<T: Copy + Unpin> (pub(crate) cl_mem, PhantomData<T>); 

impl<T: Copy + Unpin> UnsafeBuffer<T> {
    #[inline(always)]
    pub fn new (ctx: &Context, size: size_t, flags: Option<MemFlags>) -> Result<Self, ErrorCL> {
        unsafe { Self::with_host_ptr(ctx, size, flags.unwrap_or_else(MemFlags::default), None) }
    }

    #[inline(always)]
    pub fn new_and_copy (ctx: &Context, flags: Option<MemFlags>, src: &[T]) -> Result<Self, ErrorCL> {
        let flags = flags.unwrap_or_default() | MemFlags::COPY_HOST_PTR;
        unsafe { Self::with_host_ptr(ctx, src.len(), flags, NonNull::new(src.as_ptr() as *mut _)) }
    }

    pub unsafe fn with_host_ptr (ctx: &Context, size: size_t, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self, ErrorCL> {
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr().cast(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let size = size.checked_mul(core::mem::size_of::<T>()).expect("Buffer size overflow");
        let id = clCreateBuffer(ctx.0, flags.bits(), size, host_ptr, &mut err);

        if err == 0 {
            return Ok(Self(id, PhantomData));
        }

        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn copy_to<'a> (&self, queue: &CommandQueue, src_pffset: size_t, dst: UnsafeBuffer<T>, dst_offset: size_t, len: size_t, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<CopyBuffer<T>, ErrorCL> {
        CopyBuffer::new(queue, src_pffset, dst_offset, len, self.clone(), dst, wait)
    }

    #[inline(always)]
    pub unsafe fn read<'a> (&self, queue: &CommandQueue, blocking: bool, offset: size_t, len: size_t, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<impl Event<Result = Vec<T>>, ErrorCL> where T: 'static {
        let mut dst = Vec::<T>::with_capacity(len);
        let read = self.read_into_ptr(queue, blocking, offset, dst.as_mut_ptr(), len, wait)?;
       
        Ok(read.then(move |_| {
            dst.set_len(len);
            dst
        }))
    }

    #[inline(always)]
    pub unsafe fn read_into<'a, 'b> (&self, queue: &CommandQueue, blocking: bool, offset: size_t, dst: &'a mut [T], wait: impl IntoIterator<Item = &'b BaseEvent>) -> Result<ReadBuffer<'a, T>, ErrorCL> {
        ReadBuffer::new(queue, blocking, offset, self.clone(), dst, wait)
    }

    #[inline(always)]
    pub unsafe fn read_into_ptr<'b> (&self, queue: &CommandQueue, blocking: bool, offset: size_t, dst: *mut T, len: size_t, wait: impl IntoIterator<Item = &'b BaseEvent>) -> Result<ReadBuffer<'static, T>, ErrorCL> {
        let dst = core::slice::from_raw_parts_mut(dst, len);
        self.read_into(queue, blocking, offset, dst, wait)
    }

    #[inline(always)]
    pub unsafe fn write<'a, 'b> (&self, queue: &CommandQueue, blocking: bool, offset: size_t, src: impl Into<Cow<'a, [T]>>, wait: impl IntoIterator<Item = &'b BaseEvent>) -> Result<WriteBuffer<'a, T>, ErrorCL> where T: 'a {
        WriteBuffer::new(queue, blocking, offset, src, self.clone(), wait)
    }
}

impl<T: Copy + Unpin> Clone for UnsafeBuffer<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainMemObject(self.0));
        }
        
        Self(self.0, self.1)
    }
}

impl<T: Copy + Unpin> Drop for UnsafeBuffer<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.0));
        }
    }
}

unsafe impl<T: Send + Copy + Unpin> Send for UnsafeBuffer<T> {}
unsafe impl<T: Sync + Copy + Unpin> Sync for UnsafeBuffer<T> {}