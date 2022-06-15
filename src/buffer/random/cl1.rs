#[cfg(test)]
extern crate std;

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
        
        Ok(Self {
            seeds,
            program,
            rand_byte: Mutex::new(rand_byte),
            wait_for: Mutex::new(None)
        })
    }

    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.program.context()
    }

    pub fn random_u8_with_queue (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<u8>, BaseEvent>> {
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
        let mut kernel = self.rand_byte.lock();

        let mut wait;
        if div > 0 {
            wait = self.inner_random_u8(queue, &mut kernel, &out, 0, len, wgs, wait_for)?;
            for i in 1..div {
                wait = self.inner_random_u8(queue, &mut kernel, &out, i * seeds_len, len, wgs, [wait])?;
            }

            if rem > 0 {
                wait = self.inner_random_u8(queue, &mut kernel, &out, div * seeds_len, rem, wgs, [wait])?;
            }
        } else {
            wait = self.inner_random_u8(queue, &mut kernel, &out, div * seeds_len, rem, wgs, wait_for)?;
        }

        drop(kernel);
        *this_wait = Some(wait.clone());
        drop(this_wait);
        Ok(wait.swap(out))
    }

    pub fn random_i8_with_queue (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<impl Event<Output = MemBuffer<i8>>> {
        let evt = self.random_u8_with_queue(queue, len, flags, wait)?;
        todo!()
    }

    #[inline]
    fn inner_random_u8 (&self, queue: &CommandQueue, kernel: &mut Kernel, out: &MemBuffer<u8>, offset: usize, len: usize, wgs: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<BaseEvent> {
        kernel.set_arg(0, len)?;
        kernel.set_arg(1, offset)?;
        kernel.set_mem_arg(2, &self.seeds)?;
        kernel.set_mem_arg(3, out)?;
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

#[inline(always)]
fn generate_random_u64 (seed: u64) -> u64 {
    (seed.wrapping_mul(FAST_MUL).wrapping_add(ADDEND)) & MASK
}

macro_rules! impl_random {
    ($($kernel:ident = $s:ident as $sf:ident & $u:ident as $uf:ident => $fun:ident),+) => {
        $(
            pub fn $uf (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<$u>, BaseEvent>> {
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
                let mut kernel = self.$kernel.lock();
        
                let mut wait;
                if div > 0 {
                    wait = self.$fun(queue, &mut kernel, &out, 0, len, wgs, wait_for)?;
                    for i in 1..div {
                        wait = self.$fun(queue, &mut kernel, &out, i * seeds_len, len, wgs, [wait])?;
                    }
        
                    if rem > 0 {
                        wait = self.$fun(queue, &mut kernel, &out, div * seeds_len, rem, wgs, [wait])?;
                    }
                } else {
                    wait = self.$fun(queue, &mut kernel, &out, div * seeds_len, rem, wgs, wait_for)?;
                }
        
                drop(kernel);
                *this_wait = Some(wait.clone());
                drop(this_wait);
                Ok(wait.swap(out))
            }

            pub fn $sf (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<$s>, BaseEvent>> {
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
        
                let out = unsafe { MemBuffer::<$u>::uninit_with_context(&self.context()?, len, flags)? };
                let mut kernel = self.$kernel.lock();
        
                let mut wait;
                if div > 0 {
                    wait = self.$fun(queue, &mut kernel, &out, 0, len, wgs, wait_for)?;
                    for i in 1..div {
                        wait = self.$fun(queue, &mut kernel, &out, i * seeds_len, len, wgs, [wait])?;
                    }
        
                    if rem > 0 {
                        wait = self.$fun(queue, &mut kernel, &out, div * seeds_len, rem, wgs, [wait])?;
                    }
                } else {
                    wait = self.$fun(queue, &mut kernel, &out, div * seeds_len, rem, wgs, wait_for)?;
                }
        
                drop(kernel);
                *this_wait = Some(wait.clone());
                drop(this_wait);

                let out = unsafe { out.transmute() };
                Ok(wait.swap(out))
            }

            #[inline]
            fn $fun (&self, queue: &CommandQueue, kernel: &mut Kernel, out: &MemBuffer<$u>, offset: usize, len: usize, wgs: usize, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<BaseEvent> {
                kernel.set_arg(0, len)?;
                kernel.set_arg(1, offset)?;
                kernel.set_mem_arg(2, &self.seeds)?;
                kernel.set_mem_arg(3, out)?;
                kernel.enqueue_with_queue(queue, &[wgs, 1, 1], None, wait)
            }
        )*
    };
}