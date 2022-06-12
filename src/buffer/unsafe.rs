use core::{ptr::{NonNull, addr_of}, marker::PhantomData, mem::{MaybeUninit, ManuallyDrop}};
use alloc::{vec::Vec, boxed::Box};
use cl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clCreateBuffer, cl_mem_info, clGetMemObjectInfo, CL_MEM_FLAGS, CL_MEM_SIZE, c_void, CL_MEM_HOST_PTR, CL_MEM_MAP_COUNT, CL_MEM_REFERENCE_COUNT, CL_MEM_CONTEXT, CL_MEM_ASSOCIATED_MEMOBJECT, CL_MEM_OFFSET, clCreateSubBuffer, CL_BUFFER_CREATE_TYPE_REGION};
use crate::{prelude::{Context, ErrorCL, CommandQueue}, event::{ReadBuffer, BaseEvent, WriteBuffer, Event, CopyBuffer, various::{Swap, Then}}};
use super::{MemFlags};

#[repr(transparent)]
pub struct UnsafeBuffer<T: Copy + Unpin> (pub(crate) cl_mem, pub(super) PhantomData<T>); 

impl<T: Copy + Unpin> UnsafeBuffer<T> {
    #[inline(always)]
    pub fn new (ctx: &Context, size: usize, flags: impl Into<Option<MemFlags>>) -> Result<Self, ErrorCL> {
        unsafe { Self::with_host_ptr(ctx, size, flags.into().unwrap_or_default(), None) }
    }

    #[inline(always)]
    pub fn new_and_copy (ctx: &Context, flags: impl Into<Option<MemFlags>>, src: &[T]) -> Result<Self, ErrorCL> {
        let flags = flags.into().unwrap_or_default() | MemFlags::COPY_HOST_PTR;
        unsafe { Self::with_host_ptr(ctx, src.len(), flags, NonNull::new(src.as_ptr() as *mut _)) }
    }

    pub unsafe fn with_host_ptr (ctx: &Context, size: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self, ErrorCL> {
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
    pub fn ty (&self) -> Result<(), ErrorCL> {
        todo!()
    }

    /// Returns the flags argument value specified when memobj is created
    #[inline(always)]
    pub fn flags (&self) -> Result<MemFlags, ErrorCL> {
        self.get_info(CL_MEM_FLAGS)
    }

    #[inline(always)]
    pub fn len (&self) -> Result<usize, ErrorCL> {
        let bytes = self.byte_size()?;
        Ok(bytes / core::mem::size_of::<T>())
    }

    #[inline(always)]
    pub fn byte_size (&self) -> Result<usize, ErrorCL> {
        self.get_info(CL_MEM_SIZE)
    }

    #[inline(always)]
    pub fn host_ptr (&self) -> Result<Option<NonNull<c_void>>, ErrorCL> {
        self.get_info(CL_MEM_HOST_PTR).map(NonNull::new)
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_MEM_MAP_COUNT)
    }

    /// Return _memobj_ reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_MEM_REFERENCE_COUNT)
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context (&self) -> Result<Context, ErrorCL> {
        self.get_info(CL_MEM_CONTEXT)
    }

    /// Return memory object from which memobj is created. 
    #[inline(always)]
    pub fn parent (&self) -> Result<Option<UnsafeBuffer<T>>, ErrorCL> {
        let id = self.get_info::<cl_mem>(CL_MEM_ASSOCIATED_MEMOBJECT)?;
        if id.is_null() { return Ok(None); }
        Ok(Some(UnsafeBuffer(id, PhantomData)))
    }

    #[inline(always)]
    pub fn offset (&self) -> Result<usize, ErrorCL> {
        self.get_info(CL_MEM_OFFSET)
    }

    #[inline(always)]
    pub unsafe fn transmute<O: Copy + Unpin> (self) -> UnsafeBuffer<O> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<O>());
        let me = ManuallyDrop::new(self);
        UnsafeBuffer(me.0, PhantomData)
    }

    #[inline(always)]
    pub unsafe fn get_unchecked (&self, queue: &CommandQueue, idx: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<impl Event<Result = T>, ErrorCL> where T: 'static {
        let evt = self.read(queue, false, idx, 1, wait)?;
        Ok(Event::map(evt, |x| x[0]))
    }

    #[inline(always)]
    pub unsafe fn set_unchecked (&mut self, queue: &CommandQueue, idx: usize, v: T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Then<WriteBuffer, impl FnOnce(&mut ())>, ErrorCL> where T: 'static {
        let ptr = Box::into_raw(Box::new(v));
        let evt = self.write(queue, false, idx, core::slice::from_raw_parts(ptr, 1), wait)?;
        // TODO potential memory leak?
        Ok(evt.then(move |_| drop(Box::from_raw(ptr))))
    }

    #[inline(always)]
    pub unsafe fn slice_unchecked (&self, offset: usize, len: usize) -> Result<Self, ErrorCL> {
        let flags = self.flags()? & (MemFlags::READ_WRITE | MemFlags::READ_ONLY | MemFlags::WRITE_ONLY);
        let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
        let len = len.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");

        let region = cl_sys::cl_buffer_region {
            origin: offset,
            size: len
        };
        
        let mut err = 0;
        let id = clCreateSubBuffer(self.0, flags.bits(), CL_BUFFER_CREATE_TYPE_REGION, addr_of!(region).cast(), &mut err);

        if err == 0 {
            return Ok(Self(id, PhantomData));
        }

        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn copy_to (&self, queue: &CommandQueue, src_pffset: usize, dst: UnsafeBuffer<T>, dst_offset: usize, len: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<CopyBuffer<T>, ErrorCL> {
        CopyBuffer::new(queue, src_pffset, dst_offset, len, self, dst, wait)
    }

    #[inline(always)]
    pub unsafe fn read (&self, queue: &CommandQueue, blocking: bool, offset: usize, len: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Vec<T>, ReadBuffer<'static>>, ErrorCL> where T: 'static {
        let mut dst = Vec::with_capacity(len);
        dst.set_len(len);

        let read = self.read_into_ptr(queue, blocking, offset, dst.as_mut_ptr(), len, wait)?;
        Ok(read.swap(dst))
    }

    #[inline(always)]
    pub unsafe fn read_into<'a> (&self, queue: &CommandQueue, blocking: bool, offset: usize, dst: &'a mut [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'a>, ErrorCL> {
        ReadBuffer::new(queue, blocking, offset, self, dst, wait)
    }

    #[inline(always)]
    pub unsafe fn read_into_ptr (&self, queue: &CommandQueue, blocking: bool, offset: usize, dst: *mut T, len: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'static>, ErrorCL> where T: 'static {
        let dst = core::slice::from_raw_parts_mut(dst, len);
        self.read_into(queue, blocking, offset, dst, wait)
    }

    #[inline(always)]
    pub unsafe fn write<'a> (&mut self, queue: &CommandQueue, blocking: bool, offset: usize, src: &'a [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<WriteBuffer<'a>, ErrorCL> where T: 'a {
        WriteBuffer::new(queue, blocking, offset, src, self, wait)
    }

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O, ErrorCL> {
        let mut result = MaybeUninit::<O>::uninit();
        unsafe {
            let err = clGetMemObjectInfo(self.0, ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(result.assume_init());
            }

            Err(ErrorCL::from(err))
        }
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