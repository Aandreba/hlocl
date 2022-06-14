flat_mod!(flags);

use core::{ptr::NonNull, ops::{Deref, DerefMut}, fmt::Debug};
use opencl_sys::{clSVMAllocARM, clSVMFreeARM};
use crate::prelude::{Context};

pub struct SvmBuffer<T: Copy + Unpin> {
    inner: NonNull<T>,
    len: usize,
    ctx: Context
}

impl<T: Copy + Unpin> SvmBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (v: &[T], flags: SvmFlag) -> Option<Self> {
        Self::with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn alloc (len: usize, flags: SvmFlag) -> Option<Self> {
        Self::alloc_with_context(Context::default(), len, flags)
    }

    #[inline]
    pub fn with_context (ctx: &Context, v: &[T], flags: SvmFlag) -> Option<Self> {
        unsafe {
            let this = Self::alloc_with_context(ctx, v.len(), flags)?;
            core::ptr::copy(v.as_ptr(), this.inner.as_ptr(), v.len());
            Some(this)
        }
    }

    #[inline]
    pub unsafe fn alloc_with_context (ctx: &Context, len: usize, flags: SvmFlag) -> Option<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).expect("Buffer too large");
        let align = u32::try_from(core::mem::align_of::<T>()).unwrap();
        let ptr = clSVMAllocARM(ctx.0, flags.bits(), size, align);

        if let Some(ptr) = NonNull::new(ptr) {
            return Some(Self {
                inner: ptr.cast(),
                len,
                ctx: ctx.clone()
            })
        }

        None
    }
}

impl<T: Copy + Unpin + Debug> Debug for SvmBuffer<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: Copy + Unpin> Deref for SvmBuffer<T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.inner.as_ptr(), self.len) }
    }
}

impl<T: Copy + Unpin> DerefMut for SvmBuffer<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.inner.as_ptr(), self.len) }
    }
}

impl<T: Copy + Unpin> Drop for SvmBuffer<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            clSVMFreeARM(self.ctx.0, self.inner.as_ptr().cast())
        }
    }
}