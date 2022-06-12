// Ensure we are using core/alloc, so a possible future migration to no_std is easier.
#![cfg_attr(test, no_std)]
#![allow(incomplete_features)]
#![feature(arc_unwrap_or_clone, vec_into_raw_parts, array_try_map, generic_const_exprs)]
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
            _ => panic!("{}", crate::error::ErrorCL::from($i))
        }
    };
}

#[cfg(feature = "async")]
#[macro_export]
macro_rules! impl_prog {
    ($( #[$ctx_meta:meta] )* $ctx_vis:vis $ctx:ident = {$(
        $( #[$meta:meta] )*
        $vis:vis $id:ident as $name:ident @ $path:literal => $($fun:ident),+
    );+}) => {
        // CONTEXT
        $( #[$ctx_meta] )*
        $ctx_vis struct $ctx<T: $crate::utils::MathCL, R = parking_lot::RawMutex> where R: parking_lot::lock_api::RawMutex {
            ctx: $crate::context::Context,
            $(
                pub $id: $name<T, R>
            ),+
        }

        impl<T: $crate::utils::MathCL, R> $ctx<T, R> where R: parking_lot::lock_api::RawMutex {
            pub fn new (ctx: $crate::prelude::Context) -> Result<Self, $crate::prelude::ErrorCL> {
                $(let $id = $name::new(&ctx)?;)*

                Ok(Self {
                    ctx,
                    $(
                        $id
                    ),+
                })
            }
        }

        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::Deref for $ctx<T, R> {
            type Target = $crate::prelude::Context;
        
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.ctx
            }
        }
        
        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::DerefMut for $ctx<T, R> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.ctx
            }
        }

        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$ctx<T, R>> for $ctx<T, R> {
            #[inline(always)]
            fn as_ref(&self) -> &$ctx<T, R> {
                &self
            }
        }

        $(
            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$name<T, R>> for $ctx<T, R> {
                #[inline(always)]
                fn as_ref(&self) -> &$name<T, R> {
                    &self.$id
                }
            }
        )+

        // PROGRAMS
        $(
            $( #[$meta] )*
            $vis struct $name <T: $crate::utils::MathCL, R = parking_lot::RawMutex> where R: parking_lot::lock_api::RawMutex {
                phtm: core::marker::PhantomData<T>,
                program: $crate::prelude::Program,
                $(
                    pub $fun: parking_lot::lock_api::Mutex<future_parking_lot::mutex::FutureRawMutex<R>, $crate::prelude::Kernel>,
                )*
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> $name<T, R> {
                pub fn new<'a> (ctx: &$crate::prelude::Context) -> Result<Self, $crate::prelude::ErrorCL> {
                    let program = $crate::prelude::Program::from_source(ctx, &std::format!("#define IS_FLOAT {}\ntypedef {} number;\n{}", T::FLOAT, T::NAME, include_str!($path)))?;
                    
                    $(
                        let $fun = $crate::prelude::Kernel::new(&program, stringify!($fun)).map(parking_lot::lock_api::Mutex::new)?;
                    )+

                    Ok(Self {
                        phtm: core::marker::PhantomData,
                        program,
                        $($fun),+
                    })
                }

                #[inline(always)]
                pub fn program (&self) -> &$crate::prelude::Program {
                    &self.program
                }
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$name<T,R>> for $name<T,R> {
                #[inline(always)]
                fn as_ref(&self) -> &$name<T, R> {
                    self
                }
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::Deref for $name<T, R> {
                type Target = $crate::prelude::Program;
            
                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.program
                }
            }
            
            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::DerefMut for $name<T, R> {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.program
                }
            }
        )+
    };
}

#[cfg(not(feature = "async"))]
#[macro_export]
macro_rules! impl_prog {
    ($( #[$ctx_meta:meta] )* $ctx_vis:vis $ctx:ident = {$(
        $( #[$meta:meta] )*
        $vis:vis $id:ident as $name:ident @ $path:literal => $($fun:ident),+
    );+}) => {
        // CONTEXT
        $( #[$ctx_meta] )*
        $ctx_vis struct $ctx<T: $crate::utils::MathCL, R = parking_lot::RawMutex> where R: parking_lot::lock_api::RawMutex {
            ctx: $crate::context::Context,
            $(
                pub $id: $name<T, R>
            ),+
        }

        impl<T: $crate::utils::MathCL, R> $ctx<T, R> where R: parking_lot::lock_api::RawMutex {
            pub fn new (ctx: $crate::prelude::Context) -> Result<Self, $crate::prelude::ErrorCL> {
                $(let $id = $name::new(&ctx)?;)*

                Ok(Self {
                    ctx,
                    $(
                        $id
                    ),+
                })
            }
        }

        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::Deref for $ctx<T, R> {
            type Target = $crate::prelude::Context;
        
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.ctx
            }
        }
        
        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::DerefMut for $ctx<T, R> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.ctx
            }
        }

        impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$ctx<T, R>> for $ctx<T, R> {
            #[inline(always)]
            fn as_ref(&self) -> &$ctx<T, R> {
                &self
            }
        }

        $(
            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$name<T, R>> for $ctx<T, R> {
                #[inline(always)]
                fn as_ref(&self) -> &$name<T, R> {
                    &self.$id
                }
            }
        )+

        // PROGRAMS
        $(
            $( #[$meta] )*
            $vis struct $name <T: $crate::utils::MathCL, R = parking_lot::RawMutex> where R: parking_lot::lock_api::RawMutex {
                phtm: core::marker::PhantomData<T>,
                program: $crate::prelude::Program,
                $(
                    pub $fun: parking_lot::lock_api::Mutex<R, $crate::prelude::Kernel>,
                )*
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> $name<T, R> {
                pub fn new<'a> (ctx: &$crate::prelude::Context) -> Result<Self, $crate::prelude::ErrorCL> {
                    let program = $crate::prelude::Program::from_source(ctx, &std::format!("#define IS_FLOAT {}\ntypedef {} number;\n{}", T::FLOAT, T::NAME, include_str!($path)))?;
                    
                    $(
                        let $fun = $crate::prelude::Kernel::new(&program, stringify!($fun)).map(parking_lot::lock_api::Mutex::new)?;
                    )+

                    Ok(Self {
                        phtm: core::marker::PhantomData,
                        program,
                        $($fun),+
                    })
                }

                #[inline(always)]
                pub fn program (&self) -> &$crate::prelude::Program {
                    &self.program
                }
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::convert::AsRef<$name<T,R>> for $name<T,R> {
                #[inline(always)]
                fn as_ref(&self) -> &$name<T, R> {
                    self
                }
            }

            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::Deref for $name<T, R> {
                type Target = $crate::prelude::Program;
            
                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.program
                }
            }
            
            impl<T: $crate::utils::MathCL, R: parking_lot::lock_api::RawMutex> core::ops::DerefMut for $name<T, R> {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.program
                }
            }
        )+
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
    pub use crate::buffer::{MemBuffer};
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
