use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::number_theory;

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
