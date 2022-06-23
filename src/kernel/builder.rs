use core::{ptr::addr_of, alloc::Layout};
use alloc::{boxed::Box, alloc::alloc};
use super::Kernel;
use crate::prelude::*;

type Argument = Option<Box<[u8]>>;

pub struct Builder<'a> {
    inner: &'a Kernel,
    args: Box<[Argument]>,
}

impl<'a> Builder<'a> {
    pub fn new (inner: &'a Kernel) -> Result<Self> {
        let arg_count = inner.num_args()? as usize;
        let args;

        unsafe {
            let ptr = alloc(Layout::array::<Argument>(arg_count).unwrap()) as *mut Argument;
            assert!(!ptr.is_null());

            for i in 0..arg_count {
                ptr.add(i).write(None)
            }

            let ptr : *mut [Argument] = core::ptr::from_raw_parts_mut(ptr.cast(), arg_count);
            args = Box::from_raw(ptr);
        }

        Ok(Self {
            inner,
            args
        })
    }

    fn set_arg<T: Copy> (&mut self, idx: u32, v: T) -> &mut Self {
        unsafe {
            let ptr = alloc(Layout::new::<T>());
            assert!(!ptr.is_null());
            core::ptr::copy_nonoverlapping(addr_of!(v).cast(), ptr, core::mem::size_of::<T>());

            let ptr : *mut [u8] = core::ptr::from_raw_parts_mut(ptr.cast(), core::mem::size_of::<T>());
            let v = Box::from_raw(ptr);
            self.args[idx as usize] = Some(v);
        }

        self
    }

    fn set_mem_arg<T: Copy + Unpin> (&mut self, idx: u32, v: &MemBuffer<T>) -> &mut Self {
        self
    }

    fn alloc_arg<T> (&mut self, idx: u32, len: usize) -> &mut Self {
        self
    }

    pub fn enqueue (self) -> Result<()> {
        todo!()
    }
}