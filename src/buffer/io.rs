#[cfg(test)]
extern crate std;

use std::io::{Read, Write};
use alloc::vec::Vec;
use crate::{prelude::{Result, Context, Error, CommandQueue, Event, BaseEvent}};
use super::{MemBuffer, MemFlag};
#[cfg(feature = "async")]
use futures::{AsyncReadExt, AsyncWriteExt};

impl<T: Copy + Unpin> MemBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn from_io<R: ?Sized + Read> (flags: MemFlag, src: &mut R) -> Result<Self> {
        Self::from_io_with_context(Context::default(), flags, src)
    }

    pub unsafe fn from_io_with_context<R: ?Sized + Read> (ctx: &Context, flags: MemFlag, src: &mut R) -> Result<Self> {
        let mut buff = Vec::<u8>::with_capacity(core::mem::size_of::<T>());
        let read = src.read_to_end(&mut buff);
        #[cfg(feature = "error-stack")]
        read.map_err(|e| error_stack::Report::new(Error::MemObjectAllocationFailure).attach_printable(e))?;
        #[cfg(not(feature = "error-stack"))]
        read.map_err(|_| Error::MemObjectAllocationFailure)?; 

        if buff.len() % core::mem::size_of::<T>() != 0 {
            #[cfg(feature = "error-stack")]
            return Err(error_stack::Report::new(Error::InvalidBufferSize).attach_printable("Buffer size is not a multiple of the element size"));
            #[cfg(not(feature = "error-stack"))]
            return Err(Error::InvalidBufferSize);
        }

        let buff = core::slice::from_raw_parts(buff.as_ptr().cast(), buff.len() / core::mem::size_of::<T>());
        Self::with_context(ctx, flags, &buff)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn write_into<W: ?Sized + Write> (&self, dst: &mut W, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<()> {
        self.write_into_with_queue(CommandQueue::default(), dst, wait)
    }

    pub fn write_into_with_queue<W: ?Sized + Write> (&self, queue: &CommandQueue, dst: &mut W, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<()> {
        let vec = self.to_vec_with_queue(queue, wait)?.wait()?;
        let bytes = unsafe { core::slice::from_raw_parts::<u8>(vec.as_ptr().cast(), vec.len().checked_mul(core::mem::size_of::<T>()).unwrap()) };
        let write = dst.write_all(bytes);

        #[cfg(feature = "error-stack")]
        write.map_err(|e| error_stack::Report::new(Error::MemObjectAllocationFailure).attach_printable(e))?;
        #[cfg(not(feature = "error-stack"))]
        write.map_err(|_| Error::MemObjectAllocationFailure)?;

        Ok(())
    }
}

#[cfg(feature = "async")]
impl<T: 'static + Copy + Unpin> MemBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub async unsafe fn from_io_async<R: ?Sized + Unpin + AsyncReadExt> (flags: MemFlag, src: &mut R) -> Result<Self> {
        Self::from_io_with_context_async(Context::default(), flags, src).await
    }

    pub async unsafe fn from_io_with_context_async<R: ?Sized + Unpin + AsyncReadExt> (ctx: &Context, flags: MemFlag, src: &mut R) -> Result<Self> {
        let mut buff = Vec::<u8>::with_capacity(core::mem::size_of::<T>());
        let read = src.read_to_end(&mut buff).await;

        #[cfg(feature = "error-stack")]
        read.map_err(|e| error_stack::Report::new(Error::MemObjectAllocationFailure).attach_printable(e))?;
        #[cfg(not(feature = "error-stack"))]
        read.map_err(|_| Error::MemObjectAllocationFailure)?; 

        if buff.len() % core::mem::size_of::<T>() != 0 {
            #[cfg(feature = "error-stack")]
            return Err(error_stack::Report::new(Error::InvalidBufferSize).attach_printable("Buffer size is not a multiple of the element size"));
            #[cfg(not(feature = "error-stack"))]
            return Err(Error::InvalidBufferSize);
        }

        let buff = core::slice::from_raw_parts(buff.as_ptr().cast(), buff.len() / core::mem::size_of::<T>());
        Self::with_context(ctx, flags, &buff)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn write_into_async<W: ?Sized + Write> (&self, dst: &mut W, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<()> {
        self.write_into_with_queue(CommandQueue::default(), dst, wait)
    }

    pub async fn write_into_with_queue_async<W: ?Sized + Unpin + AsyncWriteExt> (&self, queue: &CommandQueue, dst: &mut W, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<()> {
        let vec = self.to_vec_with_queue(queue, wait)?.await?;
        let bytes = unsafe { core::slice::from_raw_parts::<u8>(vec.as_ptr().cast(), vec.len().checked_mul(core::mem::size_of::<T>()).unwrap()) };
        let write = dst.write_all(bytes).await;

        #[cfg(feature = "error-stack")]
        write.map_err(|e| error_stack::Report::new(Error::MemObjectAllocationFailure).attach_printable(e))?;
        #[cfg(not(feature = "error-stack"))]
        write.map_err(|_| Error::MemObjectAllocationFailure)?;

        Ok(())
    }
}