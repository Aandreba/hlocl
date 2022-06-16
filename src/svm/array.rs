#[cfg(test)]
extern crate std;

use core::{ptr::NonNull, ops::{Deref, DerefMut}};
use opencl_sys::{clSVMAlloc, clSVMFree};
use core::fmt::Debug;
use crate::prelude::Context;
use super::SvmFlag;

pub struct SvmArray<T, const N: usize> {
    inner: NonNull<T>,
    ctx: Context
}

impl<T, const N: usize> SvmArray<T, N> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (v: [T;N], flags: SvmFlag) -> Option<Self> {
        Self::with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn alloc (flags: SvmFlag) -> Option<Self> {
        Self::alloc_with_context(Context::default(), flags)
    }

    #[inline]
    pub fn with_context (ctx: &Context, v: [T;N], flags: SvmFlag) -> Option<Self> {
        unsafe {
            let this = Self::alloc_with_context(ctx, flags)?;
            core::ptr::copy(v.as_ptr(), this.inner.as_ptr(), N);
            Some(this)
        }
    }

    #[inline]
    pub unsafe fn alloc_with_context (ctx: &Context, flags: SvmFlag) -> Option<Self> {
        #[cfg(debug_assertions)]
        if let Ok(device) = ctx.devices() {
            if !device.into_iter().all(|x| x.version().map(|x| x.major() >= 2).unwrap_or(true)) {
                std::eprintln!("WARNING: Some of the devices inside this context arn't OpenCL 2.0+ compatible. This may cause problems.");
            }
        }

        let size = N.checked_mul(core::mem::size_of::<T>()).expect("Buffer too large");
        let align = u32::try_from(core::mem::align_of::<T>()).unwrap();
        let ptr = clSVMAlloc(ctx.0, flags.bits(), size, align);

        if let Some(ptr) = NonNull::new(ptr) {
            return Some(Self {
                inner: ptr.cast(),
                ctx: ctx.clone()
            })
        }

        None
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *mut T {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_ref (&self) -> &[T;N] {
        unsafe { &*self.inner.as_ptr().cast() }
    }

    #[inline(always)]
    pub fn as_mut (&mut self) -> &mut [T;N] {
        unsafe { &mut *self.inner.as_ptr().cast() }
    }

    #[inline(always)]
    pub fn as_slice (&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.inner.as_ptr(), N) }
    }

    #[inline(always)]
    pub fn as_mut_slice (&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.inner.as_ptr(), N) }
    }
}

impl<T: Debug, const N: usize> Debug for SvmArray<T, N> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T, const N: usize> Deref for SvmArray<T, N> {
    type Target = [T;N];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T, const N: usize> DerefMut for SvmArray<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T, const N: usize> Drop for SvmArray<T, N> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            for i in 0..N {
                core::ptr::drop_in_place(self.inner.as_ptr().add(i))
            }

            clSVMFree(self.ctx.0, self.inner.as_ptr().cast())
        }
    }
}