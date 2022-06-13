use core::{ops::RangeBounds, fmt::Debug};
use alloc::vec::Vec;
use crate::{prelude::{Result, Context, CommandQueue}, event::{BaseEvent, Event, WriteBuffer, various::{Swap, Then}, ReadBuffer}, utils::ContextManager};
use super::{UnsafeBuffer, MemFlags};

#[derive(Clone)]
#[repr(transparent)]
pub struct MemBuffer<T: 'static + Copy + Unpin> (pub(super) UnsafeBuffer<T>);

impl<T: Copy + Unpin> MemBuffer<T> {
    #[inline(always)]
    pub fn new (ctx: &Context, flags: impl Into<Option<MemFlags>>, v: &[T]) -> Result<Self> {
        UnsafeBuffer::<T>::new_and_copy(ctx, flags, v).map(Self)
    }

    #[inline(always)]
    pub unsafe fn uninit (ctx: &Context, size: usize, flags: impl Into<Option<MemFlags>>) -> Result<Self> {
        UnsafeBuffer::new(ctx, size, flags).map(Self)
    }

    #[inline(always)]
    pub fn to_vec (&self, queue: &CommandQueue, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Vec<T>, ReadBuffer<'static>>> where T: 'static {
        unsafe { self.read(queue, false, 0, self.len()?, wait) }
    }

    #[inline(always)]
    pub fn get (&self, queue: &CommandQueue, idx: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<impl Event<Result = T>> where T: 'static {
        let len = self.len()?;
        if idx >= len { panic!("Index out of bounds. Tried to access index {idx} of a buffer of size {len}") }
        unsafe { self.get_unchecked(queue, idx, wait) }
    }

    #[inline(always)]
    pub fn set (&mut self, queue: &CommandQueue, idx: usize, v: T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Then<WriteBuffer, impl FnOnce(&mut ())>> {
        let len = self.len()?;
        if idx >= len { panic!("Index out of bounds. Tried to access index {idx} of a buffer of size {len}") }
        unsafe { self.set_unchecked(queue, idx, v, wait) }
    }

    #[inline(always)]
    pub fn slice (&self, range: impl RangeBounds<usize>) -> Result<Self> {
        let len = self.len()?;

        let offset = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0
        };

        let slice_len = match range.end_bound() {
            core::ops::Bound::Included(&end) => end - offset + 1,
            core::ops::Bound::Excluded(&end) => end - offset,
            core::ops::Bound::Unbounded => len - offset
        };

        if offset + slice_len > len { panic!("Index out of bounds") }
        unsafe { self.slice_unchecked(offset, slice_len).map(Self) }
    }
    
    #[inline(always)]
    pub fn get_checked (&self, queue: &CommandQueue, idx: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Option<impl Event<Result = T>>> {
        if idx >= self.len()? { return Ok(None); }
        unsafe { self.get_unchecked(queue, idx, wait).map(Some) }
    }

    #[inline(always)]
    pub fn set_checked (&mut self, queue: &CommandQueue, idx: usize, v: T, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Option<Then<WriteBuffer, impl FnOnce(&mut ())>>> {
        if idx >= self.len()? { return Ok(None); }
        unsafe { self.set_unchecked(queue, idx, v, wait).map(Some) }
    }

    #[inline(always)]
    pub fn slice_checked (&self, range: impl RangeBounds<usize>) -> Result<Option<Self>> {
        let len = self.len()?;

        let offset = match range.start_bound() {
            core::ops::Bound::Included(&start) => start,
            core::ops::Bound::Excluded(&start) => start + 1,
            core::ops::Bound::Unbounded => 0
        };

        let slice_len = match range.end_bound() {
            core::ops::Bound::Included(&end) => end - offset + 1,
            core::ops::Bound::Excluded(&end) => end - offset,
            core::ops::Bound::Unbounded => len - offset
        };

        if offset + slice_len > len { return Ok(None); }
        unsafe { self.slice_unchecked(offset, slice_len).map(|x| Some(Self(x))) }
    }
}

#[cfg(feature = "def")]
impl<T: 'static + Copy + Unpin + Debug> Debug for MemBuffer<T> {
    #[inline(always)]
    fn fmt (&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner = self.to_vec(ContextManager::default().queue(), BaseEvent::empty()).unwrap().wait().unwrap();
        Debug::fmt(&inner, f)
    }
}

impl<T: Copy + Unpin> core::ops::Deref for MemBuffer<T> {
    type Target = UnsafeBuffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Copy + Unpin> core::ops::DerefMut for MemBuffer<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}