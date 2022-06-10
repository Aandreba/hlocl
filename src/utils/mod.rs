flat_mod!(ctx, kernel, math);

// Generic conditionals
pub(crate) trait IsTrue {}
pub(crate) struct GenericConstr<const F: bool>;
impl IsTrue for GenericConstr<true> {}
