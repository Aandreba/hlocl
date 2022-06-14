#[cfg(test)]
extern crate std;

use core::{sync::atomic::{AtomicU64, Ordering}};
use std::{time::{SystemTime}};
use alloc::vec::Vec;
use parking_lot::{Mutex, RwLockReadGuard, lock_api::{RwLock}};

use crate::{prelude::*, kernel::Kernel, event::various::Swap};
use super::MemFlag;

static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012);
const FAST_MUL : u64 = 0x5DEECE66D;
const ADDEND : u64 = 0xB;
const MASK : u64 = (1 << 48) - 1;

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This RNG is not secure enough for cryptographic purposes
pub struct FastRng {
    seeds: RwLock<parking_lot::RawRwLock, MemBuffer<u64>>,
    program: Program,
    rand_byte: Mutex<Kernel>
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
        let seeds = MemBuffer::with_context(ctx, MemFlag::default(), seeds)?;
        let program = Program::from_source_with_context(ctx, include_str!("../kernels/fast_rand.ocl"))?;
        let rand_byte = unsafe { Kernel::new_unchecked(&program, "rand_byte")? };
        
        Ok(Self {
            seeds: RwLock::new(seeds),
            program,
            rand_byte: Mutex::new(rand_byte)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    pub fn random_u8_with_queue (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u8>, BaseEvent>> {
        let mut seeds = self.seeds.read();
        if len > seeds.len()? {
            drop(seeds);
            seeds = self.grow_seeds(queue, len)?;
        }
        
        let mut kernel = self.rand_byte.lock();
        let max_wgs = queue.device()?.max_work_item_dimensions()?.get() as usize;
        let out = unsafe { MemBuffer::<u8>::uninit_with_context(&self.context()?, len, flags)? };
        
        kernel.set_arg(0, len as u64)?;
        kernel.set_mem_arg(1, &seeds)?;
        kernel.set_mem_arg(2, &out)?;
        
        let evt = kernel.enqueue_with_queue(queue, &[len.min(max_wgs), 1, 1], None, wait)?;
        drop((kernel, seeds));
        Ok(evt.swap(out))
    }

    fn grow_seeds (&self, queue: &CommandQueue, next_len: usize) -> Result<RwLockReadGuard<MemBuffer<u64>>> {
        let mut seeds = self.seeds.write();
        let prev_len = seeds.len()?;

        let mut new_seeds = unsafe { MemBuffer::<u64>::uninit_with_context(&self.context()?, next_len, seeds.flags()?)? };
        seeds.copy_to(0, &mut new_seeds, 0..prev_len, EMPTY)?.wait()?;

        let mut prev_seed = seeds.get_with_queue(queue, prev_len - 1, EMPTY)?.wait()?;
        for i in prev_len..next_len {
            let seed = generate_random_u64(prev_seed);
            seeds.set_with_queue(queue, i, ((Self::seed_uniquifier() ^ seed) ^ FAST_MUL) & MASK, EMPTY)?.wait()?;
            prev_seed = seed;
        }

        let seeds = parking_lot::lock_api::RwLockWriteGuard::<'_, parking_lot::RawRwLock, MemBuffer<u64>>::downgrade(seeds);
        Ok(seeds)
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

#[inline(always)]
fn generate_random_u64 (seed: u64) -> u64 {
    (seed * FAST_MUL + ADDEND) & MASK
}