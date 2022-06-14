#[cfg(test)]
extern crate std;

use core::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime};
use crate::{prelude::{Program, Context}, kernel::Kernel};

static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012);
const FAST_MUL : u64 = 0x5DEECE66D;
const MASK : u64 = (1 << 48) - 1;

/// Random number generator based on Java's ```Random``` class.
/// # Warning
/// This RNG is not secure enough for cryptographic purposes
pub struct FastRng {
    seed: AtomicU64,
    program: Program,
    rand_int: Kernel
}

impl FastRng {
    #[inline(always)]
    pub fn new () -> Self {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as u64;
        Self::with_seed(Self::seed_uniquifier() ^ now)
    }

    #[inline(always)]
    pub const fn with_seed_context (ctx: &Context, seed: u64) -> Self {
        let seed = (seed ^ FAST_MUL) & MASK;
        let program = Program::from_source_with_context(ctx, source)
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