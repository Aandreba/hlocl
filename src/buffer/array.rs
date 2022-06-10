use core::ops::RangeBounds;
use alloc::vec::Vec;

use crate::{prelude::{Context, ErrorCL, CommandQueue}, event::{BaseEvent, Event, WriteBuffer, various::Then}};
use super::{UnsafeBuffer, MemFlags, MemBuffer};

#[derive(Clone)]
#[repr(transparent)]
pub struct ArrayBuffer<T: 'static + Copy + Unpin, const N: usize> (MemBuffer<T>);

impl<T: Copy + Unpin, const N: usize> ArrayBuffer<T, N> {
    #[inline(always)]
    pub fn new (ctx: &Context, flags: impl Into<Option<MemFlags>>, v: &[T; N]) -> Result<Self, ErrorCL> {
        MemBuffer::<T>::new(ctx, flags, v).map(Self)
    }

    #[inline(always)]
    pub unsafe fn uninit (ctx: &Context, flags: impl Into<Option<MemFlags>>) -> Result<Self, ErrorCL> {
        MemBuffer::<T>::uninit(ctx, N, flags).map(Self)
    }

    #[inline(always)]
    pub fn len (&self) -> usize {
        N
    }

    #[inline(always)]
    pub fn to_vec<'a> (&self, queue: &CommandQueue, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<impl Event<Result = Vec<T>>, ErrorCL> where T: 'static {
        unsafe { self.read(queue, false, 0, N, wait) }
    }

    #[inline(always)]
    pub fn to_array<'a> (&self, queue: &CommandQueue, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<impl Event<Result = [T;N]>, ErrorCL> where T: 'static {
        let vec = self.to_vec(queue, wait)?;
        Ok(Event::map(vec, |x| unsafe { <[T;N]>::try_from(x).unwrap_unchecked() }))
    }

    #[inline(always)]
    pub fn get<'a> (&self, queue: &CommandQueue, idx: usize, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<impl Event<Result = T>>, ErrorCL> where T: 'static {
        if idx >= N { return Ok(None); }
        unsafe { self.get_unchecked(queue, idx, wait).map(Some) }
    }

    #[inline(always)]
    pub fn set<'a> (&mut self, queue: &CommandQueue, idx: usize, v: T, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Then<WriteBuffer, impl FnOnce(&mut ())>>, ErrorCL> {
        if idx >= N { return Ok(None); }
        unsafe { self.set_unchecked(queue, idx, v, wait).map(Some) }
    }

    #[inline(always)]
    pub fn slice (&self, flags: Option<MemFlags>, range: impl RangeBounds<usize>) -> Result<Option<MemBuffer<T>>, ErrorCL> {
        let offset = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0
        };

        let slice_len = match range.end_bound() {
            core::ops::Bound::Included(&end) => end - offset + 1,
            core::ops::Bound::Excluded(&end) => end - offset,
            core::ops::Bound::Unbounded => N - offset
        };

        if offset + slice_len > N { return Ok(None); }
        unsafe { self.slice_unchecked(flags, offset, slice_len).map(|x| Some(MemBuffer(x))) }
    }
}

impl<T: Copy + Unpin, const N: usize> core::ops::Deref for ArrayBuffer<T, N> {
    type Target = UnsafeBuffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Copy + Unpin, const N: usize> core::ops::DerefMut for ArrayBuffer<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}