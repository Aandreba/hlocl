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

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This RNG is not secure enough for cryptographic purposes
pub struct FastRng {
    seeds: MemBuffer<u64>,
    program: Program,
    rand_byte: Mutex<Kernel>,
    rand_short: Mutex<Kernel>,
    rand_int: Mutex<Kernel>,
    rand_long: Mutex<Kernel>,
    rand_float: Mutex<Kernel>,
    rand_double: Mutex<Kernel>,
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
        let seeds = MemBuffer::with_context(ctx, MemFlag::default(), seeds)?;
        let program = Program::from_source_with_context(ctx, include_str!("../../kernels/fast_rand_cl1.ocl"))?;

        let rand_byte = unsafe { Kernel::new_unchecked(&program, "rand_byte")? };
        let rand_short = unsafe { Kernel::new_unchecked(&program, "rand_short")? };
        let rand_int = unsafe { Kernel::new_unchecked(&program, "rand_int")? };
        let rand_long = unsafe { Kernel::new_unchecked(&program, "rand_long")? };
        let rand_float = unsafe { Kernel::new_unchecked(&program, "rand_float")? };
        let rand_double = unsafe { Kernel::new_unchecked(&program, "rand_double")? };
        
        Ok(Self {
            seeds,
            program,
            rand_byte: Mutex::new(rand_byte),
            rand_short: Mutex::new(rand_short),
            rand_int: Mutex::new(rand_int),
            rand_long: Mutex::new(rand_long),
            rand_float: Mutex::new(rand_float),
            rand_double: Mutex::new(rand_double),
            wait_for: Mutex::new(None)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    impl_random! {
        rand_byte = u8 as random_u8_with_queue & i8 as random_i8_with_queue => inner_random_u8,
        rand_short = u16 as random_u16_with_queue & i16 as random_i16_with_queue => inner_random_u16,
        rand_int = u32 as random_u32_with_queue & i32 as random_i32_with_queue => inner_random_u32,
        rand_long = u64 as random_u64_with_queue & i64 as random_i64_with_queue => inner_random_u64,
        rand_float = f32 as random_f32_with_queue => inner_random_f32,
        rand_double = f64 as random_f64_with_queue => inner_random_f64
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
    (seed.wrapping_mul(FAST_MUL).wrapping_add(ADDEND)) & MASK
}