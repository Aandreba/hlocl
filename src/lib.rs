#![cfg_attr(not(feature = "std"), no_std)]
#![feature(arc_unwrap_or_clone)]

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
}

pub mod error;
pub mod platform;
pub mod device;
pub mod queue;
pub mod context;
pub mod program;
pub mod buffer;
pub mod event;