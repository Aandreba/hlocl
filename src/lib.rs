#![cfg_attr(not(feature = "std"), no_std)]
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
    pub use crate::error::ErrorCL;
}

pub mod error;
pub mod platform;
pub mod device;
pub mod queue;