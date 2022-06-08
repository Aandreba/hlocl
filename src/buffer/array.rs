use alloc::vec;
use cl_sys::size_t;
use crate::{prelude::{Context, ErrorCL, CommandQueue}, event::{BaseEvent, Event, WriteBuffer}};
use super::{UnsafeBuffer, MemFlags};

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ArrayBuffer<T: Copy + Unpin, const N: size_t> (UnsafeBuffer<T>);

impl<T: Copy + Unpin, const N: size_t> ArrayBuffer<T, N> {
    #[inline(always)]
    pub fn new (ctx: &Context, flags: Option<MemFlags>, v: [T; N]) -> Result<Self, ErrorCL> {
        UnsafeBuffer::<T>::new_and_copy(ctx, flags, &v).map(Self)
    }

    #[inline(always)]
    pub unsafe fn uninit (ctx: &Context, flags: Option<MemFlags>) -> Result<Self, ErrorCL> {
        UnsafeBuffer::<T>::new(ctx, N, flags).map(Self)
    }

    #[inline(always)]
    pub fn get<'a> (&self, queue: &CommandQueue, idx: size_t, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<impl Event<Result = T>>, ErrorCL> where T: 'static {
        if idx >= N { return Ok(None); }
        unsafe { self.get_unchecked(queue, idx, wait).map(Some) }
    }

    #[inline(always)]
    pub unsafe fn get_unchecked<'a> (&self, queue: &CommandQueue, idx: size_t, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<impl Event<Result = T>, ErrorCL> where T: 'static {
        let evt = self.0.read(queue, false, idx, 1, wait)?;
        Ok(Event::then(evt, |x| x[0]))
    }

    #[inline(always)]
    pub fn set<'a> (&self, queue: &CommandQueue, idx: size_t, v: T, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<WriteBuffer<T>>, ErrorCL> {
        if idx >= N { return Ok(None); }
        unsafe { self.set_unchecked(queue, idx, v, wait).map(Some) }
    }

    #[inline(always)]
    pub unsafe fn set_unchecked<'a> (&self, queue: &CommandQueue, idx: size_t, v: T, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<WriteBuffer<T>, ErrorCL> {
        self.0.write(queue, false, idx, vec![v], wait)
    }
}

impl<T: Copy + Unpin, const N: size_t> core::ops::Deref for ArrayBuffer<T, N> {
    type Target = UnsafeBuffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}