use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, Capability, HoldAttribute};

use crate::random;

const ATTR_PROTECTED: AttributeSet = AttributeSet::from_attribute(Attribute::Protected);

pub(crate) const RANDOM_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "SeedRandom",
        "Reseeds the deterministic session PRNG from an exact integer.",
        ATTR_PROTECTED,
        random::eval_seed_random,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomInteger",
        "Draws exact integers uniformly from the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_integer,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomChoice",
        "Chooses elements from a list using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_choice,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomSample",
        "Samples distinct positions from a list using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_sample,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomString",
        "Draws a string of characters from an alphabet using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_string,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "BlockRandom",
        "Evaluates an expression while restoring the deterministic session PRNG state afterward.",
        AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected),
        random::eval_block_random,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomUuid",
        "Returns a reproducible version-4-format UUID string from the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_uuid,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "UuidQ",
        "Tests whether a string has version-4 UUID format and variant bits.",
        ATTR_PROTECTED,
        random::eval_uuid_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "RandomBytes",
        "Returns entropy bytes as a ByteArray using the host entropy provider.",
        ATTR_PROTECTED,
        random::eval_random_bytes,
    )
    .with_capability(Capability::Random),
    BuiltinSpec::ordinary_with_attributes(
        "ReseedRandom",
        "Reseeds the deterministic session PRNG from host entropy. Requires Capability::Random; denied under sandboxed().",
        ATTR_PROTECTED,
        random::eval_reseed_random,
    )
    .with_capability(Capability::Random),
    BuiltinSpec::ordinary_with_attributes(
        "RandomPassword",
        "Draws an entropy-backed password string from an alphabet, optionally enforcing policy constraints.",
        ATTR_PROTECTED,
        random::eval_random_password,
    )
    .with_capability(Capability::Random),
];
