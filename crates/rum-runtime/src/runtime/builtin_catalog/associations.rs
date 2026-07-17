use super::message_templates::*;
use super::*;

pub(super) const ASSOCIATION_BUILTINS: &[BuiltinSpec] = &[BuiltinSpec::ordinary(
    "MissingQ",
    "Tests whether an expression is a Missing object.",
    Session::eval_missing_q,
)
.with_messages(&[ARGX_EXACT])
.protected()];
