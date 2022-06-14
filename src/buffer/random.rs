#[cfg(test)]
extern crate std;

use core::sync::atomic::{AtomicU64, Ordering};
use std::{time::{SystemTime}, println};
use parking_lot::Mutex;

use crate::{prelude::*, kernel::Kernel, event::various::Swap};
use super::MemFlag;

static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012);
const FAST_MUL : u64 = 0x5DEECE66D;
const MASK : u64 = (1 << 48) - 1;

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This RNG is not secure enough for cryptographic purposes
pub struct FastRng {
    seed: MemBuffer<u64>,
    program: Program,
    rand_byte: Mutex<Kernel>
}

impl FastRng {
    #[inline(always)]
    pub fn with_context (ctx: &Context) -> Result<Self> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as u64;
        Self::with_seed_context(ctx, Self::seed_uniquifier() ^ now)
    }

    #[inline(always)]
    pub fn with_seed_context (ctx: &Context, seed: u64) -> Result<Self> {
        let seed = MemBuffer::with_context(ctx, MemFlag::default(), &[(seed ^ FAST_MUL) & MASK])?;
        let program = Program::from_source_with_context(ctx, include_str!("../kernels/fast_rand.ocl"))?;
        let rand_byte = unsafe { Kernel::new_unchecked(&program, "rand_byte")? };
        
        Ok(Self {
            seed,
            program,
            rand_byte: Mutex::new(rand_byte)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    pub fn random_u8_with_queue (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u8>, BaseEvent>> {
        let mut kernel = self.rand_byte.lock();
        let max_wgs = queue.device()?.max_work_item_dimensions()?.get() as usize;
        let out = unsafe { MemBuffer::<u8>::uninit_with_context(&self.context()?, len, flags)? };
        
        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, &self.seed)?;
        kernel.set_mem_arg(2, &out)?;
        
        let evt = kernel.enqueue_with_queue(queue, &[len.min(max_wgs), 1, 1], None, wait)?;
        drop(kernel);
        Ok(evt.swap(out))
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