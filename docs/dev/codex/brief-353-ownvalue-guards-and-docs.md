# brief-353 — rhs `Condition` guards, step 6: `OwnValues`, and close the line

## Goal

Finish the rhs-guard repair. `OwnValues` is the last leaking lane, and this
brief also documents the whole feature and regenerates the reference.

Measured today (release CLI at `79afc6d8`):

```text
y := 6 /; True          ->  y  is  6 /; True      must be 6
z := 7 /; False         ->  z  is  7 /; False     must be z (unevaluated)
w := 8 /; someUndefined ->  w  is  8 /; someUndefined   must be w
ValueQ[z]               ->  True                   must be False once z stays z
```

Unguarded own-values are correct and must stay byte-identical.

## Director's rulings — implement exactly these

1. **No storage-representation change.** `SymbolDefinition` keeps its
   immediate/delayed own-value slots. Do **not** migrate symbols to an ordered
   candidate store; that was considered and rejected as far too much machinery
   for how rare a guarded own-value is.
2. **No cost on the own-value read path.** `symbol-ownvalue-read` is one of
   the hottest labels in the suite. Whatever distinguishes a guarded value
   must be effectively free when there is none — a discriminant already being
   matched, or a `bool` set at installation. If you cannot show it is free,
   STOP and report rather than shipping a hot-path check.
3. **A declining guard leaves the symbol unevaluated**, exactly as if it had
   no own-value at all. `z := 7 /; False` makes `z` evaluate to `z`.
4. **Multiple guarded own-values for one symbol are not supported.** The
   single slot means a later definition replaces an earlier one, which is
   today's behavior and stays. Document it as a known limitation; do not
   invent a candidate list.

## Context pointers

- The five previous steps: `5ab6afc5` (DownValues), `efd6dc80`
  (UpValues/SubValues/Defaults), `5418c060` (rewrite rules), `b94a5a4a`
  (`ReplacePart`), `79afc6d8` (string rules). The substrate is
  `definitions/rhs_guards.rs`; reuse it, do not fork it.
- `crates/rum-runtime/src/runtime/eval_core/symbol.rs:65-91` — the own-value
  read path, in evaluation order: `own_value`, `delayed_own_value`, then the
  dynamic frame.
- Installation goes through the same definition paths the earlier steps
  touched.
- Reading guard-effect counters: a `Rule`/`Set` evaluates its rhs at
  construction, so a guard written immediately has already run once. Use the
  delayed form to observe guard evaluation alone.

## Deliverable

### `OwnValues`

- Immediate and delayed own-values split their guard at installation.
- On read, a guarded own-value evaluates its guards left to right in the
  value's option scope, accepting only exact `True`; a decline leaves the
  symbol unevaluated and falls through to the dynamic frame exactly as an
  absent own-value does.
- `OwnValues[sym]`, `Definition[sym]`, and package serialization rebuild the
  `Condition` form.
- Suspension inside the guard resumes without replaying it.

### Documentation (this is the only brief in the line allowed to touch docs)

- `docs/guide/patterns-and-rules.md`: a section on rhs guards covering what
  `lhs := rhs /; test` means, that a decline is indistinguishable from the
  rule not matching, that guards run user code, and the per-lane ordering
  contracts the line established:
  - definition tables and rewrite-rule lists: candidates in their normal
    order, first acceptance wins;
  - `ReplacePart`: contested positions run **last rule to first**, and the
    winner's earlier rivals are not evaluated at all;
  - string rules: first-match-wins, rules tried in list order at each
    position;
  - a declining `Default` leaves the optional argument with no default;
  - the `OwnValues` limitation from ruling 4.
- Any catalog description that is now inaccurate.
- Regenerate the reference:
  `cargo run -p rum-stdlib --bin generate-builtin-reference`, and include the
  regenerated file in the diff.

### Tests

- Immediate and delayed own-values, accepting and declining guards.
- A declining own-value guard inside `Block`/`Module` scoping still falls
  through correctly.
- `ValueQ[z]` is `False` when `z`'s guard declines.
- Non-boolean guard results decline.
- A guard that suspends resumes correctly — assert a counter.
- Ruling 4 pinned: the second guarded own-value replaces the first.
- Unguarded own-values byte-identical (existing suites are the assertion).
- **A whole-line regression test**: one test that exercises a guarded rule in
  every lane — `DownValue`, `UpValue`, `SubValue`, `Default`, rewrite rule,
  `ReplacePart`, `StringReplace`, `OwnValue` — asserting exact values, so a
  future change cannot silently break one lane.

### A/B

`--scale 5`, same-binary noise floor **first**, then base versus candidate:

```
symbol-ownvalue-read      set-ownvalue        block-symbol-only-repeat
block-init-read-repeat    block-init-null-repeat
downvalue-exact-hit       matcher-single-tiny
```

`symbol-ownvalue-read` and `set-ownvalue` are the two that must not move; per
ruling 2 the expectation is that they cannot. Add a guarded own-value read
label. Name the untouched controls; never the suite median. If a required
label lands within a third of a percentage point of your measured floor,
re-measure the pair before concluding.

## Scope

- Files you MAY modify: `eval_core/symbol.rs`; the definition installation and
  catalog modules needed for own-values; `model/definitions.rs` if ruling 2
  needs a flag; `definitions/rhs_guards.rs` only if a genuine extension is
  needed; runtime and stdlib tests; `docs/guide/patterns-and-rules.md`;
  `docs/reference/builtins.md` (regenerated only); the catalog descriptions
  behind it; `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
  If a site needs a real decision rather than a mechanical fix, STOP and
  report.
- **The general accept/stop criterion**: *any behavior change that occurs only
  when a guarded definition exists is legitimate; any behavior change
  observable with no guarded definition present is a stop.*
- MUST NOT touch: `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `.git`, `Cargo.lock`,
  `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **Ruling 2 is the hard one.** The own-value read is on the hot path for
   every symbol lookup in the language. Show your reasoning for why the
   guarded check is free, and back it with the A/B.
2. **Do not build a candidate list** — ruling 1 and 4 exist to keep this
   step small.
3. **Guards run user code**; preserve suspension continuation shape.
4. **A decline is not an error** and emits no message.
5. **Reuse the substrate.**
6. **Regenerate, do not hand-edit, `docs/reference/builtins.md`.**
7. **Counters before timings.**
8. **Stop-loss is real.** If a step fails twice for the same reason, STOP and
   report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described. Confirm the regenerated reference is in the diff
and that the drift test covering it passes.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no third-party engine comparison anywhere, including in the
  new guide section.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file, mechanically-forced files listed separately.
2. **How ruling 2 was satisfied**: what distinguishes a guarded own-value, and
   why it costs nothing when there is none.
3. Verification — each command with verbatim output, including the reference
   regeneration and its drift test.
4. The exact-value test matrix, including the whole-line regression test.
5. The A/B — counters first, then rows, with the same-binary noise floor and
   the controls named.
6. Stop-loss / open questions.
