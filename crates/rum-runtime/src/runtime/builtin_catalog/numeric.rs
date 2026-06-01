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
    BuiltinSpec::ordinary(
        "Power",
        "Raises an exact number to an integer power when possible.",
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
    BuiltinSpec::ordinary(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        BuiltinHandler::Mod,
    ),
    BuiltinSpec::ordinary(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        BuiltinHandler::Quotient,
    ),
    BuiltinSpec::ordinary(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        BuiltinHandler::PowerMod,
    ),
    BuiltinSpec::ordinary(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        BuiltinHandler::GCD,
    ),
    BuiltinSpec::ordinary(
        "LCM",
        "Computes the least common multiple of exact integers.",
        BuiltinHandler::LCM,
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
