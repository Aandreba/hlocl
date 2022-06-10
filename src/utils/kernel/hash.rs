#[cfg(test)]
extern crate std;

use core::ops::DerefMut;
use std::collections::HashMap;
use alloc::{string::{String}, borrow::Cow};
use parking_lot::{RawMutex, lock_api::{Mutex, MutexGuard}};
use crate::{kernel::Kernel, prelude::{ErrorCL, Program}};

#[cfg(feature = "async")]
use future_parking_lot::mutex::{FutureLock, FutureLockable};

pub struct HashProgramManager<R = RawMutex> where R: parking_lot::lock_api::RawMutex {
    program: Program,
    #[cfg(not(feature = "async"))]
    kernels: HashMap<String, Mutex<R, Kernel>>,
    #[cfg(feature = "async")]
    kernels: HashMap<String, Mutex<future_parking_lot::mutex::FutureRawMutex<R>, Kernel>>
}

impl<R> HashProgramManager<R> where R: parking_lot::lock_api::RawMutex {
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

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            #[inline(always)]
            pub fn kernel (&self, name: &str) -> Option<MutexGuard<'_, future_parking_lot::mutex::FutureRawMutex<R>, Kernel>> {
                if let Some(kernel) = self.kernels.get(name) {
                    return Some(kernel.lock())
                }

                None
            }

            #[cfg(feature = "async")]
            pub fn kernel_async (&self, name: &str) -> Option<FutureLock<'_, R, Kernel>> {
                if let Some(kernel) = self.kernels.get(name) {
                    return Some(kernel.future_lock())
                }

                None
            }
        } else {
            #[inline(always)]
            pub fn kernel (&self, name: &str) -> Option<MutexGuard<'_, R, Kernel>> {
                if let Some(kernel) = self.kernels.get(name) {
                    return Some(kernel.lock())
                }

                None
            }
        }
    }
}

impl core::ops::Deref for HashProgramManager {
    type Target = Program;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl DerefMut for HashProgramManager {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.program
    }
}