use rum_runtime::kernel_api::{BuiltinSpec, HoldAttribute};

use crate::calculus;

pub(crate) const CALCULUS_BUILTINS: &[BuiltinSpec] = &[BuiltinSpec::ordinary_with_hold(
    "D",
    "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
    HoldAttribute::HoldRest,
    calculus::eval_d,
)];
