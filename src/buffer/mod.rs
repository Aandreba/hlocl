flat_mod!(io, flags, r#unsafe);

#[cfg(feature = "serde")]
flat_mod!(ser_de);

#[cfg(feature = "rand")]
flat_mod!(random);