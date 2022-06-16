#[cfg(test)]
extern crate std;

flat_mod!(array, value, flags);
use core::{ptr::NonNull, ops::{Deref, DerefMut}, fmt::Debug};
use opencl_sys::{clSVMAlloc, clSVMFree};
use crate::prelude::{Context};
use alloc::vec::Vec;

pub struct SvmBuffer<T> {
    inner: NonNull<T>,
    len: usize,
    ctx: Context
}

impl<T> SvmBuffer<T> {
    #[inline(always)]
    pub fn with_context (ctx: &Context, v: Vec<T>, flags: SvmFlag) -> Option<Self> {
        unsafe {
            let buff = Self::alloc_with_context(ctx, v.len(), flags)?;
            let me = core::mem::ManuallyDrop::new(v);
            core::ptr::copy_nonoverlapping(me.as_ptr(), buff.as_ptr(), me.len());
            Some(buff)
        }
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn alloc (len: usize, flags: SvmFlag) -> Option<Self> {
        Self::alloc_with_context(Context::default(), len, flags)
    }

    #[inline]
    pub unsafe fn alloc_with_context (ctx: &Context, len: usize, flags: SvmFlag) -> Option<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).expect("Buffer too large");
        let align = u32::try_from(core::mem::align_of::<T>()).unwrap();
        let ptr = clSVMAlloc(ctx.0, flags.bits(), size, align);

        if let Some(ptr) = NonNull::new(ptr) {
            return Some(Self {
                inner: ptr.cast(),
                len,
                ctx: ctx.clone()
            })
        }

        None
    }

    #[inline(always)]
    pub fn len (&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *mut T {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_ref (&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.inner.as_ptr(), self.len) }
    }

    #[inline(always)]
    pub fn as_mut (&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.inner.as_ptr(), self.len) }
    }
}

impl<T: Copy> SvmBuffer<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_slice (v: &[T], flags: SvmFlag) -> Option<Self> {
        Self::from_slice_with_context(Context::default(), v, flags)
    }

    #[inline]
    pub fn from_slice_with_context (ctx: &Context, v: &[T], flags: SvmFlag) -> Option<Self> {
        unsafe {
            let this = Self::alloc_with_context(ctx, v.len(), flags)?;
            core::ptr::copy(v.as_ptr(), this.inner.as_ptr(), v.len());
            Some(this)
        }
    }
}

impl<T: Copy + Debug> Debug for SvmBuffer<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: Copy> Deref for SvmBuffer<T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: Copy> DerefMut for SvmBuffer<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Drop for SvmBuffer<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                core::ptr::drop_in_place(self.inner.as_ptr().add(i))
            }

            clSVMFree(self.ctx.0, self.inner.as_ptr().cast())
        }
    }
}