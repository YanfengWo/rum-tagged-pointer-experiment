use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute};

use crate::logic;

pub(crate) const LOGIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_and,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_or,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Nand",
        "Evaluates a negated conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nand,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Nor",
        "Evaluates a negated disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nor,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Xor",
        "Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty()
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        logic::eval_xor,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Equivalent",
        "Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty().with(Attribute::Orderless),
        logic::eval_equivalent,
    ).protected(),
    BuiltinSpec::ordinary(
        "Implies",
        "Evaluates binary logical implication over exact boolean arguments in the current subset.",
        logic::eval_implies,
    ).protected(),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        logic::eval_not,
    ).protected(),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_equal,
    ).protected(),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_unequal,
    ).protected(),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        logic::eval_less,
    ).protected(),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        logic::eval_greater,
    ).protected(),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        logic::eval_less_equal,
    ).protected(),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        logic::eval_greater_equal,
    ).protected(),
    BuiltinSpec::ordinary(
        "LessThan",
        "Represents a unary comparison operator equivalent to Less[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "GreaterThan",
        "Represents a unary comparison operator equivalent to Greater[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "LessEqualThan",
        "Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "GreaterEqualThan",
        "Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "EqualTo",
        "Represents a unary comparison operator equivalent to Equal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "UnequalTo",
        "Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
    BuiltinSpec::ordinary(
        "SameAs",
        "Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
    ).protected(),
];
