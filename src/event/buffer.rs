use core::{marker::PhantomData};
use alloc::{vec::Vec, format};
use opencl_sys::{cl_event, clEnqueueWriteBuffer, clEnqueueCopyBuffer, clEnqueueReadBuffer};
use crate::{prelude::{Result, Error, CommandQueue}, buffer::{MemBuffer}};
use super::{BaseEvent, Event};

/// OpenCL event that reads from one buffer to another 
#[repr(transparent)]
pub struct CopyBuffer<'a, 'b> {
    inner: BaseEvent,
    phtm: PhantomData<(&'a (), &'b ())>
}

impl<'a, 'b> CopyBuffer<'a, 'b> {
    pub fn new<T: 'static + Copy + Unpin> (queue: &CommandQueue, src_offset: usize, dst_offset: usize, len: usize, src: &'a MemBuffer<T>, dst: &'b mut MemBuffer<T>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Self> {
        let wait = wait.into_iter().map(|x| x.as_ref().0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event : cl_event = core::ptr::null_mut();
        let err = unsafe { clEnqueueCopyBuffer(queue.0, src.0, dst.0, src_offset, dst_offset, len, wait_len, wait, &mut event) };

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, phtm: PhantomData });
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidCommandQueue => report.attach_printable(format!("'{:?}' is not a valid command-queue", queue.0)),
                    Error::InvalidContext => report.attach_printable("the context associated with the command queue and buffer are not the same or the context associated with command queue and events in the event wait list are not the same"),
                    Error::InvalidMemObject => report.attach_printable(format!("'{:?}' and/or '{:?}' are not a valid buffer", src.0, dst.0)),
                    Error::InvalidValue => report.attach_printable("the region being written is out of bounds or ptr is a NULL value"),
                    Error::InvalidEventWaitList => report.attach_printable("event objects in the event wait list are not valid events"),
                    Error::MemObjectAllocationFailure => report.attach_printable("there is a failure to allocate memory for data store associated with buffer"),
                    Error::OutOfHostMemory => report.attach_printable("there is a failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
}

impl Event for CopyBuffer<'_, '_> {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
        self.inner.wait()?;
        Ok(())
    }

    #[inline(always)]
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<alloc::vec::Vec<Self::Result>> {
        let iter = iter.into_iter().map(|x| x.inner);
        BaseEvent::wait_all(iter)
    }
}

#[cfg(feature = "async")]
impl futures::Future for CopyBuffer<'_, '_> {
    type Output = Result<()>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.inner).poll(cx)
    }
}

impl AsRef<BaseEvent> for CopyBuffer<'_, '_> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}

/// Event that writes from host memory to an OpenCL buffer
#[repr(transparent)]
pub struct WriteBuffer<'a, 'b> {
    inner: BaseEvent,
    phtm: PhantomData<(&'a (), &'b ())>
}

impl<'a, 'b> WriteBuffer<'a, 'b> {
    pub unsafe fn new_by_ref<T: 'static + Copy + Unpin> (queue: &CommandQueue, blocking: bool, offset: usize, src: &'a [T], dst: &'b MemBuffer<T>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Self> {
        let wait = wait.into_iter().map(|x| x.as_ref().0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
        let len = src.len().checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");

        let mut event : cl_event = core::ptr::null_mut();
        let err = clEnqueueWriteBuffer(queue.0, dst.0, opencl_sys::cl_bool::from(blocking), offset, len, src.as_ptr().cast(), wait_len, wait, &mut event);

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, phtm: PhantomData });
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidCommandQueue => report.attach_printable(format!("'{:?}' is not a valid command-queue", queue.0)),
                    Error::InvalidContext => report.attach_printable("the context associated with the command queue and buffer are not the same or the context associated with command queue and events in the event wait list are not the same"),
                    Error::InvalidMemObject => report.attach_printable(format!("'{:?}' is not a valid buffer", dst.0)),
                    Error::InvalidValue => report.attach_printable("the region being written is out of bounds or ptr is a NULL value"),
                    Error::InvalidEventWaitList => report.attach_printable("event objects in the event wait list are not valid events"),
                    Error::MemObjectAllocationFailure => report.attach_printable("there is a failure to allocate memory for data store associated with buffer"),
                    Error::OutOfResources => report.attach_printable("there is a failure to allocate resources required by the OpenCL implementation on the device"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }

    #[inline(always)]
    pub fn new<T: Copy + Unpin> (queue: &CommandQueue, blocking: bool, offset: usize, src: &'a [T], dst: &'b mut MemBuffer<T>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Self> {
        // SAFETY: Borrow of dst is mutable, so it's safe to write in it
        unsafe { Self::new_by_ref(queue, blocking, offset, src, dst, wait) }
    }
}

impl Event for WriteBuffer<'_, '_> {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
        self.inner.wait()
    }

    #[inline(always)]
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<alloc::vec::Vec<Self::Result>> {
        let iter = iter.into_iter().map(|x| x.inner);
        BaseEvent::wait_all(iter)
    }
}

#[cfg(feature = "async")]
impl futures::Future for WriteBuffer<'_, '_> {
    type Output = Result<()>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.inner).poll(cx)
    }
}

impl AsRef<BaseEvent> for WriteBuffer<'_, '_> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}

/// Event that reads from an OpenCL buffer to host memory
pub struct ReadBuffer<'a, 'b> {
    inner: BaseEvent,
    phtm: PhantomData<(&'a (), &'b ())>
}

impl<'a, 'b> ReadBuffer<'a, 'b> {
    pub fn new<T: Copy + Unpin> (queue: &CommandQueue, blocking: bool, offset: usize, src: &'a MemBuffer<T>, dst: &'b mut [T], wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Self> {
        let wait = wait.into_iter().map(|x| x.as_ref().0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event : cl_event = core::ptr::null_mut();
        let err = unsafe {
            let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            let len = dst.len().checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            clEnqueueReadBuffer(queue.0, src.0, opencl_sys::cl_bool::from(blocking), offset, len, dst.as_mut_ptr().cast(), wait_len, wait, &mut event)
        };

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, phtm: PhantomData });
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidCommandQueue => report.attach_printable(format!("'{:?}' is not a valid command-queue", queue.0)),
                    Error::InvalidContext => report.attach_printable("the context associated with the command queue and buffer are not the same or the context associated with command queue and events in the event wait list are not the same"),
                    Error::InvalidMemObject => report.attach_printable(format!("'{:?}' is not a valid buffer", src.0)),
                    Error::InvalidValue => report.attach_printable("the region being read is out of bounds or ptr is a NULL value"),
                    Error::InvalidEventWaitList => report.attach_printable("event objects in the event wait list are not valid events"),
                    Error::MemObjectAllocationFailure => report.attach_printable("there is a failure to allocate memory for data store associated with buffer"),
                    Error::OutOfResources => report.attach_printable("there is a failure to allocate resources required by the OpenCL implementation on the device"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
}

impl Event for ReadBuffer<'_, '_> {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
        self.inner.wait()
    }

    #[inline(always)]
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<alloc::vec::Vec<Self::Result>> {
        let iter = iter.into_iter().map(|x| x.inner);
        BaseEvent::wait_all(iter)
    }
}

#[cfg(feature = "async")]
impl futures::Future for ReadBuffer<'_, '_> {
    type Output = Result<()>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.inner).poll(cx)
    }
}

impl AsRef<BaseEvent> for ReadBuffer<'_, '_> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}