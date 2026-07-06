use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::arithmetic;

pub(crate) const ARITHMETIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        arithmetic::eval_identity,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        arithmetic::eval_plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        arithmetic::eval_times,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Power",
        "Raises an exact number to an integer power when possible.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_power,
    ),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        arithmetic::eval_subtract,
    ),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        arithmetic::eval_divide,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Abs",
        "Returns the absolute value of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_abs,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sign",
        "Returns the sign of exact integers and rationals.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_sign,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Floor",
        "Returns the greatest integer not greater than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_floor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Ceiling",
        "Returns the least integer not less than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_ceiling,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerPart",
        "Returns the integer part of an exact number by truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_integer_part,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FractionalPart",
        "Returns the exact fractional part after truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_fractional_part,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Minus",
        "Negates one exact or symbolic argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_minus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Round",
        "Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_round,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial",
        "Computes factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_factorial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial2",
        "Computes double factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_factorial2,
    ),
    BuiltinSpec::ordinary(
        "Total",
        "Totals list elements through the current exact Plus semantics.",
        arithmetic::eval_total,
    ),
    BuiltinSpec::ordinary(
        "Mean",
        "Aggregates a non-empty list as Total[list] divided by Length[list] through exact arithmetic.",
        arithmetic::eval_mean,
    ),
    BuiltinSpec::ordinary(
        "Median",
        "Returns the middle element of a canonically sorted non-empty list, averaging the two middle values exactly for even lengths.",
        arithmetic::eval_median,
    ),
    BuiltinSpec::ordinary(
        "Commonest",
        "Returns the most frequent elements of a list in first-occurrence order, including all ties.",
        arithmetic::eval_commonest,
    ),
    BuiltinSpec::ordinary(
        "Quantile",
        "Returns the nearest-rank quantile of a non-empty list using Ceiling[q * n] clamped to 1..n, with q exact in [0, 1].",
        arithmetic::eval_quantile,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Denominator",
        "Returns the denominator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_denominator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Numerator",
        "Returns the numerator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_numerator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_mod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_quotient,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_power_mod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_gcd,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LCM",
        "Computes the least common multiple of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_lcm,
    ),
    BuiltinSpec::ordinary(
        "Max",
        "Returns the maximum exact integer or rational operand.",
        arithmetic::eval_max,
    ),
    BuiltinSpec::ordinary(
        "Min",
        "Returns the minimum exact integer or rational operand.",
        arithmetic::eval_min,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MinMax",
        "Returns the exact minimum and maximum for one scalar or list argument.",
        AttributeSet::from_attribute(Attribute::Protected),
        arithmetic::eval_min_max,
    ),
];
