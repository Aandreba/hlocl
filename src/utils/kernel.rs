#[cfg(test)]
extern crate std;

use core::ops::DerefMut;
use std::collections::HashMap;
use alloc::{string::{String}, borrow::Cow};
use crate::{kernel::Kernel, prelude::{ErrorCL, Program}};

pub struct ProgramManager<R = parking_lot::RawMutex> where R: lock_api::RawMutex {
    program: Program,
    #[cfg(not(feature = "async"))]
    kernels: HashMap<String, lock_api::Mutex<R, Kernel>>,
    #[cfg(feature = "async")]
    kernels: HashMap<String, lock_api::Mutex<future_parking_lot::mutex::FutureRawMutex<R>, Kernel>>
}

impl<R> ProgramManager<R> where R: parking_lot::lock_api::RawMutex {
    #[inline]
    pub fn new<'a> (program: Program, kernels: impl IntoIterator<Item = impl Into<Cow<'a, str>>>) -> Result<Self, ErrorCL> {
        let kernels = kernels.into_iter().map(|name| {
            let name = name.into().into_owned();
            Kernel::new(&program, &name).map(|k| (name, Mutex::new(k)))
        }).collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Self {
            program,
            kernels
        })
    }

    #[inline(always)]
    pub fn program (&self) -> &Program {
        &self.program
    }

    #[inline(always)]
    pub fn kernel (&self, name: &str) -> Option<MutexGuard<'_, RawMutex, Kernel>> {
        if let Some(kernel) = self.kernels.get(name) {
            todo!()
            //return Some(kernel.lock())
        }

        None
    }

    #[cfg(feature = "async")]
    pub fn lock_async (&self, name: &str) -> Option<FutureLock<'_, RawMutex, Kernel>> {
        if let Some(kernel) = self.kernels.get(name) {
            let lock = kernel.future_lock();
            todo!()
            //return Some(kernel.lock())
        }

        None
    }
}

impl core::ops::Deref for ProgramManager {
    type Target = Program;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl DerefMut for ProgramManager {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.program
    }
}