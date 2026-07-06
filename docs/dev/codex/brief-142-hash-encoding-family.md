# Brief 142 — Hash/encoding family (digests via RustCrypto; hex/base64 self-implemented)

Implementation brief. The byte toolbox: digests, hex/base64 codecs, and the
missing `ByteArrayQ`. Dependencies are ALREADY added and cached (commit
"brief-142 prep"): `sha2`, `sha1`, `md-5`, `crc32fast` — do NOT touch any
`Cargo.toml`/`Cargo.lock`; everything builds offline.

## Boundary (write it into the guide section)

Hashing yes, encryption never: digests and encodings are pure deterministic
byte functions and belong in the stdlib; encryption, signing, and key
handling never enter the kernel (a future downstream capability crate if
ever). `Md5`/`Sha1` carry reference limits notes: interop/checksum use only,
not collision-resistant.

## Builtins (new stdlib family; all `Pure`)

Input convention for the digest/encoding functions: a String (hashed as its
UTF-8 bytes) or a `ByteArray`. Errors follow family style (message + inert
or `$Failed` — match how existing string builtins reject wrong arg types;
state your choice).

- `Sha256[x]`, `Sha512[x]`, `Sha1[x]`, `Md5[x]` → lowercase hex String.
  Second form `[x, "Bytes"]` → `ByteArray`. (RustCrypto `digest` API.)
- `Crc32[x]` → unsigned integer (0..2^32-1), via `crc32fast`.
- `ToHexString[x]` → lowercase hex of the bytes; `FromHexString[str]` →
  `ByteArray` (case-insensitive input; odd length / non-hex → fail-closed
  message + `$Failed`).
- `ToBase64[x]` → standard RFC 4648 base64 with padding; `FromBase64[str]`
  → `ByteArray` (reject invalid alphabet/padding — fail-closed). Self-
  implemented (~40 lines each direction, safe Rust, no dependency).
- `ByteArrayQ[x]` → True/False.
- `ByteArrayToString[ba]` → String if the bytes are valid UTF-8, else
  fail-closed message + `$Failed` (partner of the existing
  `StringToByteArray`).

## Scope

- MAY create: `crates/rum-stdlib/src/bytes.rs` (+ submodules; or a
  `digest.rs`/`encoding.rs` split — your call, follow the family-layout
  norms), `crates/rum-stdlib/src/catalog/bytes.rs`, tests
  `crates/rum-stdlib/src/tests/bytes.rs`.
- MAY modify: `crates/rum-stdlib/src/catalog/mod.rs` + install path,
  `stdlib_attributes.rs` vocabulary (brief-136 lesson), docgen if needed,
  `docs/reference/` via generator, `docs/guide/structured-data.md` (a
  "Bytes, Digests, And Encodings" section with real transcripts — it is
  the natural companion of the codec chapter) and the guide README link
  line if needed.
- MUST NOT touch: `Cargo.toml`/`Cargo.lock` (deps are pre-added),
  evaluator/dispatch, kernel crates beyond the vocabulary registration.

## Tests

- Known-answer vectors (from the algorithms' specs) for each digest:
  empty input, "abc", a longer multi-block input — both String and
  ByteArray forms, hex and `"Bytes"` outputs consistent.
- `Crc32` known vectors (e.g. "123456789" → 3421780262).
- Hex/base64 round-trips incl. empty, all-byte-values (0..=255), unicode
  strings through UTF-8; fail-closed negatives (odd hex, bad base64
  padding/alphabet).
- `ByteArrayToString` UTF-8 validity both ways; `ByteArrayQ` boundaries.
- One end-to-end: `Sha256[ToJson[<|"a" -> 1|>]]` pinned literal (ties the
  codec and digest families together).
- Vocabulary/catalog registrations + reference drift.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
cargo run -q -p rum-cli -- --eval 'Sha256["abc"]'
cargo run -q -p rum-cli -- --eval '{Md5[""], Sha1["abc"], Crc32["123456789"]}'
cargo run -q -p rum-cli -- --eval 'FromBase64[ToBase64["héllo, 世界"]] // ByteArrayToString'
cargo run -q -p rum-cli -- --eval '{ToHexString[StringToByteArray["ab"]], FromHexString["6162"] // ByteArrayToString}'
cargo run -q -p rum-cli -- --eval 'FromHexString["6"]'        # fail-closed
cargo run -q -p rum-cli -- --eval 'Sha256[FromHexString["00ff"], "Bytes"]'
git status --short
```

(`Sha256["abc"]` must equal the standard
`ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network —
  everything needed is in the cargo cache.
- Digests ONLY through the pre-added crates — never hand-rolled; hex/base64
  ONLY hand-rolled (no new deps).
- No new `unsafe` in rum code. If dispatch/eval code needs touching beyond
  registrations, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file; the family layout chosen.
2. Verification — all commands verbatim (known-answer vectors included).
3. Any argument-convention decisions beyond this brief.
4. Stop-loss / open questions (empty if none).
