use rum_runtime::kernel_api::BuiltinSpec;

use crate::polynomials;

pub(crate) const POLYNOMIAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        polynomials::eval_expand,
    ).protected(),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        polynomials::eval_coefficient,
    ).protected(),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        polynomials::eval_coefficient_list,
    ).protected(),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        polynomials::eval_together,
    ).protected(),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        polynomials::eval_factor,
    ).protected(),
];
