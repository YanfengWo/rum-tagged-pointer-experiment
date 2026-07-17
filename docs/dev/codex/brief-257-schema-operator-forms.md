# Brief 257 — ConformSchema/SchemaErrors operator forms via special-dispatch capture (wl15/wfr batch 2b-II)

> **REVISION 1 (2026-07-18).** The first run stop-lossed on an
> over-strong probe expectation written by the director — NOT on a
> mechanism defect. Director-verified truth on HEAD: the DIRECT form
> today also evaluates ordinary subexpressions inside the schema
> during validation (`SchemaValidator::match_leaf` → `MatchQ` pattern
> resolution): `c = 0; ConformSchema[<|"a" -> 1|>, (c = c + 1;
> <|"a" -> _|>)]` yields `$Failed` with `c = 1`. HoldRest prevents
> ARGUMENT-time evaluation; validation-time resolution is a shared,
> pre-existing property of both calling forms. The first run's
> candidate already achieved exactly the right thing: capture
> evaluates nothing, and application output was byte-identical to the
> direct call (same `$Failed`, same messages, same final `c`). The
> timing probe and its deliverable bullet below are corrected to pin
> THAT parity contract; everything else is unchanged. Re-implement
> (the previous work was reverted under stop-loss).

`ConformSchema[schema][value]` and `SchemaErrors[schema][value]` work,
with the schema NEVER evaluated in either calling form (user ruling B2
in `docs/dev/wl15-wfr-hold-design.md` §6: special-dispatch capture,
with a stop-loss back to DEFERRED). Direct calls
`ConformSchema[value, schema(, opts…)]` stay byte-identical — same
results, same messages, same `Attributes` introspection
(`{HoldRest, Protected}`). The unary capture `ConformSchema[schema]`
becomes a quiet inert callable (today it emits a `schema` message).

## Context pointers (all sites director-verified 2026-07-17)

- Read `AGENTS.md` and `docs/dev/wl15-wfr-hold-design.md` §4 B2 first.
- `crates/rum-stdlib/src/catalog/schema.rs` — both entries are
  `BuiltinSpec::ordinary_with_attributes(…, HoldRest+Protected, …)`.
  Switch to `BuiltinSpec::special_with_hold(name, summary,
  HoldAttribute::HoldRest, handler).with_messages(…).protected()` —
  special dispatch, attributes still declare `{HoldRest, Protected}`
  so introspection is unchanged (kernel precedent: catalog/rewrite.rs
  uses `BuiltinSpec::special` for the ReplaceAll family; the stdlib
  seam exists).
- `crates/rum-stdlib/src/schema.rs:49-94` — `eval_schema_errors` /
  `eval_conform_schema` (same `BuiltinFn` shape serves special forms;
  they will now receive RAW args). Handler changes:
  1. arity 1 → return the inert call quietly (the capture); use the
     `head` parameter (currently `_head`) to rebuild.
  2. arity ≥2 → evaluate `args[0]` explicitly (the value position —
     today generic preparation evaluates it; positions 2+ are already
     raw under HoldRest, so `parse_schema_call` sees exactly what it
     sees today). Follow the existing special-form handler style for
     in-handler evaluation (the Rule family evaluates subterms the
     same way); propagate `EvalError` as usual.
- `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs` —
  add a small schema-capture wrapper family beside the mutation family
  (brief-256's shape): recognize unary `ConformSchema[s]` /
  `SchemaErrors[s]` as callable wrappers; application to exactly one
  argument rebuilds `Head[value, s]` with `s` cloned RAW; wrong
  application arity via the argx helper. Declare the wrapper `argx`
  message on both catalog entries (they currently declare only
  `schema`; the push_message debug assertion is the arbiter — mirror
  brief-256's `ARGX_OPERATOR` usage in catalog/associations.rs).
- Tests: `crates/rum-stdlib/src/tests/schema.rs` (existing suite must
  pass UNCHANGED — it is the byte-identity evidence; if any test
  fails, that is the stop-loss signal, not a test to edit) + a new
  test file or module for the operator forms.
- `docs/reference/builtins.md` — regenerate ONLY if a summary/message
  declaration changes (`cargo run -p rum-stdlib --bin
  generate-builtin-reference`).

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/catalog/schema.rs`
  - `crates/rum-stdlib/src/schema.rs`
  - `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  - a NEW stdlib test file for the operator forms + its `mod`
    registration (`crates/rum-stdlib/src/tests.rs` or the tests dir
    mod file — mechanical one-liner)
  - `docs/reference/builtins.md` (regeneration output only)
- MUST NOT touch: `crates/rum-stdlib/src/tests/schema.rs` (byte-
  identity evidence), brief-255/256 planner+wrapper code beyond adding
  the new family, everything else.
- No unconditional default-path cost; the new recognizer runs only on
  compound-head dispatch.

## Design decisions (ruled — implement, do not re-litigate)

1. Mechanism: special dispatch + raw-schema capture (option 1). Do
   NOT use `SubValuesHoldAll` here — the applied value MUST evaluate.
2. The capture is quiet inert; captures applied to ≠1 args get the
   wrapper argx message.
3. Known, accepted semantic footnote: special forms skip the
   UpValues/Listable stages. No existing schema test exercises
   upvalues on these heads (the suite passing unchanged is the
   evidence); note the footnote in your report.
4. Double evaluation of the applied value (once during compound-head
   preparation, once by the handler) is idempotent under the
   fixed-point discipline; pin with the capture-timing probe below —
   side effects in the VALUE fire exactly once per application.
5. **Stop-loss (ruled)**: if any existing schema test changes
   behavior, or the handler rework grows beyond value-evaluation +
   the unary branch, STOP, leave the tree clean of half-work, and
   report — the fallback is DEFERRED, decided by the director.

## Deliverable

- The dispatch switch + handler rework + wrapper family + tests
  (measured expectations — probe the release CLI first, then pin):
  operator form for both heads (conform success, conform failure →
  `$Failed` + `schema` message parity, SchemaErrors error list),
  schema-side-effect timing per the REVISED contract — the CAPTURE
  evaluates nothing (`w = ConformSchema[(c = c + 1; <|…|>)]` leaves
  `c` at 0), and APPLICATION behavior is byte-identical to the direct
  call (same result, same messages, same final `c` — validation-time
  evaluation included; the direct-form baseline for the same schema
  expression is the comparison, pasted side by side),
  value-side-effect single-fire, quiet capture, wrapper argx,
  direct-call byte-identity probes, options form (`"AllowExtraKeys"`)
  through both calling shapes.
- Full existing schema suite green UNCHANGED.
- Microbench A/B (best-of-2 per side, interleaved runs B C B C):
  downvalue tier + neighbors; the panel has no compound-head label —
  state it. FAIL criterion: a label's Δ outside the unaffected-controls
  |Δ| distribution AND same direction across independent runs.

## rum / Rust pitfalls (read twice)

1. Expected values MEASURED, not predicted; probes feed stdin
   (`printf '' | … --eval '…'`).
2. In-handler evaluation must stay suspension-safe: propagate
   `EvalError` (including `InputRequired`) without collapsing frames.
3. The raw schema is `Arc`-cheap to clone; no deep clones.
4. Do not "improve" schema validation logic while there — minimal diff.
5. Stop-loss is real (decision 5): twice-failed same step → STOP.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace --exclude rum-jupyter   # sandbox loopback exclusion; note it
```

Then the A/B, and paste VERBATIM (all via `printf '' | cargo run -q
-p rum-cli -- --eval …`):

```
ConformSchema[<|"a" -> 1|>, <|"a" -> _Integer|>]
ConformSchema[<|"a" -> _Integer|>][<|"a" -> 1|>]
ConformSchema[<|"a" -> _Integer|>][<|"a" -> "x"|>]
SchemaErrors[<|"a" -> _Integer|>][<|"a" -> "x"|>]
c = 0; w = ConformSchema[(c = c + 1; <|"a" -> _|>)]; {c, w[<|"a" -> 1|>], c}
c = 0; {ConformSchema[<|"a" -> 1|>, (c = c + 1; <|"a" -> _|>)], c}
d = 0; ConformSchema[<|"a" -> _Integer|>][(d = d + 1; <|"a" -> 1|>)]; d
ConformSchema[<|"a" -> _Integer|>]
ConformSchema[<|"a" -> _Integer|>][1, 2]
{Attributes[ConformSchema], Attributes[SchemaErrors]}
```

## Hard rules

- No `git commit`/branch/push; leave changes in the tree.
- Nothing outside Scope; must-touch conflicts → STOP and report.
- Nothing outside this repo, no network; commands: Scope edits,
  `cargo`, release `rum-cli`, microbench scripts, `python3`.
- Testing Policy hard line; no new `unsafe`; minimal diff; probe
  budget best-of-2 per A/B side; verbatim output only, never faked.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — commands + verbatim output.
3. Verdict — behavior table + A/B reading + the upvalue footnote.
4. Stop-loss / open questions (empty if none).
