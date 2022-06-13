#[cfg(test)]
extern crate std;

use core::fmt::Debug;

use crate::vec::VectorManager;
use crate::prelude::Context;
#[cfg(feature = "half")]
use half::f16;

pub trait MathCL: 'static + Copy + Unpin + Debug {
    const NAME : &'static str;
    const FLOAT : bool;

    #[cfg(feature = "def")]
    fn default_vec_manager () -> &'static VectorManager<Self>;
}

macro_rules! float {
    (f32) => {true};
    (f64) => {true};
    (f16) => {true};
    ($t:ty) => {false}
}

macro_rules! impl_math {
    ($($ty:ident => $c:ident as $name:literal),+) => {
        $(
            impl MathCL for $ty {
                const NAME : &'static str = $name;
                const FLOAT : bool = float!($ty);

                #[cfg(feature = "def")]
                #[inline(always)]
                fn default_vec_manager () -> &'static VectorManager<Self> {
                    &$c
                }
            }

            #[cfg(feature = "def")]
            lazy_static! {
                static ref $c : VectorManager<$ty> = VectorManager::new(Context::default().clone()).unwrap();
            }

            #[cfg(feature = "def")]
            impl VectorManager<$ty> {
                #[inline(always)]
                pub fn default () -> &'static VectorManager<$ty> {
                    &$c
                }
            }
        )*
    };
}

#[cfg(not(feature = "half"))]
impl_math! {
    u8 => UCHAR as "uchar",
    i8 => CHAR as "char",
    u16 => USHORT as "ushort",
    i16 => SHORT as "short",
    u32 => UINT as "uint",
    i32 => INT as "int",
    u64 => ULONG as "ulong",
    i64 => LONG as "long",
    f32 => FLOAT as "float",
    f64 => DOUBLE as "double"
}

#[cfg(feature = "half")]
impl_math! {
    u8 => UCHAR as "uchar",
    i8 => CHAR as "char",
    u16 => USHORT as "ushort",
    i16 => SHORT as "short",
    u32 => UINT as "uint",
    i32 => INT as "int",
    u64 => ULONG as "ulong",
    i64 => LONG as "long",
    f32 => FLOAT as "float",
    f64 => DOUBLE as "double",
    f16 => HALF as "half"
}