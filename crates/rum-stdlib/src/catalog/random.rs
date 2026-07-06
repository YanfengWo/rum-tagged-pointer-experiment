use rum_runtime::kernel_api::{BuiltinSpec, Capability, HoldAttribute};

use crate::random;

pub(crate) const RANDOM_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "SeedRandom",
        "Reseeds the deterministic session PRNG from an exact integer, or from host entropy in the zero-argument form.",
        random::eval_seed_random,
    ),
    BuiltinSpec::ordinary(
        "RandomInteger",
        "Draws exact integers uniformly from the deterministic session PRNG.",
        random::eval_random_integer,
    ),
    BuiltinSpec::ordinary(
        "RandomChoice",
        "Chooses elements from a list using the deterministic session PRNG.",
        random::eval_random_choice,
    ),
    BuiltinSpec::ordinary(
        "RandomSample",
        "Samples distinct positions from a list using the deterministic session PRNG.",
        random::eval_random_sample,
    ),
    BuiltinSpec::ordinary(
        "RandomString",
        "Draws a string of characters from an alphabet using the deterministic session PRNG.",
        random::eval_random_string,
    ),
    BuiltinSpec::ordinary_with_hold(
        "BlockRandom",
        "Evaluates an expression while restoring the deterministic session PRNG state afterward.",
        HoldAttribute::HoldAll,
        random::eval_block_random,
    ),
    BuiltinSpec::ordinary(
        "RandomUuid",
        "Returns a reproducible version-4-format UUID string from the deterministic session PRNG.",
        random::eval_random_uuid,
    ),
    BuiltinSpec::ordinary(
        "UuidQ",
        "Tests whether a string has version-4 UUID format and variant bits.",
        random::eval_uuid_q,
    ),
    BuiltinSpec::ordinary(
        "RandomBytes",
        "Returns entropy bytes as a ByteArray using the host entropy provider.",
        random::eval_random_bytes,
    )
    .with_capability(Capability::Random),
    BuiltinSpec::ordinary(
        "RandomPassword",
        "Draws an entropy-backed string of characters from an alphabet.",
        random::eval_random_password,
    )
    .with_capability(Capability::Random),
];
