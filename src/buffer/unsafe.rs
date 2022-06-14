use core::{ptr::{NonNull, addr_of}, marker::PhantomData, mem::{MaybeUninit, ManuallyDrop}, ops::{RangeBounds, Bound}, fmt::Debug};
use alloc::{vec::{Vec, IntoIter}, boxed::Box, format};
use cl_sys::{cl_mem, clReleaseMemObject, clCreateBuffer, cl_mem_info, clGetMemObjectInfo, CL_MEM_FLAGS, CL_MEM_SIZE, c_void, CL_MEM_HOST_PTR, CL_MEM_MAP_COUNT, CL_MEM_REFERENCE_COUNT, CL_MEM_CONTEXT, CL_MEM_ASSOCIATED_MEMOBJECT, CL_MEM_OFFSET, clCreateSubBuffer, CL_BUFFER_CREATE_TYPE_REGION};
use crate::{prelude::{Result, Context, Error, CommandQueue, EMPTY}, event::{ReadBuffer, BaseEvent, WriteBuffer, Event, CopyBuffer, various::{Then, Map}}};
use super::{MemFlag};

#[repr(transparent)]
pub struct MemBuffer<T: 'static + Copy + Unpin> (pub(crate) cl_mem, pub(super) PhantomData<T>); 

impl<T: Copy + Unpin> MemBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn uninit (size: usize, flags: MemFlag) -> Result<Self> {
        Self::uninit_with_context(Context::default(), size, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (src: &[T], flags: MemFlag) -> Result<Self> {
        Self::with_context(Context::default(), flags, src)
    }

    #[inline(always)]
    pub unsafe fn uninit_with_context (ctx: &Context, size: usize, flags: impl Into<Option<MemFlag>>) -> Result<Self> {
        Self::with_host_ptr(ctx, size, flags.into().unwrap_or_default(), None)
    }

    #[inline(always)]
    pub fn with_context (ctx: &Context, flags: MemFlag, src: &[T]) -> Result<Self> {
        let flags = flags | MemFlag::COPY_HOST_PTR;
        unsafe { Self::with_host_ptr(ctx, src.len(), flags, NonNull::new(src.as_ptr() as *mut _)) }
    }

    pub unsafe fn with_host_ptr (ctx: &Context, size: usize, flags: MemFlag, host_ptr: Option<NonNull<T>>) -> Result<Self> {
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

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidContext => report.attach_printable(format!("'{:?}' is not a valid context", ctx.0)),
                    Error::InvalidValue => report.attach_printable(format!("'{:?}' is not a valid flag", flags)),
                    Error::InvalidBufferSize => report.attach_printable("size is zero or greater than the max"),
                    Error::InvalidHostPtr => report.attach_printable(format!("'{:?}' is not a valid host pointer", host_ptr)),
                    Error::MemObjectAllocationFailure => report.attach_printable("failed to allocate memory object"),
                    Error::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }

    #[inline(always)]
    pub fn ty (&self) -> Result<()> {
        todo!()
    }

    /// Returns the flags argument value specified when memobj is created
    #[inline(always)]
    pub fn flags (&self) -> Result<MemFlag> {
        self.get_info(CL_MEM_FLAGS)
    }

    #[inline(always)]
    pub fn len (&self) -> Result<usize> {
        let bytes = self.byte_size()?;
        Ok(bytes / core::mem::size_of::<T>())
    }

    #[inline(always)]
    pub fn byte_size (&self) -> Result<usize> {
        self.get_info(CL_MEM_SIZE)
    }

    #[inline(always)]
    pub fn host_ptr (&self) -> Result<Option<NonNull<c_void>>> {
        self.get_info(CL_MEM_HOST_PTR).map(NonNull::new)
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_MAP_COUNT)
    }

    /// Return _memobj_ reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_REFERENCE_COUNT)
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.get_info(CL_MEM_CONTEXT)
    }

    /// Return memory object from which memobj is created. 
    #[inline(always)]
    pub fn parent (&self) -> Result<Option<MemBuffer<T>>> {
        let id = self.get_info::<cl_mem>(CL_MEM_ASSOCIATED_MEMOBJECT)?;
        if id.is_null() { return Ok(None); }
        Ok(Some(MemBuffer(id, PhantomData)))
    }

    #[inline(always)]
    pub fn offset (&self) -> Result<usize> {
        self.get_info(CL_MEM_OFFSET)
    }

    #[inline(always)]
    pub unsafe fn transmute<O: Copy + Unpin> (self) -> MemBuffer<O> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<O>());
        let me = ManuallyDrop::new(self);
        MemBuffer(me.0, PhantomData)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn to_vec (&self, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<Vec<T>, ReadBuffer<'_, 'static>, impl FnOnce(()) -> Vec<T>>> {
        self.to_vec_with_queue(CommandQueue::default(), wait)
    }

    #[inline(always)]
    pub fn to_vec_with_queue (&self, queue: &CommandQueue, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<Vec<T>, ReadBuffer<'_, 'static>, impl FnOnce(()) -> Vec<T>>> {
        self.read_with_queue(queue, .., wait)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn get (&self, idx: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<T, Map<Vec<T>, ReadBuffer<'_, 'static>, impl FnOnce(()) -> Vec<T>>, impl FnOnce(Vec<T>) -> T>> {
        self.get_with_queue(CommandQueue::default(), idx, wait)
    }

    #[inline(always)]
    pub fn get_with_queue (&self, queue: &CommandQueue, idx: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<T, Map<Vec<T>, ReadBuffer<'_, 'static>, impl FnOnce(()) -> Vec<T>>, impl FnOnce(Vec<T>) -> T>> {
        let evt = self.read_with_queue(queue, idx..=idx, wait)?;
        Ok(Event::map(evt, |v| v[0]))
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn set (&mut self, idx: usize, v: T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Then<WriteBuffer, impl FnOnce(&mut ())>> {
        self.set_with_queue(CommandQueue::default(), idx, v, wait)
    }

    #[inline(always)]
    pub fn set_with_queue (&mut self, queue: &CommandQueue, idx: usize, v: T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Then<WriteBuffer, impl FnOnce(&mut ())>> {
        let ptr = Box::into_raw(Box::new(v));
        let src = unsafe { core::slice::from_raw_parts(ptr, 1) };

        let evt = self.write_with_queue(queue, idx, src, wait)?;
        Ok(evt.then(move |_| unsafe { drop(Box::from_raw(ptr)) }))
    }

    #[inline(always)]
    pub fn slice (&self, range: impl RangeBounds<usize>) -> Result<Self> {
        let (offset, len) = self.get_offset_len(&range)?;
        let flags = self.flags()? & (MemFlag::READ_WRITE | MemFlag::READ_ONLY | MemFlag::WRITE_ONLY);
        let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
        let len = len.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");

        let region = cl_sys::cl_buffer_region {
            origin: offset,
            size: len
        };
        
        let mut err = 0;
        let id = unsafe { clCreateSubBuffer(self.0, flags.bits(), CL_BUFFER_CREATE_TYPE_REGION, addr_of!(region).cast(), &mut err) };

        if err == 0 {
            return Ok(Self(id, PhantomData));
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidValue => report.attach_printable(format!("'{:?}' is not a valid flag", flags)),
                    Error::InvalidMemObject => report.attach_printable(format!("'{:?}' is not a valid memory object", self.0)),
                    Error::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    Error::OutOfResources => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the device"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn copy_to<'a> (&self, offset: usize, dst: &'a mut MemBuffer<T>, dst_range: impl RangeBounds<usize>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<CopyBuffer<'_, 'a>> {
        self.copy_to_with_queue(CommandQueue::default(), offset, dst, dst_range, wait)
    }

    #[inline(always)]
    pub fn copy_to_with_queue<'a, 'b> (&'a self, queue: &CommandQueue, offset: usize, dst: &'b mut MemBuffer<T>, dst_range: impl RangeBounds<usize>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<CopyBuffer<'a, 'b>> {
        let (dst_offset, len) = self.get_offset_len(&dst_range)?;
        CopyBuffer::new(queue, offset, dst_offset, len, self, dst, wait)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn read (&self, range: impl RangeBounds<usize>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<Vec<T>, ReadBuffer<'_, 'static>, impl FnOnce(()) -> Vec<T>>> {
        self.read_with_queue(CommandQueue::default(), range, wait)
    }

    #[inline(always)]
    pub fn read_with_queue<'a> (&'a self, queue: &CommandQueue, range: impl RangeBounds<usize>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Map<Vec<T>, ReadBuffer<'a, 'static>, impl FnOnce(()) -> Vec<T>>> {
        let (offset, len) = self.get_offset_len(&range)?;
        let (ptr, _, len) = Vec::with_capacity(len).into_raw_parts();
        
        let read = unsafe { 
            let dst = core::slice::from_raw_parts_mut::<'static, T>(ptr, len);
            self.read_into_with_queue(queue, offset, dst, wait)
        };

        match read {
            Ok(read) => Ok(read.map(move |_| unsafe { Vec::from_raw_parts(ptr, len, len) })),
            Err(e) => {
                let _ = unsafe { Vec::from_raw_parts(ptr, len, len) };
                Err(e)
            }
        }
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn read_into<'a> (&self, offset: usize, dst: &'a mut [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'_, 'a>> {
        self.read_into_with_queue(CommandQueue::default(), offset, dst, wait)
    }

    #[inline(always)]
    pub fn read_into_with_queue<'a> (&self, queue: &CommandQueue, offset: usize, dst: &'a mut [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'_, 'a>> {
        ReadBuffer::new(queue, false, offset, self, dst, wait)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn read_into_ptr (&self, range: impl RangeBounds<usize>, dst: *mut T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'_, 'static>> {
        self.read_into_ptr_with_queue(CommandQueue::default(), range, dst, wait)
    }

    #[inline(always)]
    pub unsafe fn read_into_ptr_with_queue<'a> (&'a self, queue: &CommandQueue, range: impl RangeBounds<usize>, dst: *mut T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<ReadBuffer<'a, 'static>> {
        let (offset, len) = self.get_offset_len(&range)?;
        let dst = core::slice::from_raw_parts_mut(dst, len);
        self.read_into_with_queue(queue, offset, dst, wait)
    }

    #[inline(always)]
    pub fn write<'a> (&mut self, queue: &CommandQueue, offset: usize, src: &'a [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<WriteBuffer<'a, '_>> {
        WriteBuffer::new(queue, false, offset, src, self, wait)
    }

    #[inline(always)]
    pub fn write_with_queue<'a> (&mut self, queue: &CommandQueue, offset: usize, src: &'a [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<WriteBuffer<'a, '_>> {
        WriteBuffer::new(queue, false, offset, src, self, wait)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn iter (&self, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<IntoIter<T>> {
        self.iter_with_queue(CommandQueue::default(), wait)
    }

    #[inline(always)]
    pub fn iter_with_queue (&self, queue: &CommandQueue, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<IntoIter<T>> {
        let vec = self.to_vec_with_queue(queue, wait)?;
        let vec = vec.wait()?;
        Ok(vec.into_iter())
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn into_iter (self, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<IntoIter<T>> {
        self.iter(wait)
    } 
    #[inline(always)]
    pub fn into_iter_with_queue (self, queue: &CommandQueue, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<IntoIter<T>> {
        self.iter_with_queue(queue, wait)
    }

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            let err = clGetMemObjectInfo(self.0, ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(result.assume_init());
            }

            cfg_if::cfg_if! {
                if #[cfg(feature = "error-stack")] {
                    let err = Error::from(err);
                    let report = error_stack::Report::new(err);
    
                    let report = match err {
                        Error::InvalidMemObject => report.attach_printable(format!("'{:?}' is not a valid memory object", self.0)),
                        Error::InvalidValue => report.attach_printable(format!("'{ty}' is not one of the supported values or size in bytes specified by param_value_size is less than size of return type and param_value is not a NULL value")),
                        _ => report
                    };
    
                    Err(report)
                } else {
                    Err(Error::from(err))
                }
            }
        }
    }

    #[inline(always)]
    fn get_offset_len (&self, range: &impl RangeBounds<usize>) -> Result<(usize, usize)> {
        let offset = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + 1,
            Bound::Unbounded => 0
        };

        let len = match range.end_bound() {
            Bound::Included(x) => *x - offset + 1,
            Bound::Excluded(x) => *x - offset,
            Bound::Unbounded => self.len()? - offset
        };

        Ok((offset, len))
    }
}

#[cfg(feature = "def")]
impl<T: Copy + Unpin + Debug> Debug for MemBuffer<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let vec = self.to_vec(EMPTY).map_err(|_| core::fmt::Error)?;
        let vec = vec.wait().map_err(|_| core::fmt::Error)?;
        Debug::fmt(&vec, f)
    }
}

impl<T: Copy + Unpin> Drop for MemBuffer<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.0));
        }
    }
}

unsafe impl<T: Send + Copy + Unpin> Send for MemBuffer<T> {}
unsafe impl<T: Sync + Copy + Unpin> Sync for MemBuffer<T> {}