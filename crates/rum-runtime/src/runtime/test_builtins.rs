use super::*;

macro_rules! test_arithmetic_handler {
    ($name:ident, $builtin:ident) => {
        fn $name(session: &mut Session, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
            session.eval_arithmetic_data_builtin(ArithmeticDataBuiltin::$builtin, head, args)
        }
    };
}

test_arithmetic_handler!(test_arithmetic_identity, Identity);
test_arithmetic_handler!(test_arithmetic_plus, Plus);
test_arithmetic_handler!(test_arithmetic_times, Times);
test_arithmetic_handler!(test_arithmetic_power, Power);
test_arithmetic_handler!(test_arithmetic_subtract, Subtract);
test_arithmetic_handler!(test_arithmetic_divide, Divide);
test_arithmetic_handler!(test_arithmetic_abs, Abs);
test_arithmetic_handler!(test_arithmetic_sign, Sign);
test_arithmetic_handler!(test_arithmetic_floor, Floor);
test_arithmetic_handler!(test_arithmetic_ceiling, Ceiling);
test_arithmetic_handler!(test_arithmetic_integer_part, IntegerPart);
test_arithmetic_handler!(test_arithmetic_fractional_part, FractionalPart);
test_arithmetic_handler!(test_arithmetic_minus, Minus);
test_arithmetic_handler!(test_arithmetic_round, Round);
test_arithmetic_handler!(test_arithmetic_factorial, Factorial);
test_arithmetic_handler!(test_arithmetic_factorial2, Factorial2);
test_arithmetic_handler!(test_arithmetic_total, Total);
test_arithmetic_handler!(test_arithmetic_denominator, Denominator);
test_arithmetic_handler!(test_arithmetic_numerator, Numerator);
test_arithmetic_handler!(test_arithmetic_mod, Mod);
test_arithmetic_handler!(test_arithmetic_quotient, Quotient);
test_arithmetic_handler!(test_arithmetic_power_mod, PowerMod);
test_arithmetic_handler!(test_arithmetic_gcd, GCD);
test_arithmetic_handler!(test_arithmetic_lcm, LCM);
test_arithmetic_handler!(test_arithmetic_max, Max);
test_arithmetic_handler!(test_arithmetic_min, Min);
test_arithmetic_handler!(test_arithmetic_min_max, MinMax);

pub(in crate::runtime) fn register_test_arithmetic_builtins(session: &mut Session) {
    let specs = [
        BuiltinSpec::ordinary(
            "Identity",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_identity,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Plus",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::Orderless)
                .with(Attribute::Flat)
                .with(Attribute::OneIdentity),
            test_arithmetic_plus,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Times",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::Orderless)
                .with(Attribute::Flat)
                .with(Attribute::OneIdentity),
            test_arithmetic_times,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Power",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_power,
        ),
        BuiltinSpec::ordinary(
            "Subtract",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_subtract,
        ),
        BuiltinSpec::ordinary(
            "Divide",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_divide,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Abs",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_abs,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Sign",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_sign,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Floor",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_floor,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Ceiling",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_ceiling,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "IntegerPart",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_integer_part,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "FractionalPart",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_fractional_part,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Minus",
            "Test-local moved arithmetic builtin.",
            AttributeSet::empty()
                .with(Attribute::Listable)
                .with(Attribute::NumericFunction)
                .with(Attribute::Protected),
            test_arithmetic_minus,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Round",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_round,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Factorial",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_factorial,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Factorial2",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_factorial2,
        ),
        BuiltinSpec::ordinary(
            "Total",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_total,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Denominator",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_denominator,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Numerator",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_numerator,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Mod",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_mod,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Quotient",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_quotient,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "PowerMod",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_power_mod,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "GCD",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_gcd,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "LCM",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Listable),
            test_arithmetic_lcm,
        ),
        BuiltinSpec::ordinary(
            "Max",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_max,
        ),
        BuiltinSpec::ordinary(
            "Min",
            "Test-local moved arithmetic builtin.",
            test_arithmetic_min,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "MinMax",
            "Test-local moved arithmetic builtin.",
            AttributeSet::from_attribute(Attribute::Protected),
            test_arithmetic_min_max,
        ),
    ];

    for spec in specs {
        session.register_builtin(spec);
    }
}
