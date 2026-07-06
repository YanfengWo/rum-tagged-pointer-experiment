# Brief 160 — Feature-gating skeleton; gate the digest family behind `hash`

Implementation brief. Establishes the Cargo-feature mechanism in
`rum-stdlib` (policy: `ARCHITECTURE.md` §No unwanted dependencies) and
applies it to the first case: the digest builtins, so a user who does not
want hashing can drop `sha2`/`sha1`/`md-5`/`crc32fast` entirely. Behavior
under default features is UNCHANGED.

## The feature

- Add to `crates/rum-stdlib/Cargo.toml`:
  ```
  [features]
  default = ["hash"]
  hash = ["dep:sha2", "dep:sha1", "dep:md-5", "dep:crc32fast"]
  ```
  and make those four dependencies `optional = true` (they are workspace
  deps — use `sha2 = { workspace = true, optional = true }` etc.).

## What is gated (digests only — NOT the whole byte family)

The `bytes` family currently mixes external-dep digests with self-
implemented encodings. Gate ONLY the digest builtins; keep the rest
always-on.

- `#[cfg(feature = "hash")]`: `Sha256`, `Sha512`, `Sha1`, `Md5`, `Crc32`
  — their implementations, catalog entries, `install_core` registration,
  and tests.
- ALWAYS ON (self-implemented, zero-dep): `ToHexString`, `FromHexString`,
  `ToBase64`, `FromBase64`, `ByteArrayQ`, `ByteArrayToString`.
- Split the module and catalog array accordingly (e.g. a `digest`
  submodule under `bytes/` gated as a whole; `BYTE_BUILTINS` becomes the
  always-on set, plus a `#[cfg(feature = "hash")] DIGEST_BUILTINS` appended
  in `install_core` under cfg).

## Kernel vocabulary interaction (important)

The kernel attribute vocabulary (`rum-runtime`
`stdlib_attributes.rs`) is NOT feature-aware and lists the digest names.
The `catalog_attributes_match_kernel_vocabulary` test cross-checks catalog
vs vocabulary. Under `--no-default-features` the catalog loses the digest
entries while the vocabulary keeps them, which would break a bidirectional
check. Resolve by making that test assert the feature-robust invariant:
**every INSTALLED catalog builtin has a matching vocabulary entry** (catalog
⊆ vocabulary), which holds under any feature set; the vocabulary MAY be a
superset (it documents the full names). If the test currently asserts
equality both ways, relax the vocabulary-⊆-catalog direction (or cfg-gate
that direction to default features) and say what you did. Do NOT feature-
gate the kernel vocabulary itself.

## Reference / drift

The generated reference and its drift test are defined at DEFAULT features
(hash on) — the full catalog. `cargo test --workspace` runs with default
features, so the drift test sees the digests and passes unchanged; the
reference file is not regenerated (no content change). Add a one-line note
in `docs/reference/README.md` that the reference reflects the default
feature set.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# default features: digests present
printf '' | cargo run -q -p rum-cli -- --eval 'Sha256["abc"]'
# no-default-features: crate builds, digest crates NOT linked, digests absent
cargo build -p rum-stdlib --no-default-features 2>&1 | tail -3
cargo tree -p rum-stdlib --no-default-features -e normal | grep -E 'sha2|sha-1|sha1|md-5|crc32fast' || echo NO-DIGEST-CRATES
cargo test -p rum-stdlib --no-default-features 2>&1 | grep -E '^test result' | sort | uniq -c
# encodings still work without the feature (build a tiny check or a --no-default-features unit test):
#   ToHexString/FromBase64/ByteArrayQ must be present with --no-default-features
git status --short
```

The key gate: `NO-DIGEST-CRATES` prints (the four crates are absent from
the no-default-features dependency graph), the crate builds and its tests
pass under `--no-default-features` (digest tests cfg'd out), and the full
`cargo test --workspace` (default) stays green with the digests present.

## Scope

- MAY modify: `crates/rum-stdlib/Cargo.toml` (features + optional deps),
  `crates/rum-stdlib/src/**` (cfg gating of the digest builtins/tests,
  module/catalog split, install_core), the
  `catalog_attributes_match_kernel_vocabulary` test in
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/README.md` (one line).
- MUST NOT touch: the digest ALGORITHMS' behavior, the encoding builtins,
  the kernel vocabulary contents, other crates, root `Cargo.toml` (the
  workspace dep entries stay; only the rum-stdlib `[dependencies]` lines
  gain `optional = true`).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Default-feature behavior byte-identical: `cargo test --workspace` totals
  unchanged except any test you intentionally cfg-gate (report those).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. The feature mechanism as landed + the vocabulary-test resolution.
2. What changed — per file.
3. Verification — all commands verbatim (incl. NO-DIGEST-CRATES and the
   no-default-features test run).
4. Stop-loss / open questions (empty if none).
