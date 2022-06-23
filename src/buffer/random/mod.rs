#[cfg(test)]
extern crate std;
include!("macro.rs");

use core::{sync::atomic::{AtomicU64, Ordering}};
use std::{time::{SystemTime}};
use alloc::vec::Vec;
use parking_lot::{Mutex};
use crate::{prelude::*, kernel::Kernel, event::various::Swap, buffer::MemFlag};

static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012);
const FAST_MUL : u64 = 0x5DEECE66D;
const ADDEND : u64 = 0xB;
const MASK : u64 = (1 << 48) - 1;

#[cfg(feature = "def")]
lazy_static! {
    static ref RNG : FastRng = FastRng::new(1024).unwrap();
}

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This rng is NOT secure enough for cryptographic purposes
pub struct FastRng {
    seeds: MemBuffer<u64>,
    program: Program,
    rand_byte: Mutex<Kernel>,
    rand_short: Mutex<Kernel>,
    rand_int: Mutex<Kernel>,
    rand_long: Mutex<Kernel>,
    rand_float: Mutex<Kernel>,
    rand_double: Option<Mutex<Kernel>>,
    wait_for: Mutex<Option<BaseEvent>>
}

impl FastRng {
    #[inline(always)]
    pub fn with_context (ctx: &Context, len: usize) -> Result<Self> {        
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as u64;
        
        let mut seeds = Vec::with_capacity(len);
        seeds.push(((Self::seed_uniquifier() ^ now) ^ FAST_MUL) & MASK);

        for i in 1..len {
            let seed = generate_random_u64(seeds[i-1]);
            seeds.push(((Self::seed_uniquifier() ^ seed) ^ FAST_MUL) & MASK);
        }

        Self::with_seeds_context(ctx, &seeds)
    }

    #[inline(always)]
    pub fn with_seeds_context (ctx: &Context, seeds: &[u64]) -> Result<Self> {
        let devices = ctx.devices()?;
        let seeds = MemBuffer::with_context(ctx, MemFlag::default(), seeds)?;
        let program = Program::from_source_with_context(ctx, include_str!("fast_rand.ocl"))?;

        let rand_byte = unsafe { Kernel::new_unchecked(&program, "rand_byte")? };
        let rand_short = unsafe { Kernel::new_unchecked(&program, "rand_short")? };
        let rand_int = unsafe { Kernel::new_unchecked(&program, "rand_int")? };
        let rand_long = unsafe { Kernel::new_unchecked(&program, "rand_long")? };
        let rand_float = unsafe { Kernel::new_unchecked(&program, "rand_float")? };
        let rand_double;

        if devices.iter().all(|x| x.has_f64().unwrap_or(false)) {
            rand_double = Some(unsafe { Kernel::new_unchecked(&program, "rand_double")? });
        } else {
            rand_double = None;
        }
        
        Ok(Self {
            seeds,
            program,
            rand_byte: Mutex::new(rand_byte),
            rand_short: Mutex::new(rand_short),
            rand_int: Mutex::new(rand_int),
            rand_long: Mutex::new(rand_long),
            rand_float: Mutex::new(rand_float),
            rand_double: rand_double.map(Mutex::new),
            wait_for: Mutex::new(None)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    impl_random! {
        rand_byte = u8 as random_u8_with_queue & i8 as random_i8_with_queue,
        rand_short = u16 as random_u16_with_queue & i16 as random_i16_with_queue,
        rand_int = u32 as random_u32_with_queue & i32 as random_i32_with_queue,
        rand_long = u64 as random_u64_with_queue & i64 as random_i64_with_queue
    }

    pub fn random_f32_with_queue (&self, queue: &CommandQueue, min: f32, max: f32, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<f32>, BaseEvent>> {
        assert_ne!(len, 0);

        let seeds_len = self.seeds.len()?;
        let max_wgs = queue.device()?.max_work_group_size()?.get();
        let wgs = seeds_len.min(max_wgs);

        let div = len / seeds_len;
        let rem = len % seeds_len;

        let mut this_wait = self.wait_for.lock();
        let wait_for = this_wait.iter()
            .cloned()
            .chain(wait.into_iter().map(|x| x.as_ref().clone()))
            .collect::<Vec<_>>();

        let out = unsafe { MemBuffer::uninit_with_context(&self.context()?, len, flags)? };
        let mut kernel = self.rand_float.lock();

        let mut wait;
        if div > 0 {
            wait = self.inner_random_float(queue, &mut kernel, &out, min, max, 0, len, wgs, wait_for)?;
            for i in 1..div {
                wait = self.inner_random_float(queue, &mut kernel, &out, min, max, i * seeds_len, len, wgs, [wait])?;
            }

            if rem > 0 {
                wait = self.inner_random_float(queue, &mut kernel, &out, min, max, div * seeds_len, rem, wgs, [wait])?;
            }
        } else {
            wait = self.inner_random_float(queue, &mut kernel, &out, min, max, div * seeds_len, rem, wgs, wait_for)?;
        }

        drop(kernel);
        *this_wait = Some(wait.clone());
        drop(this_wait);
        Ok(wait.swap(out))
    }

    pub fn random_f64_with_queue (&self, queue: &CommandQueue, min: f64, max: f64, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<f64>, BaseEvent>> {
        assert_ne!(len, 0);
        #[cfg(feature = "error-stack")]
        let kernel = self.rand_double.as_ref().ok_or(error_stack::Report::new(Error::InvalidCompilerOptions).attach_printable("Double precision is not supported on this context"))?;
        #[cfg(not(feature = "error-stack"))]
        let kernel = self.rand_double.as_ref().ok_or(Error::InvalidCompilerOptions)?;

        let seeds_len = self.seeds.len()?;
        let max_wgs = queue.device()?.max_work_group_size()?.get();
        let wgs = seeds_len.min(max_wgs);

        let div = len / seeds_len;
        let rem = len % seeds_len;

        let mut this_wait = self.wait_for.lock();
        let wait_for = this_wait.iter()
            .cloned()
            .chain(wait.into_iter().map(|x| x.as_ref().clone()))
            .collect::<Vec<_>>();

        let out = unsafe { MemBuffer::<f64>::uninit_with_context(&self.context()?, len, flags)? };
        let mut kernel = kernel.lock();

        let mut wait;
        if div > 0 {
            wait = self.inner_random_float(queue, &mut kernel, &out, min, max, 0, len, wgs, wait_for)?;
            for i in 1..div {
                wait = self.inner_random_float(queue, &mut kernel, &out, min, max, i * seeds_len, len, wgs, [wait])?;
            }

            if rem > 0 {
                wait = self.inner_random_float(queue, &mut kernel, &out, min, max, div * seeds_len, rem, wgs, [wait])?;
            }
        } else {
            wait = self.inner_random_float(queue, &mut kernel, &out, min, max, div * seeds_len, rem, wgs, wait_for)?;
        }

        drop(kernel);
        *this_wait = Some(wait.clone());
        drop(this_wait);
        Ok(wait.swap(out))
    }

    #[inline]
    fn inner_random<T: Copy + Unpin> (&self, queue: &CommandQueue, kernel: &mut Kernel, out: &MemBuffer<T>, offset: usize, len: usize, wgs: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<BaseEvent> {
        kernel.set_arg(0, len)?;
        kernel.set_arg(1, offset)?;
        kernel.set_mem_arg(2, &self.seeds)?;
        kernel.set_mem_arg(3, out)?;
        kernel.enqueue_with_queue(queue, &[wgs, 1, 1], None, wait)
    }

    #[inline]
    fn inner_random_float<T: Copy + Unpin> (&self, queue: &CommandQueue, kernel: &mut Kernel, out: &MemBuffer<T>, min: T, max: T, offset: usize, len: usize, wgs: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<BaseEvent> {
        kernel.set_arg(0, len)?;
        kernel.set_arg(1, min)?;
        kernel.set_arg(2, max)?;
        kernel.set_arg(3, offset)?;
        kernel.set_mem_arg(4, &self.seeds)?;
        kernel.set_mem_arg(5, out)?;
        kernel.enqueue_with_queue(queue, &[wgs, 1, 1], None, wait)
    }

    #[inline(always)]
    fn seed_uniquifier () -> u64 {
        loop {
            let current = UNIQUIFIER.load(Ordering::Acquire);
            let next = current.wrapping_mul(1181783497276652981);

            if UNIQUIFIER.compare_exchange(current, next, Ordering::Acquire, Ordering::Acquire).is_ok() {
                return next;
            }
        }
    }
}

#[cfg(feature = "def")]
impl FastRng {
    #[inline(always)]
    pub fn new (len: usize) -> Result<Self> {
        Self::with_context(Context::default(), len)
    }

    #[inline(always)]
    pub fn with_seeds (seeds: &[u64]) -> Result<Self> {
        Self::with_seeds_context(Context::default(), seeds)
    }

    #[inline(always)]
    pub fn random_u8 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u8>, BaseEvent>> {
        Self::random_u8_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_i8 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<i8>, BaseEvent>> {
        Self::random_i8_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_u16 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u16>, BaseEvent>> {
        Self::random_u16_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_i16 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<i16>, BaseEvent>> {
        Self::random_i16_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_u32 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u32>, BaseEvent>> {
        Self::random_u32_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_i32 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<i32>, BaseEvent>> {
        Self::random_i32_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_u64 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u64>, BaseEvent>> {
        Self::random_u64_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_i64 (len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<i64>, BaseEvent>> {
        Self::random_i64_with_queue(&RNG, CommandQueue::default(), len, flags, wait)
    }

    #[inline(always)]
    pub fn random_f32 (min: f32, max: f32, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<f32>, BaseEvent>> {
        Self::random_f32_with_queue(&RNG, CommandQueue::default(), min, max, len, flags, wait)
    }

    #[inline(always)]
    pub fn random_f64 (min: f64, max: f64, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<f64>, BaseEvent>> {
        Self::random_f64_with_queue(&RNG, CommandQueue::default(), min, max, len, flags, wait)
    }
}

#[inline(always)]
fn generate_random_u64 (seed: u64) -> u64 {
    (seed.wrapping_mul(FAST_MUL).wrapping_add(ADDEND)) & MASK
}