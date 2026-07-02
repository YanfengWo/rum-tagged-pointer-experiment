use super::*;

pub(super) const NUMERIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        BuiltinHandler::Identity,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Times,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Power",
        "Raises an exact number to an integer power when possible.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Power,
    ),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        BuiltinHandler::Subtract,
    ),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        BuiltinHandler::Divide,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Abs",
        "Returns the absolute value of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Abs,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sign",
        "Returns the sign of exact integers and rationals.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::Sign,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Floor",
        "Returns the greatest integer not greater than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::Floor,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Ceiling",
        "Returns the least integer not less than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::Ceiling,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerPart",
        "Returns the integer part of an exact number by truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::IntegerPart,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FractionalPart",
        "Returns the exact fractional part after truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::FractionalPart,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Minus",
        "Negates one exact or symbolic argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        BuiltinHandler::Minus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Round",
        "Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Round,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial",
        "Computes factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Factorial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial2",
        "Computes double factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Factorial2,
    ),
    BuiltinSpec::ordinary(
        "Total",
        "Totals list elements through the current exact Plus semantics.",
        BuiltinHandler::Total,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Denominator",
        "Returns the denominator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Denominator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Numerator",
        "Returns the numerator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Numerator,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerLength",
        "Returns the number of digits in an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        BuiltinHandler::IntegerLength,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerDigits",
        "Returns the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        BuiltinHandler::IntegerDigits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerReverse",
        "Reverses the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        BuiltinHandler::IntegerReverse,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromDigits",
        "Constructs an exact integer from a list or string of digits.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::FromDigits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DigitCount",
        "Counts digits in the absolute value of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        BuiltinHandler::DigitCount,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Mod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Quotient,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::PowerMod,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::GCD,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LCM",
        "Computes the least common multiple of exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::LCM,
    ),
    BuiltinSpec::ordinary(
        "Max",
        "Returns the maximum exact integer or rational operand.",
        BuiltinHandler::Max,
    ),
    BuiltinSpec::ordinary(
        "Min",
        "Returns the minimum exact integer or rational operand.",
        BuiltinHandler::Min,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MinMax",
        "Returns the exact minimum and maximum for one scalar or list argument.",
        AttributeSet::from_attribute(Attribute::Protected),
        BuiltinHandler::MinMax,
    ),
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        BuiltinHandler::Expand,
    ),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        BuiltinHandler::Coefficient,
    ),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        BuiltinHandler::CoefficientList,
    ),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        BuiltinHandler::Together,
    ),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        BuiltinHandler::Factor,
    ),
    BuiltinSpec::ordinary_with_hold(
        "D",
        "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
        HoldAttribute::HoldRest,
        BuiltinHandler::D,
    ),
];
