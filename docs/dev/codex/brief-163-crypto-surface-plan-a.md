# Brief 163 — Crypto surface plan A: remove toy Encrypt/Decrypt; Hash onto RustCrypto

Implementation brief. Executes the user's 2026-07-07 decision (tracker
`docs/dev/usability-pass.md`, unit-1 triage item 6, plan A). Two policy
lines are being enforced: encryption/signing never enter the kernel
(`docs/guide/structured-data.md` already says so — the toy `Encrypt`
contradicts the guide today), and digests come from the RustCrypto
crates, never hand-rolled.

## Changes

1. **Remove `Encrypt` and `Decrypt`**: their stdlib catalog entries
   (`crates/rum-stdlib/src/catalog/strings.rs`), implementations
   (`crates/rum-stdlib/src/strings/encoding.rs`), and tests. Remove the
   `EncryptedObject` inert head from the kernel registration
   (`crates/rum-runtime/src/runtime/builtin_catalog/strings.rs` — the
   `BuiltinSpec::new` entry). No deprecation shim: the three names
   become ordinary undefined symbols. Git history is the archive.
2. **Delete the hand-rolled SHA-256** in `strings/encoding.rs`
   (`sha256_compress`, `sha256_many`, `sha256_bytes`,
   `xor_with_sha256_keystream`, and their helpers). After #1 the only
   remaining caller is `Hash`; if you find any OTHER caller, STOP and
   report. `Hash` switches to the `sha2` crate (as
   `crates/rum-stdlib/src/bytes/digest.rs` does).
   **Byte-identical contract**: BEFORE swapping the engine, pin the
   current `Hash` outputs with tests (the existing known-answer vectors
   plus every supported output mode — hex default, `"DecimalString"`,
   and whatever else the implementation accepts today); after the swap,
   the same tests pass unchanged.
3. **Extend `Hash` method names** (folk WL parity): `"SHA1"`,
   `"SHA512"`, `"MD5"`, `"CRC32"`, routed to the same crates the digest
   builtins use (`sha1`, `sha2`, `md-5`, `crc32fast`). Cross-check the
   outputs against the known-answer vectors already in
   `crates/rum-stdlib/src/tests/bytes.rs` (rum-internal consistency —
   no external engines). Unknown method names keep today's inert
   behavior; the default method is unchanged.
4. **Feature-gate `Hash` behind `hash`** (brief-160 mechanism): its
   catalog entry, implementation, and tests go under
   `#[cfg(feature = "hash")]`; under `--no-default-features` the
   builtin is absent, exactly like `Sha256`. Extend the existing
   `no_default_features_drop_digest_builtins_but_keep_codecs` test to
   include `Hash` in the dropped set. (You may either cfg-gate in place
   under `strings/` or move the `Hash` implementation next to
   `bytes/digest.rs` — state which you chose and why; smaller diff
   wins.) The kernel vocabulary stays feature-unaware (catalog ⊆
   vocabulary).
5. **Regenerate the builtin reference**
   (`cargo run -p rum-stdlib --bin generate-builtin-reference`): three
   entries disappear; `Hash`'s description updates to name the method
   set. Touch `docs/guide/structured-data.md` ONLY if it references
   `Encrypt`/`Decrypt` by name (it appears not to — the policy sentence
   already matches the new reality).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# byte-identity + new methods (SHA1/MD5 of "abc" must match the KAT
# vectors used by the digest builtins' tests):
printf '' | cargo run -q -p rum-cli -- --eval '{Hash["abc", "SHA256"], Hash["abc", "SHA256", "DecimalString"], Hash["abc", "SHA1"], Hash["abc", "SHA512"], Hash["abc", "MD5"], Hash["abc", "CRC32"], Hash["abc", "NoSuchMethod"]}'
# removal: plain undefined symbols now
printf '' | cargo run -q -p rum-cli -- --eval '{Encrypt["k", "v"], Decrypt["k", "x"], EncryptedObject[1]}'
# hand-rolled digest gone
grep -rn 'sha256_compress\|xor_with_sha256_keystream' crates/ || echo HAND-ROLLED-GONE
# no-default-features: builds, tests green, digest crates absent
cargo build -p rum-stdlib --no-default-features 2>&1 | tail -3
cargo tree -p rum-stdlib --no-default-features -e normal | grep -E 'sha2|sha1|md-5|crc32fast' || echo NO-DIGEST-CRATES
cargo test -p rum-stdlib --no-default-features 2>&1 | grep -E '^test result' | sort | uniq -c
git status --short
```

Expected: `Hash["abc","SHA256"]` output identical to before the change
(paste the pre-change value alongside); the SHA1/MD5 values equal the
KAT vectors; unknown method inert; the three removed names evaluate to
inert undefined-symbol calls with no messages; `HAND-ROLLED-GONE` and
`NO-DIGEST-CRATES` print; both default and NDF test suites green.

## Scope

- MAY modify: `crates/rum-stdlib/src/strings/encoding.rs` (or a moved
  successor under `crates/rum-stdlib/src/bytes/`),
  `crates/rum-stdlib/src/catalog/strings.rs`,
  `crates/rum-stdlib/src/catalog/bytes.rs` (only if Hash moves there),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  `crates/rum-stdlib/src/lib.rs` (registration only),
  `crates/rum-runtime/src/runtime/builtin_catalog/strings.rs`
  (EncryptedObject removal + Hash vocabulary line if present),
  `docs/reference/builtins.md` (regenerated only),
  `docs/guide/structured-data.md` (only per Changes #5).
- MUST NOT touch: the digest builtins' behavior (`Sha256` etc.), other
  catalog families, `Cargo.toml` files, `TODO.md`, the tracker, the
  findings file.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior changes are EXACTLY: (a) `Encrypt`/`Decrypt`/
  `EncryptedObject` become undefined symbols, (b) `Hash` gains the four
  method names, (c) `Hash` disappears under `--no-default-features`.
  Every other result — including every existing `Hash` call — is
  byte-identical (pin before, verify after).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; where `Hash` ended up living and why.
2. The pre-change vs post-change `Hash` pin outputs.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
