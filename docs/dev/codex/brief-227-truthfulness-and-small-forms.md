# Brief 227 — Truthfulness + small forms: dead options, DumpSave result, ByteCount doc, EchoFunction, Contexts filter, TestCreate discovery

Fix brief — unit-10 batch 4, the CLOSING mechanical batch (safety/
convenience, S/M). Source: `docs/dev/usability-pass/kernel-c-findings.md`
(top-gaps #6/7/8/9/10, Evidence E4/E6/E8). Six independent builtin-local
sub-fixes; NONE touches call planning (no A/B) and NONE changes the
DumpSave FILE format (no mma-hol gate). All are truthfulness or
WL-parity moves; valid-input behavior is byte-identical except where a
current output is a documented lie.

## (a) Dead-option guards (the 217 pattern)

Two registered options are accepted but never read:

- `Options[Names]` advertises `ResolveContextAliases -> True`; `Names`
  never consults it.
- `Options[DumpSave]` advertises `CompressionLevel -> 1`; `DumpSave`
  never consults it (gate-verified: `CompressionLevel -> 0` and `-> 9`
  produce byte-identical 911-byte files).

Required: follow EXACTLY what brief-217 did for `Names`
`SpellingCorrection` — a NON-DEFAULT value emits a message and the call
proceeds with the default / fails closed in the same shape 217 used
(PROBE 217's `Names::optval` behavior and mirror it precisely; use
`DumpSave::optval` for the DumpSave option). The DEFAULT value stays
byte-identical and silent, and the option STAYS registered (discovery is
fine — the lie was the silent acceptance of a non-default). State in the
report which message shape you mirrored.

## (b) DumpSave result list — return symbol identities, not values

Today (E4): `aVal = 5; DumpSave["/tmp/x.mx", aVal]` → `{5}`. `DumpSave`
is `{HoldRest, Protected}`, so it RECEIVES the symbol `aVal`
unevaluated — but its result-list construction re-evaluates it to `5`.
WL's `DumpSave` returns the list of SAVED SYMBOLS, not their values.

Required: the result list carries the symbol IDENTITY (the held symbol,
or its context-qualified name string — pick whichever rum's
`Save`/`Definition`/value-table family already uses for "a symbol
reference" and PIN it; state which and why), NOT the re-evaluated value.
No attribute change is needed (`HoldRest` is already present); the fix is
in how the result is built. The FILE written is byte-identical — this
changes only the returned value. Baseline-verify the file is unchanged.

## (c) ByteCount doc aligns to the (WL-correct) structural behavior

Today (E8): `ByteCount[1]` = `ByteCount[123456789]` = 32,
`ByteCount["abc"]` = 35 — a structural memory proxy, which is what WL's
`ByteCount` returns (expression memory size, not canonical-text length).
The BEHAVIOR is right; the reference DOC claiming "canonical-text bytes"
is the lie.

Required: fix the `ByteCount` reference/docgen summary to describe the
structural memory-size behavior honestly (folk proxy, WL-sense). NO
behavior change. docgen + regenerate reference; MERGE if a Limits note
exists.

## (d) EchoFunction[f] applies f (WL parity)

Today (E6): `EchoFunction[f][3]` → prints `f: 3`, returns 3 — it treats
`f` as a LABEL, duplicating `EchoLabel[f][3]` (`lbl: 3`). WL's
`EchoFunction[f][expr]` prints `f[expr]` (the function APPLIED to the
value) and returns `expr` unchanged.

Required: `EchoFunction[f][expr]` echoes `f[expr]` (applied, then
rendered) and returns `expr`. `EchoLabel` is untouched (it stays the
label form). Session-mediated output stays `Pure` (the Sow/Reap
doctrine — verify the echo goes through the session print channel, as
`Echo` already does). Pin `EchoFunction[f][x]` ≠ `EchoLabel[f][x]` in a
test.

## (e) Contexts[patt] filter form

Today (E8): `Contexts["System`*"]` → `Contexts::argx: expected exactly 0
argument(s)`. WL's `Contexts[]` returns all contexts; `Contexts["patt"]`
filters by a context name pattern.

Required: `Contexts[patt]` returns the subset of context names matching
`patt`, REUSING the existing wildcard/name matcher (the one `Names` uses
— cite it; do not hand-roll a second matcher). `Contexts[]` (0-arg)
byte-identical. A no-match pattern returns `{}` (not an error, not a
literal wildcard-named entry — the brief-213 name-pattern discipline).

## (f) TestCreate option discovery

Today (E8): `Options[TestCreate]` = `{}` empty, though `TestCreate`
accepts option-like rules. Register the option defaults for `Options[…]`
discovery — the brief-197/207 pattern (registration only; the
implementation already reads them directly, so behavior is
byte-identical). State which options you registered and confirm each is
actually consumed by the implementation (do NOT register a phantom option
— that would recreate the (a) lie).

## Method

1. Baseline-verify each item's current behavior AND the preservation pin
   before touching code (the standing lesson). List old→new per item.
2. Items are independent — implement each with its tests green before the
   next.
3. Tests: per item, the fix + the byte-identical default/preservation
   pin. For (b), assert the written FILE is unchanged and only the return
   value differs.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes per sub-fix, before/after, batched where safe (use /tmp for
#   DumpSave); the 217-mirror check for (a); the file-unchanged check
#   for (b)
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- (a) `crates/rum-runtime/src/runtime/options/defaults.rs` +
  `crates/rum-runtime/src/runtime/namespace_tools/names.rs` +
  the DumpSave option-reading site (`package_tools.rs` or wherever
  DumpSave parses options — find and name it)
- (b) the DumpSave result-building site (name it)
- (c) `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
- (d) `crates/rum-runtime/src/runtime/debug_tools/` (the Echo family) or
  wherever `EchoFunction` is implemented (find and name it)
- (e) the `Contexts` implementation site (find and name it) — reuse the
  Names matcher
- (f) `crates/rum-runtime/src/runtime/options/defaults.rs` (TestCreate
  registration)
- Tests: the namespace / package / debug-tools / options test families
  across crates (list each file in the report); one new test file allowed
- docgen + reference regenerated for any contract-note change (MERGE)

MUST NOT touch: the loader seam (225/226 settled), the evaluator core,
the matcher, capability labels/declarations, the DumpSave FILE format,
`Input`/`Dialog` (separate decision), `EchoLabel` (the label form stays),
attribute sets (no hold changes — DumpSave already has HoldRest),
anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`, no attribute changes, no capability changes.
- No A/B (no call-planning touch) and no mma-hol gate (no DumpSave file
  format change) — justify both in the report by showing the diff has no
  attribute/planner/loader/file-format file.
- Valid-input behavior byte-identical; baseline-verify every
  preservation pin, especially the DumpSave file (b) and the option
  defaults (a).
- WL comparison from documented knowledge only; do not register a
  phantom TestCreate option.
- Stop-loss: if any item turns out to need an attribute change (e.g. (d)
  or (b) cannot be done without a hold change), STOP that item and report
  it as needing its own A/B-gated brief; land the rest.

## Report format

1. Per-item summary: seam, mechanism, the 217-mirror for (a), the pinned
   symbol-reference shape for (b).
2. Probe panel before/after verbatim, per item; the DumpSave
   file-unchanged evidence for (b).
3. Test survey old→new.
4. Verification outputs verbatim; the no-A/B + no-mma-hol-gate
   justification (diff has no attribute/planner/loader/format file).
5. Deviations / open questions (empty if none).
