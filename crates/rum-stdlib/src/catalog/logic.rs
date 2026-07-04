use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute};

use crate::logic;

pub(crate) const LOGIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_and,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_or,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nand",
        "Evaluates a negated conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nand,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Nor",
        "Evaluates a negated disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Xor",
        "Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty()
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        logic::eval_xor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Equivalent",
        "Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty().with(Attribute::Orderless),
        logic::eval_equivalent,
    ),
    BuiltinSpec::ordinary(
        "Implies",
        "Evaluates binary logical implication over exact boolean arguments in the current subset.",
        logic::eval_implies,
    ),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        logic::eval_not,
    ),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_equal,
    ),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_unequal,
    ),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        logic::eval_less,
    ),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        logic::eval_greater,
    ),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        logic::eval_less_equal,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        logic::eval_greater_equal,
    ),
    BuiltinSpec::ordinary(
        "LessThan",
        "Represents a unary comparison operator equivalent to Less[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterThan",
        "Represents a unary comparison operator equivalent to Greater[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "LessEqualThan",
        "Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqualThan",
        "Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "EqualTo",
        "Represents a unary comparison operator equivalent to Equal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "UnequalTo",
        "Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
    BuiltinSpec::ordinary(
        "SameAs",
        "Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ),
];
