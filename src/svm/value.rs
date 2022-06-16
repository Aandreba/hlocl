#[cfg(test)]
extern crate std;

use core::{ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Display, alloc::Layout, mem::ManuallyDrop, ffi::CStr};
use alloc::{string::String, vec::Vec};
use opencl_sys::{clSVMAlloc, clSVMFree};
use core::fmt::Debug;
use crate::{prelude::Context};
use super::SvmFlag;

pub struct SvmValue<T: ?Sized> {
    inner: NonNull<T>,
    ctx: Context
}

impl<T> SvmValue<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (v: T, flags: SvmFlag) -> Option<Self> {
        Self::with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub unsafe fn alloc (flags: SvmFlag) -> Option<Self> {
        Self::alloc_with_context(Context::default(), flags)
    }

    #[inline]
    pub fn with_context (ctx: &Context, v: T, flags: SvmFlag) -> Option<Self> {
        unsafe {
            let this = Self::alloc_with_context(ctx, flags)?;
            *this.inner.as_ptr() = v;
            Some(this)
        }
    }

    #[inline]
    pub unsafe fn alloc_with_context (ctx: &Context, flags: SvmFlag) -> Option<Self> {
        let size = core::mem::size_of::<T>();
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
}

impl<T: ?Sized> SvmValue<T> {
    #[inline]
    pub unsafe fn alloc_with_layout_context (ctx: &Context, layout: Layout, flags: SvmFlag) -> Option<Self> {
        let size = layout.size();
        let align = u32::try_from(layout.align()).unwrap();
        let ptr = clSVMAlloc(ctx.0, flags.bits(), size, align);

        if let Some(ptr) = NonNull::new(ptr) {
            let ptr = core::ptr::from_raw_parts::<T>(ptr.as_ptr().cast(), core::mem::zeroed());

            return Some(Self {
                inner: NonNull::new_unchecked(ptr as *mut _),
                ctx: ctx.clone()
            })
        }

        None
    }

    #[inline(always)]
    pub unsafe fn write (&mut self, value: &T) {
        self.inner = NonNull::new_unchecked(value as *const _ as *mut _);
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *mut T {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_ref (&self) -> &T {
        unsafe { self.inner.as_ref() }
    }

    #[inline(always)]
    pub fn as_mut (&mut self) -> &mut T {
        unsafe { self.inner.as_mut() }
    }
}

impl SvmValue<str> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_str (v: &str, flags: SvmFlag) -> Option<Self> {
        Self::from_str_with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_string (v: String, flags: SvmFlag) -> Option<Self> {
        Self::from_string_with_context(Context::default(), v, flags)
    }

    #[inline(always)]
    pub fn from_str_with_context (ctx: &Context, v: &str, flags: SvmFlag) -> Option<Self> {
        unsafe {
            let mut alloc = Self::alloc_with_layout_context(ctx, Layout::for_value(v), flags)?;
            alloc.write(v);
            Some(alloc)
        }
    }

    #[inline(always)]
    pub fn from_string_with_context (ctx: &Context, v: String, flags: SvmFlag) -> Option<Self> {
        Self::from_str_with_context(ctx, &v, flags)
    }
}

impl SvmValue<CStr> {    
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_cstr (v: &CStr, flags: SvmFlag) -> Option<Self> {
        Self::from_cstr_with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_cstring (v: alloc::ffi::CString, flags: SvmFlag) -> Option<Self> {
        Self::from_cstring_with_context(Context::default(), v, flags)
    }

    #[inline(always)]
    pub fn from_cstr_with_context (ctx: &Context, v: &CStr, flags: SvmFlag) -> Option<Self> {
        unsafe {
            let mut alloc = Self::alloc_with_layout_context(ctx, Layout::for_value(v), flags)?;
            alloc.write(v);
            Some(alloc)
        }
    }

    #[inline(always)]
    pub fn from_cstring_with_context (ctx: &Context, v: alloc::ffi::CString, flags: SvmFlag) -> Option<Self> {
        Self::from_cstr_with_context(ctx, &v, flags)
    }
}

impl<T> SvmValue<[T]> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_slice (v: &[T], flags: SvmFlag) -> Option<Self> where T: Copy {
        Self::from_slice_with_context(Context::default(), v, flags)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_vec (v: Vec<T>, flags: SvmFlag) -> Option<Self> {
        Self::from_vec_with_context(Context::default(), v, flags)
    }

    #[inline(always)]
    pub fn from_slice_with_context (ctx: &Context, v: &[T], flags: SvmFlag) -> Option<Self> where T: Copy {
        unsafe {
            let mut alloc = Self::alloc_with_layout_context(ctx, Layout::for_value(v), flags)?;
            alloc.write(v);
            Some(alloc)
        }
    }

    #[inline(always)]
    pub fn from_vec_with_context (ctx: &Context, v: Vec<T>, flags: SvmFlag) -> Option<Self> {
        let me = ManuallyDrop::new(v);
        let v = me.as_slice();

        unsafe {
            let mut alloc = Self::alloc_with_layout_context(ctx, Layout::for_value(v), flags)?;
            alloc.write(v);
            Some(alloc)
        }
    }
}

impl<T: ?Sized + Debug> Debug for SvmValue<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: ?Sized + Display> Display for SvmValue<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl<T: ?Sized> Deref for SvmValue<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> DerefMut for SvmValue<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized> Drop for SvmValue<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.inner.as_ptr());
            clSVMFree(self.ctx.0, self.inner.as_ptr().cast())
        }
    }
}