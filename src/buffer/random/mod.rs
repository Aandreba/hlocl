#[cfg(feature = "cl2")]
//flat_mod!(cl2);
flat_mod!(cl1);

#[cfg(not(feature = "cl2"))]
flat_mod!(cl1);