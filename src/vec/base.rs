#[cfg(test)]
extern crate std;

use core::ops::{Deref, DerefMut};
use crate::{prelude::{Context, ErrorCL, MemBuffer, Event, BaseEvent, CommandQueue}, buffer::MemFlags, utils::{MathCL}, event::various::Swap};
use super::XArithProgram;
#[cfg(feature = "async")]
use future_parking_lot::mutex::FutureLockable;

#[derive(Clone)]
#[repr(transparent)]
pub struct Vector<T: MathCL> (MemBuffer<T>);

impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub fn new (ctx: &Context, flags: impl Into<Option<MemFlags>>, v: &[T]) -> Result<Self, ErrorCL> {
        MemBuffer::new(ctx, flags, v).map(Self)
    }

    #[inline(always)]
    unsafe fn uninit (ctx: &Context, len: usize, flags: impl Into<Option<MemFlags>>) -> Result<Self, ErrorCL> {
        MemBuffer::uninit(ctx, len, flags).map(Self)
    }

    #[inline(always)]
    pub fn from_buffer (v: MemBuffer<T>) -> Self {
        Self(v)
    }

    #[inline(always)]
    pub fn as_buffer (&self) -> &MemBuffer<T> {
        &self.0
    }
}

// LOCKING CHECKED
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub fn add<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to add vectors of different lengths ({len} v. {other})") }
        unsafe { self.add_unchecked(rhs, queue, flags, len, prog, wait) }
    }
    
    #[inline(always)]
    pub fn add_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.add_unchecked(rhs, queue, flags, len, prog, wait).map(Some) }
    }
}

// LOCKING UNCHECKED
impl<T: MathCL> Vector<T> {
    pub unsafe fn add_unchecked<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len, flags)?;
        let mut kernel = prog.as_ref().add.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }
}

// ASYNC CHECKED
#[cfg(feature = "async")]
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub async fn add_async<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to add vectors of different lengths ({len} v. {other})") }
        unsafe { self.add_async_unchecked(rhs, queue, flags, len, prog, wait).await }
    }
    
    #[inline(always)]
    pub async fn add_async_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.add_async_unchecked(rhs, queue, flags, len, prog, wait).await.map(Some) }
    }
}

// ASYNC UNCHECKED
#[cfg(feature = "async")]
impl<T: MathCL> Vector<T> {
    pub async unsafe fn add_async_unchecked<'a> (&self, rhs: &Self, queue: &CommandQueue, flags: impl Into<Option<MemFlags>>, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len, flags)?;
        let mut kernel = prog.as_ref().add.future_lock().await;

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }
}

impl<T: MathCL> Deref for Vector<T> {
    type Target = MemBuffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 

impl<T: MathCL> DerefMut for Vector<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}