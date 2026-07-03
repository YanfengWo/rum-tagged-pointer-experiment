use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::{combinatorics, integer_digits, number_theory};

pub(crate) const COMBINATORICS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Binomial",
        "Computes exact binomial coefficients for integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_binomial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Fibonacci",
        "Computes exact Fibonacci numbers for integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_fibonacci,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsP",
        "Computes the number of integer partitions by Euler's pentagonal recurrence.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_p,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsQ",
        "Computes the number of partitions into distinct parts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "BernoulliB",
        "Computes exact Bernoulli numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_bernoulli_b,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Subfactorial",
        "Computes derangement numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_subfactorial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS1",
        "Computes signed Stirling numbers of the first kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s1,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS2",
        "Computes Stirling numbers of the second kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s2,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "CatalanNumber",
        "Computes exact Catalan numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_catalan_number,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Multinomial",
        "Computes exact multinomial coefficients for non-negative integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Orderless)
            .with(Attribute::Protected),
        combinatorics::eval_multinomial,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Pochhammer",
        "Computes finite rising factorial products for non-negative integer counts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_pochhammer,
    ),
];

pub(crate) const INTEGER_DIGITS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "IntegerLength",
        "Returns the number of digits in an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_length,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerDigits",
        "Returns the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_digits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerReverse",
        "Reverses the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_integer_reverse,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromDigits",
        "Constructs an exact integer from a list or string of digits.",
        AttributeSet::from_attribute(Attribute::Protected),
        integer_digits::eval_from_digits,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DigitCount",
        "Counts digits in the absolute value of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_digit_count,
    ),
];

pub(crate) const NUMBER_THEORY_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Divisors",
        "Returns the sorted positive divisors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_divisors,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "EulerPhi",
        "Computes Euler's totient function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_euler_phi,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MoebiusMu",
        "Computes the Moebius mu function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_moebius_mu,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DivisorSigma",
        "Computes exact divisor power sums for exact integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NHoldAll)
            .with(Attribute::Protected),
        number_theory::eval_divisor_sigma,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Prime",
        "Returns the n-th positive prime for a positive integer n.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimePi",
        "Counts primes not greater than an exact integer or rational argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime_pi,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeNu",
        "Counts distinct prime factors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_nu,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeOmega",
        "Counts prime factors with multiplicity for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_omega,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "NextPrime",
        "Returns the next or previous signed prime in Wolfram order.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_next_prime,
    ),
];
