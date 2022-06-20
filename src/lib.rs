// Ensure we are using core/alloc, so a possible future migration to no_std is easier.
#![cfg_attr(test, no_std)]
#![allow(incomplete_features)]
#![feature(arc_unwrap_or_clone, vec_into_raw_parts, array_try_map, generic_const_exprs, ptr_metadata, core_c_str, alloc_c_string)]

pub(crate) extern crate alloc;

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

macro_rules! lazy_static {
    ($($vis:vis static ref $name:ident : $ty:ty = $expr:expr;)+) => {
        $(
            $vis static $name : ::once_cell::sync::Lazy<$ty> = ::once_cell::sync::Lazy::new(|| $expr);
        )+
    };
}

macro_rules! tri_panic {
    ($i:expr) => {
        match $i {
            0 => {},
            _ => {
                panic!("{}", $crate::error::Error::from($i));
            }
        }
    };
}

pub mod prelude {
    pub use crate::platform::Platform;
    pub use crate::device::Device;
    pub use crate::context::Context;
    pub use crate::queue::CommandQueue;
    pub use crate::error::{Result, Error};
    pub use crate::program::Program;
    pub use crate::event::{Event, BaseEvent, EMPTY};
    pub use crate::buffer::{MemBuffer};
    pub use crate::kernel::Kernel;
}

/// OpenCL errors
pub mod error;
pub mod platform;
pub mod device;
pub mod queue;
pub mod context;
pub mod program;
pub mod buffer;
pub mod event;
pub mod kernel;
pub mod utils;

#[cfg(feature = "cl2")]
pub mod svm;
//pub mod vec;
