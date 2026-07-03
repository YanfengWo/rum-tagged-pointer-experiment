use super::*;

pub(super) const NUMERIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        builtin_identity,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        Session::eval_plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        Session::eval_times,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Power",
        "Raises an exact number to an integer power when possible.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_power,
    ),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        Session::eval_subtract,
    ),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        Session::eval_divide,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Abs",
        "Returns the absolute value of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_abs,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sign",
        "Returns the sign of exact integers and rationals.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_sign,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Floor",
        "Returns the greatest integer not greater than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_floor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Ceiling",
        "Returns the least integer not less than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_ceiling,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerPart",
        "Returns the integer part of an exact number by truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_integer_part,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FractionalPart",
        "Returns the exact fractional part after truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_fractional_part,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Minus",
        "Negates one exact or symbolic argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        Session::eval_minus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Round",
        "Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_round,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial",
        "Computes factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_factorial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial2",
        "Computes double factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_factorial2,
    ),
    BuiltinSpec::ordinary(
        "Total",
        "Totals list elements through the current exact Plus semantics.",
        Session::eval_total,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Denominator",
        "Returns the denominator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_denominator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Numerator",
        "Returns the numerator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_numerator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_mod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_quotient,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_power_mod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_gcd,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LCM",
        "Computes the least common multiple of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        Session::eval_lcm,
    ),
    BuiltinSpec::ordinary(
        "Max",
        "Returns the maximum exact integer or rational operand.",
        Session::eval_max,
    ),
    BuiltinSpec::ordinary(
        "Min",
        "Returns the minimum exact integer or rational operand.",
        Session::eval_min,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MinMax",
        "Returns the exact minimum and maximum for one scalar or list argument.",
        AttributeSet::from_attribute(Attribute::Protected),
        Session::eval_min_max,
    ),
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        Session::eval_expand,
    ),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        Session::eval_coefficient,
    ),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        Session::eval_coefficient_list,
    ),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        Session::eval_together,
    ),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        Session::eval_factor,
    ),
    BuiltinSpec::ordinary_with_hold(
        "D",
        "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
        HoldAttribute::HoldRest,
        Session::eval_d,
    ),
];
