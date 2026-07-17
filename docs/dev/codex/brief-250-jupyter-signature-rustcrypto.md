# Brief 250 — rum-jupyter signature.rs onto RustCrypto (kill the hand-rolled sha256)

## Goal

`crates/rum-jupyter/src/signature.rs` hand-rolls the entire wire-signing
stack — an HMAC envelope, `sha256_many`, and a full `sha256_compress`
round function (~165 lines). Replace it with the workspace RustCrypto
path: `hmac::Hmac<sha2::Sha256>`. The public seam (`hmac_sha256_hex(key,
parts: [&str; 4]) -> String`, lowercase-hex output) and its observable
behavior stay byte-identical — the existing known-test-vector test is the
equivalence pin and MUST NOT change. Both dependencies are already in the
workspace and pre-fetched (commit 070a23d); you need no network.

## Context pointers

- Read `AGENTS.md` first.
- `crates/rum-jupyter/src/signature.rs` — the whole file is the target:
  `hmac_sha256_hex` (:19) is the only `pub(crate)` seam; everything below
  it (`sha256_bytes`, `sha256_many`, `sha256_compress`, `nibble_to_hex`)
  is the hand-rolled machinery to delete. The test module at the bottom
  (`hmac_sha256_matches_known_test_vector`, ~:156) is the frozen pin.
- Its caller(s): grep `hmac_sha256_hex` in `crates/rum-jupyter/` — the
  call sites must not change.
- Policy: hashing/crypto primitives are never hand-rolled in this
  repository (AGENTS.md dependency policy, security/format clause); this
  brief closes the one surviving exception (found by brief-163's sweep,
  out of its scope then).

## Scope

- Files you MAY modify: `crates/rum-jupyter/src/signature.rs` ONLY.
- MUST NOT touch: anything else — `Cargo.toml`/`Cargo.lock` (the deps are
  already declared), all other crates and files, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your own
  report, `.git`.
- The existing test functions must remain VERBATIM (same names, same
  vectors, same assertions) — they are the proof that the port is
  behavior-identical. You may ADD test cases (e.g. a >64-byte key, an
  empty part) but not modify existing ones.

## Design (pinned)

1. `hmac_sha256_hex` keeps its exact signature and semantics: HMAC-SHA256
   over the four parts fed in order (the current code feeds them as
   sequential message chunks — replicate exactly: `Mac::update` per part,
   or one concatenation; they are equivalent for HMAC), key handling per
   RFC 2104 comes free from the `hmac` crate (including >block-size key
   hashing — the current manual `key_block` logic disappears), lowercase
   hex encoding of the 32-byte tag.
2. Hex encoding: keep it self-implemented (a small format loop or the
   existing `nibble_to_hex` helper) — do NOT add a hex dependency.
3. Everything hand-rolled about SHA-256 itself is DELETED: `sha256_bytes`,
   `sha256_many`, `sha256_compress`, the constants tables. Net result
   should be a drastically smaller file.
4. Imports: `hmac::{Hmac, Mac}` + `sha2::Sha256`, nothing more.

## Deliverable

- The rewritten `signature.rs` (RustCrypto core, frozen public seam,
  frozen tests, net deletion of the hand-rolled machinery).
- Green `cargo fmt --all --check` and `cargo test --workspace`.
- A grep proof that no hand-rolled compress remains:
  `rg -n 'sha256_compress|0x428a2f98' crates/` → empty.

## rum / Rust pitfalls (read twice)

1. The pinned test vector is the contract: if it fails, YOUR port is
   wrong (HMAC-SHA256 is HMAC-SHA256) — fix the port, never the test.
2. `hmac`/`sha2` are declared with `default-features = false` — if a
   trait/type you expect is missing, check which feature carries it and
   STOP if enabling one would touch Cargo.toml (out of scope; report).
3. No network: everything is pre-fetched; if cargo attempts a download,
   something is off-plan — STOP and report.
4. This crate has a native ZeroMQ test flake history (brief-165/242): if
   a jupyter transport test fails once, rerun it in isolation and report
   both outcomes honestly (green-on-rerun = record, not hide).
5. No timing gate (wire signing is not the eval hot path).

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace
rg -n 'sha256_compress|0x428a2f98' crates/
```

Paste totals and the empty grep verbatim.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you
  must.
- Touch NOTHING outside this repo, no network. Only Scope edits, `cargo`,
  and `python3`.
- Testing Policy hard line (`AGENTS.md`). No new `unsafe`. Minimal diff.
- Paste real verbatim verification output.

## Report format (your final message)

1. What changed — line ranges, deleted machinery inventory, one-line why.
2. Verification — verbatim fmt/test totals + the empty grep.
3. Confirmation that the pinned test functions are byte-unchanged (show
   `git diff` scoped to the test module = empty).
4. Stop-loss / open questions (including any ZeroMQ flake sighting).
