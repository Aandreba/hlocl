#[cfg(test)]
extern crate std;

use crate::vec::VectorManager;
use crate::prelude::Context;

pub trait MathCL: 'static + Copy + Unpin {
    const NAME : &'static str;

    #[cfg(feature = "def")]
    fn default_vec_manager () -> &'static VectorManager<Self>;
}

macro_rules! impl_math {
    ($($ty:ty => $c:ident as $name:literal),+) => {
        $(
            impl MathCL for $ty {
                const NAME : &'static str = $name;

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
    f64 => DOUBLE as "double"
    half::f16 => HALF as "half",
}