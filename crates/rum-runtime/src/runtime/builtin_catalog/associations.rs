use super::*;

pub(super) const ASSOCIATION_BUILTINS: &[BuiltinSpec] = &[BuiltinSpec::ordinary(
    "MissingQ",
    "Tests whether an expression is a Missing object.",
    Session::eval_missing_q,
)
.protected()];
