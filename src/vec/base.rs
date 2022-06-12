#[cfg(test)]
extern crate std;

use core::{ops::{Deref, DerefMut}, mem::MaybeUninit};
use crate::{prelude::{Context, ErrorCL, MemBuffer, Event, BaseEvent, CommandQueue}, buffer::MemFlags, utils::{MathCL, ContextManager}, event::various::Swap};
use super::{XArithProgram, XHozProgram};
use cl_sys::libc::tm;
#[cfg(feature = "async")]
use future_parking_lot::mutex::FutureLockable;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Vector<T: MathCL> (MemBuffer<T>);

impl<T: MathCL> Vector<T> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (v: &[T]) -> Result<Self, ErrorCL> {
        Self::with_context(Context::default(), v)
    }

    #[inline(always)]
    pub fn with_context (ctx: &Context, v: &[T]) -> Result<Self, ErrorCL> {
        MemBuffer::new(ctx, None, v).map(Self)
    }

    #[inline(always)]
    unsafe fn uninit (ctx: &Context, len: usize) -> Result<Self, ErrorCL> {
        MemBuffer::uninit(ctx, len, None).map(Self)
    }

    #[inline(always)]
    pub unsafe fn from_buffer (v: MemBuffer<T>) -> Self {
        debug_assert!(v.flags().unwrap().contains(MemFlags::READ_WRITE));
        Self(v)
    }

    #[inline(always)]
    pub fn as_buffer (&self) -> &MemBuffer<T> {
        &self.0
    }
}

// LOCKING CHECKED BY VALUE
#[cfg(feature = "def")]
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub fn add (&self, rhs: &Self) -> Result<Self, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::add_event(self, rhs, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }

    #[inline(always)]
    pub fn sub (&self, rhs: &Self) -> Result<Self, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::sub_event(self, rhs, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }

    #[inline(always)]
    pub fn mul (&self, rhs: &Self) -> Result<Self, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::mul_event(self, rhs, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }

    #[inline(always)]
    pub fn div (&self, rhs: &Self) -> Result<Self, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::div_event(self, rhs, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }

    #[inline(always)]
    pub fn mul_add (&self, rhs: &Self, add: &Self) -> Result<Self, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::mul_add_event(self, rhs, add, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }

    #[inline(always)]
    pub fn sum (&self) -> Result<T, ErrorCL> {
        let ctx = ContextManager::default();
        Vector::<T>::sum_event(&self, ctx.queue(), T::default_vec_manager(), BaseEvent::empty())?.wait()
    }
}

// LOCKING CHECKED BY EVENT
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub fn add_event (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to add vectors of different lengths ({len} v. {other})") }
        unsafe { self.add_unchecked(rhs, queue, len, prog, wait) }
    }

    #[inline(always)]
    pub fn sub_event (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to subtract vectors of different lengths ({len} v. {other})") }
        unsafe { self.sub_unchecked(rhs, queue, len, prog, wait) }
    }

    #[inline(always)]
    pub fn mul_event (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to multiply vectors of different lengths ({len} v. {other})") }
        unsafe { self.mul_unchecked(rhs, queue, len, prog, wait) }
    }

    #[inline(always)]
    pub fn div_event (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to divide vectors of different lengths ({len} v. {other})") }
        unsafe { self.div_unchecked(rhs, queue, len, prog, wait) }
    }

    #[inline(always)]
    pub fn mul_add_event (&self, rhs: &Self, add: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;

        let other = rhs.len()?;
        if len != other { panic!("Tried to multiply vectors of different lengths ({len} v. {other})") }

        let other = add.len()?;
        if len != other { panic!("Tried to add vectors of different lengths ({len} v. {other})") }

        unsafe { self.mul_add_unchecked(rhs, add, queue, len, prog, wait) }
    }

    // TODO FIX
    pub fn sum_event (&self, queue: &CommandQueue, prog: impl AsRef<XHozProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<impl Event<Result = T>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let ctx = queue.context()?;
        let len = self.len()?;

        let next_pow_2 = len.next_power_of_two();
        let tmp_size = 1 + len / max_wg_size;
        let tmp = unsafe { MemBuffer::<T>::uninit(&ctx, tmp_size, None)? };

        let mut kernel = prog.sum.lock();
        unsafe {
            kernel.set_arg(0, len as u64)?;
            kernel.set_mem_arg(1, self)?;
            kernel.set_mem_arg(2, &tmp)?;
            kernel.alloc_arg::<T>(3, next_pow_2)?;
        }
        
        let sum = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);

        sum.wait()?;
        panic!("{:?}", tmp);
        
        if tmp_size > 1 || next_pow_2 != len {
            let result = unsafe { MemBuffer::<T>::uninit(&ctx, 1, MemFlags::WRITE_ONLY)? };
            let mut kernel = prog.sum_epilogue.lock();

            // Epilogue
            unsafe {
                kernel.set_mem_arg(0, &tmp)?;
                kernel.set_mem_arg(1, &result)?;
                kernel.alloc_arg::<T>(2, tmp_size)?;
            }

            let epilogue = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, [sum])?;
            drop(kernel);

            // Result
            return unsafe { result.get_unchecked(queue, 0, [epilogue]) }
        }
        
        unsafe { tmp.get_unchecked(queue, 0, [sum]) }
    }
}
    
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub fn add_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.add_unchecked(rhs, queue, len, prog, wait).map(Some) }
    }

    #[inline(always)]
    pub fn sub_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.sub_unchecked(rhs, queue, len, prog, wait).map(Some) }
    }

    #[inline(always)]
    pub fn mul_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.mul_unchecked(rhs, queue, len, prog, wait).map(Some) }
    }

    #[inline(always)]
    pub fn div_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.div_unchecked(rhs, queue, len, prog, wait).map(Some) }
    }

    #[inline(always)]
    pub fn mul_add_checked<'a> (&self, rhs: &Self, add: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? || len != add.len()? { return Ok(None); }
        unsafe { self.mul_add_unchecked(rhs, add, queue, len, prog, wait).map(Some) }
    }
}

// LOCKING UNCHECKED
impl<T: MathCL> Vector<T> {
    pub unsafe fn add_unchecked (&self, rhs: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
        let mut kernel = prog.add.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }

    pub unsafe fn sub_unchecked (&self, rhs: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
        let mut kernel = prog.sub.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }

    pub unsafe fn mul_unchecked (&self, rhs: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
        let mut kernel = prog.mul.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }

    pub unsafe fn div_unchecked (&self, rhs: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
        let mut kernel = prog.div.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, self)?;
        kernel.set_mem_arg(3, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }

    pub unsafe fn mul_add_unchecked (&self, rhs: &Self, add: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
        let mut kernel = prog.mul_add.lock();

        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, rhs)?;
        kernel.set_mem_arg(2, add)?;
        kernel.set_mem_arg(3, self)?;
        kernel.set_mem_arg(4, &result)?;
        
        let event = kernel.enqueue(queue, &[max_wg_size.min(len), 1, 1], None, wait)?;
        drop(kernel);
        
        Ok(event.swap(result))
    }
}

// ASYNC CHECKED
#[cfg(feature = "async")]
impl<T: MathCL> Vector<T> {
    #[inline(always)]
    pub async fn add_async<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let len = self.len()?;
        let other = rhs.len()?;

        if len != other { panic!("Tried to add vectors of different lengths ({len} v. {other})") }
        unsafe { self.add_async_unchecked(rhs, queue, len, prog, wait).await }
    }
    
    #[inline(always)]
    pub async fn add_async_checked<'a> (&self, rhs: &Self, queue: &CommandQueue, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Option<Swap<Self, BaseEvent>>, ErrorCL> {
        let len = self.len()?;
        if len != rhs.len()? { return Ok(None); }
        unsafe { self.add_async_unchecked(rhs, queue, len, prog, wait).await.map(Some) }
    }
}

// ASYNC UNCHECKED
#[cfg(feature = "async")]
impl<T: MathCL> Vector<T> {
    pub async unsafe fn add_async_unchecked<'a> (&self, rhs: &Self, queue: &CommandQueue, len: impl Into<Option<usize>>, prog: impl AsRef<XArithProgram<T>>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<Swap<Self, BaseEvent>, ErrorCL> {
        let prog = prog.as_ref();
        let max_wg_size = queue.device()?.max_work_group_size()?.get();

        let len = match len.into() {
            Some(x) => x,
            None => self.len()?
        };

        let result = Self::uninit(&prog.context()?, len)?;
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

// VECTOR-VECTOR ADDITION
cfg_if::cfg_if! {
    if #[cfg(feature = "def")] {
        impl<T: MathCL> core::ops::Add for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn add (self, rhs: Self) -> Self::Output {
                Vector::<T>::add(self, rhs).unwrap()
            }
        }

        impl<T: MathCL> core::ops::Add<Vector<T>> for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn add (self, rhs: Vector<T>) -> Self::Output {
                self + &rhs
            }
        }

        impl<T: MathCL> core::ops::Add<&'_ Vector<T>> for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn add (self, rhs: &Self) -> Self::Output {
                &self + rhs
            }
        }

        impl<T: MathCL> core::ops::Add for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn add (self, rhs: Self) -> Self::Output {
                &self + &rhs
            }
        }
    }
}

// VECTOR-VECTOR SUBTRACTION
cfg_if::cfg_if! {
    if #[cfg(feature = "def")] {
        impl<T: MathCL> core::ops::Sub for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn sub (self, rhs: Self) -> Self::Output {
                Vector::<T>::sub(self, rhs).unwrap()
            }
        }

        impl<T: MathCL> core::ops::Sub<Vector<T>> for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn sub (self, rhs: Vector<T>) -> Self::Output {
                self - &rhs
            }
        }

        impl<T: MathCL> core::ops::Sub<&'_ Vector<T>> for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn sub (self, rhs: &Self) -> Self::Output {
                &self - rhs
            }
        }

        impl<T: MathCL> core::ops::Sub for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn sub (self, rhs: Self) -> Self::Output {
                &self - &rhs
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "def")] {
        use num_traits::MulAdd;

        impl<T: MathCL> MulAdd for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: Self, rhs2: Self) -> Self::Output {
                Vector::<T>::mul_add(self, rhs, rhs2).unwrap()
            }
        }

        impl<T: MathCL> MulAdd<Vector<T>, &'_ Vector<T>> for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: Vector<T>, add: &Vector<T>) -> Self::Output {
                MulAdd::mul_add(self, &rhs, add)
            }
        }

        impl<T: MathCL> MulAdd<&'_ Vector<T>, Vector<T>> for &'_ Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: &Vector<T>, add: Vector<T>) -> Self::Output {
                MulAdd::mul_add(self, rhs, &add)
            }
        }

        impl<T: MathCL> MulAdd<Vector<T>, &'_ Vector<T>> for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: Vector<T>, add: &Vector<T>) -> Self::Output {
                MulAdd::mul_add(&self, &rhs, add)
            }
        }

        impl<T: MathCL> MulAdd<&'_ Vector<T>, Vector<T>> for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: &Vector<T>, add: Vector<T>) -> Self::Output {
                MulAdd::mul_add(&self, rhs, &add)
            }
        }

        impl<T: MathCL> MulAdd for Vector<T> {
            type Output = Vector<T>;
        
            #[inline(always)]
            fn mul_add (self, rhs: Vector<T>, add: Vector<T>) -> Self::Output {
                MulAdd::mul_add(&self, &rhs, &add)
            }
        }
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