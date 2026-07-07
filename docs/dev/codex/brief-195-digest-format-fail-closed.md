# Brief 195 — Digest fail-closed on unknown output format + Crc32 non-security note (unit-6 fix 1)

Implementation brief. Executes the unit-6 triage batch 1 (tracker
`docs/dev/usability-pass.md`, "Unit 6 (codecs+bytes+datetime) triage";
evidence E7/E13 in
`docs/dev/usability-pass/codecs-bytes-datetime-findings.md`). Read the
tracker's unit-6 triage first. This is the ONLY behavior-changing brief
in the unit; it is a fail-closed safety fix plus one reference note.

## The bug (safety — reproduced)

`eval_digest` (`crates/rum-stdlib/src/bytes/digest.rs`, the shared
function behind `Sha256`/`Sha512`/`Sha1`/`Md5`) matches its args like:

```rust
let (arg, bytes_output) = match args {
    [arg] => (arg, false),
    [arg, format] if expr_as_string(format).is_some_and(|f| f == "Bytes") => (arg, true),
    [_arg, _format] => return Ok(Expr::call(head.clone(), args.to_vec())), // <-- BUG
    _ => return Ok(session.invalid_call_with_arg_range(symbol, args, 1, 2)),
};
```

The third arm returns an INERT unevaluated call for an UNKNOWN output
format: `Sha256["abc", "Base64"] -> Sha256["abc", "Base64"]` with no
message (silent). A valid first argument with a garbage format name
should fail CLOSED, not silently do nothing.

## The fix — `crates/rum-stdlib/src/bytes/digest.rs`

Replace the inert unknown-format arm with a fail-closed message, and
accept `"Hex"` as an explicit synonym of the default hex output (so the
supported output-format set is the coherent pair `"Hex"` (default) /
`"Bytes"`):

- `[arg]` → default hex (unchanged).
- `[arg, format]` where the format string is `"Bytes"` → ByteArray output
  (unchanged).
- `[arg, format]` where the format string is `"Hex"` → default hex output
  (NEW explicit alias; behaves exactly like the no-format form).
- `[arg, format]` where `format` IS a string but is neither `"Hex"` nor
  `"Bytes"` → **fail closed**: `fail_bytes(session, symbol, "format",
  <message>)` (the helper is already imported;
  `fail_bytes(session, symbol, tag, msg) -> $Failed` pushes a
  `symbol::tag` message — see `crates/rum-stdlib/src/bytes/codec.rs:55`).
  Phrase the message in the house style of the other bytes messages
  (e.g. `unknown digest output format; use "Hex" (default) or "Bytes"`).
- Arity/other arms (`_ =>`) → unchanged (`invalid_call_with_arg_range`).

**CRITICAL — do NOT touch the WRONG-TYPE inert path.** A non-string /
non-ByteArray FIRST argument (`ByteInput::WrongType`, e.g. `Sha256[123]`,
`Crc32[123]`) must STAY inert — that is the folk-consistent behavior
shared with `ToHexString[123]`/`FromHexString[123]`/… (same test). Only
the second-argument UNKNOWN-FORMAT case (valid first arg present) changes
from inert to fail-closed. If a `format` argument is present but is not a
string at all, treat it as an unknown format and fail closed the same way
(it can never be `"Hex"`/`"Bytes"`).

Apply the SAME change once in the shared `eval_digest` so all four SHA/MD5
heads inherit it. `Crc32` (a separate `eval_crc32`) has no format
argument and is NOT part of this change.

## The Crc32 reference note — `crates/rum-stdlib/src/docgen.rs`

`Md5` and `Sha1` carry a LIMITS note (docgen table, ~lines 50-53 /
127-130): *"intended only for interoperability and checksums; it is not
collision-resistant."* `Crc32` has none. Add a `Crc32` entry to the same
`(name, note)` table, in table order, e.g.:

```
("Crc32", "computes the IEEE CRC-32 checksum for integrity and interoperability, not security; it is not collision-resistant."),
```

Then regenerate the committed reference (drift test
`crates/rum-stdlib/src/tests/builtin_reference_docgen.rs`
`builtin_reference_is_in_sync` enforces it):

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

The only `builtins.md` diff should be the new `Crc32` Limits line. Do NOT
hand-edit `builtins.md`.

## Tests — `crates/rum-stdlib/src/tests/bytes.rs`

The change breaks `unsupported_argument_types_remain_inert`
(`tests/bytes.rs:256`), which asserts `Sha256["abc", "HexString"]` stays
inert. That assertion now encodes the OLD (buggy) behavior. Move it:

- REMOVE `"Sha256[\"abc\", \"HexString\"]"` from the
  `unsupported_argument_types_remain_inert` list (KEEP `Sha256[123]` and
  `Crc32[123]` there — those are the wrong-type inert path that is
  unchanged).
- ADD a fail-closed assertion (co-locate with the other bytes
  fail-closed tests in this file; mirror their `push_message`/`$Failed`
  assertion shape, `#[cfg(feature = "hash")]`): `Sha256["abc",
  "HexString"]` → `$Failed` with a `Sha256::format` message. Also pin the
  new `"Hex"` alias equals the default: `Sha256["abc", "Hex"] ==
  Sha256["abc"]` (both the hex string).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Sha256["abc"], Sha256["abc", "Hex"], Sha256["abc", "Bytes"], Sha256["abc", "Base64"], Sha256[123], Crc32[123], Sha512["abc", "Zzz"]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — reference
regenerated, drift + bytes tests pass). Probe: `Sha256["abc"]` and
`Sha256["abc", "Hex"]` are the SAME hex string
(`ba7816bf...20015ad`); `Sha256["abc", "Bytes"]` a ByteArray;
`Sha256["abc", "Base64"]` and `Sha512["abc", "Zzz"]` now `$Failed` (with a
`::format` message); `Sha256[123]` and `Crc32[123]` STILL inert
(`Sha256[123]` / `Crc32[123]`, no message).

## Scope

- MAY modify: `crates/rum-stdlib/src/bytes/digest.rs` (the fail-closed
  fix + `"Hex"` alias), `crates/rum-stdlib/src/docgen.rs` (the `Crc32`
  Limits note), `docs/reference/builtins.md` (regenerated — the one
  `Crc32` line only), `crates/rum-stdlib/src/tests/bytes.rs` (move the
  one assertion + add the alias pin).
- MUST NOT touch: the wrong-type inert path (`ByteInput::WrongType`);
  `eval_crc32`'s behavior; any codec/date builtin; `Protected` /
  attributes (that is brief-196); option registration (brief-197); any
  other file, `TODO.md`, the tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- ONE behavior change only: unknown digest output format → fail closed;
  `"Hex"` accepted as the explicit default alias. Everything else
  byte-identical. Wrong-type first arg STAYS inert.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- The MAY list spans the test file on purpose (the cross-crate MAY-list
  lesson): a behavior change's tests must move with it. Update ONLY the
  one moved assertion + the alias pin.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the digest match arm, the `Crc32` docgen note + regen,
   the moved test assertion; line ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
