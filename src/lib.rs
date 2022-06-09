// Ensure we are using core/alloc, so a possible future migration to no_std is easier.
#![cfg_attr(test, no_std)]
#![feature(arc_unwrap_or_clone, vec_into_raw_parts)]
pub(crate) extern crate alloc;

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

macro_rules! tri_panic {
    ($i:expr) => {
        match $i {
            0 => {},
            _ => panic!("{}", crate::error::ErrorCL::from($i))
        }
    };
}

pub mod prelude {
    pub use crate::platform::Platform;
    pub use crate::device::Device;
    pub use crate::context::Context;
    pub use crate::queue::CommandQueue;
    pub use crate::error::ErrorCL;
    pub use crate::program::Program;
    pub use crate::event::{Event, BaseEvent};
    pub use crate::buffer::{MemBuffer, ArrayBuffer};
    pub use crate::kernel::Kernel;
}

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
pub mod vec;
