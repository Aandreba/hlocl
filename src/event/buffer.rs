use core::{borrow::Borrow, pin::Pin, marker::PhantomData};
use alloc::{vec::Vec};
use cl_sys::{cl_event, clEnqueueWriteBuffer, clEnqueueCopyBuffer, clEnqueueReadBuffer};
use crate::{prelude::{ErrorCL, CommandQueue}, buffer::{UnsafeBuffer}};
use super::{BaseEvent, Event};

/// OpenCL event that reads from one buffer to another 
#[derive(Clone)]
pub struct CopyBuffer<T: Copy + Unpin> {
    inner: BaseEvent,
    dst: UnsafeBuffer<T>
}

impl<T: Copy + Unpin> CopyBuffer<T> {
    pub unsafe fn new<'a> (queue: &CommandQueue, src_offset: usize, dst_offset: usize, len: usize, src: &UnsafeBuffer<T>, dst: UnsafeBuffer<T>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Self, ErrorCL> {
        let wait = wait.into_iter().map(|x| x.0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event : cl_event = core::ptr::null_mut();
        let err = clEnqueueCopyBuffer(queue.0, src.0, dst.0, src_offset, dst_offset, len, wait_len, wait, &mut event);

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, dst });
        }

        Err(ErrorCL::from(err))
    }
}

impl<T: Copy + Unpin> Event for CopyBuffer<T> {
    type Result = UnsafeBuffer<T>;

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, ErrorCL> {
        self.inner.wait()?;
        Ok(self.dst)
    }
}

#[cfg(feature = "async")]
impl<T: Copy + Unpin> futures::Future for CopyBuffer<T> {
    type Output = Result<UnsafeBuffer<T>, ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let core::task::Poll::Ready(_) = core::pin::Pin::new(&mut self.inner).poll(cx)? {
            return core::task::Poll::Ready(Ok(self.dst.clone()))
        }

        core::task::Poll::Pending
    }
}

impl<T: Copy + Unpin> AsRef<BaseEvent> for CopyBuffer<T> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}

/// Event that writes from host memory to an OpenCL buffer
#[derive(Clone)]
pub struct WriteBuffer<'a> {
    inner: BaseEvent,
    phtm: PhantomData<&'a ()>
}

impl<'a> WriteBuffer<'a> {
    pub unsafe fn new<'b, T: Copy + Unpin> (queue: &CommandQueue, blocking: bool, offset: usize, src: &'a [T], dst: &mut UnsafeBuffer<T>, wait: impl IntoIterator<Item = &'b BaseEvent>) -> Result<Self, ErrorCL> {
        let src = Pin::new(src);

        let wait = wait.into_iter().map(|x| x.borrow().0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event : cl_event = core::ptr::null_mut();
        let err = {
            let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            let len = src.len().checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            clEnqueueWriteBuffer(queue.0, dst.0, cl_sys::cl_bool::from(blocking), offset, len, src.as_ptr().cast(), wait_len, wait, &mut event)
        };

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, phtm: PhantomData });
        }

        Err(ErrorCL::from(err))
    }
}

impl Event for WriteBuffer<'_> {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, ErrorCL> {
        self.inner.wait()
    }
}

#[cfg(feature = "async")]
impl futures::Future for WriteBuffer<'_> {
    type Output = Result<(), ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.inner).poll(cx)
    }
}

impl AsRef<BaseEvent> for WriteBuffer<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}

/// Event that reads from an OpenCL buffer to host memory
pub struct ReadBuffer<'a> {
    inner: BaseEvent,
    phtm: PhantomData<&'a ()>
}

impl<'a> ReadBuffer<'a> {
    pub unsafe fn new<'b, T: Copy + Unpin> (queue: &CommandQueue, blocking: bool, offset: usize, src: &UnsafeBuffer<T>, dst: &'a mut [T], wait: impl IntoIterator<Item = &'b BaseEvent>) -> Result<Self, ErrorCL> {
        let wait = wait.into_iter().map(|x| x.0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event : cl_event = core::ptr::null_mut();
        let err = {
            let offset = offset.checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            let len = dst.len().checked_mul(core::mem::size_of::<T>()).expect("Integer overflow. Too many elements in buffer");
            clEnqueueReadBuffer(queue.0, src.0, cl_sys::cl_bool::from(blocking), offset, len, dst.as_mut_ptr().cast(), wait_len, wait, &mut event)
        };

        if err == 0 {
            let inner = BaseEvent::new(event)?;
            return Ok(Self { inner, phtm: PhantomData });
        }

        Err(ErrorCL::from(err))
    }
}

impl<'a> Event for ReadBuffer<'a> {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, ErrorCL> {
        self.inner.wait()
    }
}

#[cfg(feature = "async")]
impl<'a> futures::Future for ReadBuffer<'a> {
    type Output = Result<(), ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.inner).poll(cx)
    }
}

impl AsRef<BaseEvent> for ReadBuffer<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.inner
    }
}