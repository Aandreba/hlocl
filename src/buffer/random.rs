#[cfg(test)]
extern crate std;

use core::{sync::atomic::{AtomicU64, Ordering}};
use std::{time::{SystemTime}};
use alloc::vec::Vec;
use parking_lot::{Mutex};

use crate::{prelude::*, kernel::Kernel, event::various::Swap, svm::{SvmBuffer, SvmFlag}};
use super::MemFlag;

static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012);
const FAST_MUL : u64 = 0x5DEECE66D;
const ADDEND : u64 = 0xB;
const MASK : u64 = (1 << 48) - 1;

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This RNG is not secure enough for cryptographic purposes
pub struct FastRng {
    seed: SvmBuffer<u64>,
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
    pub fn with_seeds_context (ctx: &Context, seed: &[u64]) -> Result<Self> {
        let seeds = SvmBuffer::with_context(ctx, seeds, SvmFlag::READ_WRITE | SvmFlag::FINE_GRAIN_BUFFER | SvmFlag::ATOMICS).unwrap();
        let program = Program::from_source_with_context(ctx, include_str!("../kernels/fast_rand.ocl"))?;
        let rand_byte = unsafe { Kernel::new_unchecked(&program, "rand_byte")? };
        
        Ok(Self {
            seeds,
            program,
            rand_byte: Mutex::new(rand_byte)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    pub fn random_u8_with_queue (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u8>, BaseEvent>> {
        todo!()
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