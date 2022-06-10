#[cfg(test)]
extern crate std;
use cl_sys::{cl_char, cl_uchar, cl_short, cl_ushort, cl_int, cl_uint, cl_long, cl_ulong, cl_float, cl_double};

pub trait MathCL: 'static + Copy + Unpin + Into<Self::C> {
    type C;
    const NAME : &'static str;
}

macro_rules! impl_math {
    ($($ty:ty => $c:ty as $name:literal),+) => {
        $(
            impl MathCL for $ty {
                type C = $c;
                const NAME : &'static str = $name;
            }
        )*
    };
}

#[cfg(not(feature = "half"))]
impl_math! {
    u8 => cl_uchar as "uchar",
    i8 => cl_char as "char",
    u16 => cl_ushort as "ushort",
    i16 => cl_short as "short",
    u32 => cl_uint as "uint",
    i32 => cl_int as "int",
    u64 => cl_ulong as "ulong",
    i64 => cl_long as "long",
    f32 => cl_float as "float",
    f64 => cl_double as "double"
}

#[cfg(feature = "half")]
impl_math! {
    u8 => cl_uchar as "uchar",
    i8 => cl_char as "char",
    u16 => cl_ushort as "ushort",
    i16 => cl_short as "short",
    u32 => cl_uint as "uint",
    i32 => cl_int as "int",
    u64 => cl_ulong as "ulong",
    i64 => cl_long as "long",
    f32 => cl_float as "float",
    f64 => cl_double as "double",
    half::f16 => cl_sys::cl_half as "half",
}