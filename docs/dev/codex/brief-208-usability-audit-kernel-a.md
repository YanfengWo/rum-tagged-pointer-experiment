# Brief 208 — Usability audit: kernel A (definitions / values / attributes / protection; 55 builtins)

Audit brief — sweep unit 8 of the Usability Pass, and the FIRST kernel unit
(units 1-7 were all stdlib catalog families). Read `docs/dev/usability-pass.md`
FIRST: the three axes, the compat rule, the coverage discipline, and the
closed-unit ledgers (units 1-7) — they show the judgment standard, the
corrections method, and the operator-form disambiguation rule. This is an
AUDIT: you change NO behavior and NO code; the only file you create is the
findings document.

This unit owns the surface by which a rum program **defines, inspects, and
protects symbols**: assignment, value tables, attributes, protection, options,
messages. It is the layer every other unit's builtin sits on, so an
integrity hole here is worth more than any single builtin's ergonomics.

## Count reconciliation (do this first, and state it)

The tracker never enumerated the kernel units. The authoritative derivation
(Claude ran it; you REPRODUCE it as your first verification step):

```
installed reference (docs/reference/builtins.md `### ` headings) = 612
stdlib catalog names (crates/rum-stdlib/src/catalog/*.rs BuiltinSpec:: entries) = 453
612 - 453 = 159 kernel builtins  →  units 8 (55) + 9-10 (104, split later)
```

**Unit 8 = these 55 names** (authoritative; one row MINIMUM per name,
grouped as below, this order):

- **Assignment / definition (7):** `Set`, `SetDelayed`, `TagSet`,
  `TagSetDelayed`, `TagUnset`, `UpSet`, `UpSetDelayed`
- **Compound assignment (11):** `AddTo`, `SubtractFrom`, `TimesBy`,
  `DivideBy`, `Increment`, `Decrement`, `PreIncrement`, `PreDecrement`,
  `AppendTo`, `PrependTo`, `ApplyTo`
- **Value tables / introspection / removal (12):** `OwnValues`,
  `DownValues`, `UpValues`, `SubValues`, `DefaultValues`, `ValueQ`,
  `Definition`, `Information`, `Clear`, `ClearAll`, `Remove`, `Unset`
- **Attributes / protection (10):** `Attributes`, `SetAttributes`,
  `ClearAttributes`, `Protect`, `Unprotect`, `HoldAll`, `HoldFirst`,
  `HoldRest`, `HoldAllComplete`, `SyntaxInformation`
- **Options (4):** `Options`, `SetOptions`, `OptionValue`,
  `ArgumentsOptions`
- **Messages (11):** `Message`, `Messages`, `MessageName`, `MessageTags`,
  `MessageDefinitions`, `$MessageList`, `Quiet`, `Check`, `CheckArguments`,
  `Off`, `On`

(The four `Hold*` names here are the ATTRIBUTE SYMBOLS. The `Hold` /
`HoldForm` / `HoldPattern` / `HoldComplete` HEADS are unit 9 — do not sweep
them.)

## Head start: what Claude already found (VERIFY, do not re-derive)

Four probe results, run before this brief. Reproduce each one as evidence in
your appendix, then build on them — they are the spine of the unit.

1. **The `Protected` enforcement point EXISTS and works.** `RandomInteger`
   (given `Protected` by brief-204) correctly rejects redefinition:
   `RandomInteger[x_] := 99` → `! SetDelayed::wrsym: Symbol RandomInteger is
   Protected.` and `RandomInteger[5]` still draws. The machinery is fine.
2. **The enforcement SURFACE is only 129 of 612.** 483 installed builtins
   carry no `Protected`. (Careful: in `docs/reference/builtins.md` each
   attribute is separately backticked — `` `Listable`, `Protected` `` — a
   regex that captures only the first backticked token silently
   under-reports. Claude hit exactly that bug; calibrate your instrument
   before you trust its count, then state the count you derived.)
3. **Core language heads are redefinable.** `Plus[a_, b_] := 42` SUCCEEDS,
   after which `1 + 2` → `42`. `Attributes[Plus]` = `{Listable, Orderless,
   Flat, OneIdentity}` — no `Protected`. Same shape for `Times`, `Power`,
   `Equal`, `If`, `Map`, `Part`, `And`, `Or`, `StringLength`.
4. **All 55 names of THIS unit lack `Protected`, including the protection
   machinery itself.** `Attributes[Set]` = `{}`; `Attributes[Protect]` =
   `{HoldAll}`; `Unprotect[q_] := 7` succeeds — the guard's own head can be
   redefined.

## The scope guardrail (READ TWICE — this governs the whole unit)

- **`Protected` is a MISTAKE-GUARD and a WL-conformance property — NOT a
  sandbox boundary.** In WL semantics a program can always `Unprotect[Plus]`
  and proceed; that is the design, and rum keeps it. So the finding to write
  is "rum's `Protected` coverage is 129/612 where WL protects essentially
  every `System\`` builtin" — an attribute-hygiene / conformance gap.
  Do NOT write it up as "the sandbox is broken", do NOT propose an
  unbreakable immutability mode, a `Locked` attribute, or per-request
  definition rollback HERE. True plugin isolation is the **host-embedding
  line's** problem (`docs/dev/dun-*-requirements.md`; per-request isolation
  is already on its gap list). Flag what this unit's data contributes to that
  line in one short paragraph, and stop there.
- **`Protected` is zero eval-cost, but other attributes are NOT.** Adding
  `Protected` was gated without an A/B three times (briefs 162b, 196, 204) —
  it is checked only on the definition-mutation path. `Hold*` / `Listable` /
  `Orderless` / `Flat` / `OneIdentity` / `NumericFunction` are read by CALL
  PLANNING on the hot path. brief-193's standing ruling: attribute additions
  that change argument evaluation or reordering are REJECTED unless
  independently justified. So: propose `Protected` freely where WL has it;
  for any OTHER attribute, mark the row as "touches call planning → needs the
  microbench A/B gate", and never propose one merely because WL lists it.
- **Special-form hold semantics may be HARD-CODED, not attribute-driven.**
  `Attributes[Set]` is `{}` yet `Set` plainly does not evaluate its first
  argument — the hold behavior lives in the evaluator/lowering, not in the
  attribute set. Before proposing "add `HoldFirst` to `Set`", TRACE where the
  hold actually happens (`lowering.rs`, the eval dispatch path) and say
  whether the attribute would be a no-op, a duplicate, or a behavior CHANGE.
  A blind WL-attribute-table transcription onto special forms is exactly the
  kind of change that breaks the default path. This trace is itself a
  deliverable.
- **The stateful↔pure separation line (briefs 200-202) does NOT apply to
  session-state writes.** That line governs reads of the OUTSIDE WORLD (wall
  clock, host entropy) — those need a spec-level capability. `Set`, `Off`,
  `On`, `$MessageList`, `Quiet` mutate SESSION state, which is snapshotable,
  resettable, and deterministic; `Capability::Pure` is CORRECT for them. Do
  not file "Off/On are stateful → need a capability" — that misreads the
  line. (What IS worth checking: does session state they touch survive
  `reset()` / snapshot correctly, and is `$MessageList` cleared where the
  contract says it is.)
- **Folk scope.** No `Dialog`-grade debugger, no full WL `Information`
  formatting, no `Definition` pretty-printer arms race. Good-enough,
  documented, honest.
- **Compat rule (standing user directive).** WL names keep WL-conformant
  DEFAULT behavior; improvements land as new options or rum-native siblings.

## What IS a legitimate finding here (the three axes, tuned to this unit)

- **Safety — the `Protected` enforcement surface (the main event).** Produce
  the full picture: which of the 612 installed builtins carry `Protected` and
  which do not, and classify the gaps. WL's rule of thumb is that every
  `System\`` builtin is `Protected`; the honest question for rum is which
  names, if any, are DELIBERATELY unprotected (a rum-native symbol meant to
  be user-overridable — if such a thing exists, name it and say why). Give me
  a table I can turn into a mechanical fix batch: the counts, the exceptions,
  and the per-catalog-file breakdown (which `catalog/*.rs` and which kernel
  `builtin_catalog/*.rs` files would need to change). Note where a head is
  registered via a constructor that hard-sets `AttributeSet::empty()` (the
  brief-204 shape) versus one that already threads attributes.
- **Safety — assignment fail-closed.** Does every definition-mutation path
  honor `Protected` — `Set`, `SetDelayed`, `TagSet(Delayed)`, `UpSet(Delayed)`,
  the 11 compound assignments, `Clear`, `ClearAll`, `Unset`, `Remove`,
  `SetAttributes`, `ClearAttributes`, `SetOptions`? A path that mutates a
  protected symbol's definition WITHOUT the `wrsym`-class message is a real
  safety finding (probe a representative set — the compound assignments are
  the likely leak, since they lower to `Set`). Also: `ClearAll` /`Remove` on a
  protected symbol; `SetAttributes[Plus, ...]` on a protected symbol.
- **Safety — destructive-operation blast radius.** `Clear["Global`*"]`-style
  wildcard forms, `Remove` semantics, and whether any of them can silently
  wipe kernel definitions. Fail-closed or documented, not silent.
- **Flexibility — WL-documented forms this surface lacks.** The likely
  candidates, to CHECK not assume: list forms (`Attributes[{a, b}]`,
  `SetAttributes[{syms}, {attrs}]`, `Protect`/`Unprotect` on lists and on
  string patterns like `Protect["Foo`*"]`, returning the list of names
  actually changed); `Options[f, name]` (single-option lookup) and
  `SetOptions` return value; `Clear` / `Remove` with string-pattern name
  arguments; `Definition` / `Information` output forms; `ValueQ` on non-symbol
  arguments; `MessageName` / `Messages` / `Off`/`On` selector forms
  (`Off[f::tag]`, `Off[]`, `Quiet[expr, {f::tag}]`, `Check[expr, fail, {tags}]`).
  Apply the operator-form disambiguation rule — most of these are data forms
  and will NOT qualify; say so where it applies.
- **Convenience / discoverability.** Are the value tables round-trippable
  (`DownValues[f]` output re-assignable via `DownValues[f] = …`)? Does
  `Information` / `Definition` show attributes, options, and messages? Is
  `Options[f]` discovery registered for the kernel heads that take options
  (the unit-6/197 pattern)? Is there a coherent story for "show me everything
  about this symbol", which is the single most-used REPL affordance in this
  family?
- **Forward-pointer (record, do not design):** TODO.md parks a
  `SubValuesHoldAll`-equivalent design for "the kernel units 8-10 audit".
  `SubValues` is in THIS unit — so record, factually, how rum's `SubValues`
  behaves today with respect to holding argument evaluation, as design input.
  One row + a short appendix note; no proposal.

Origin note: every name in this unit is WL-inherited EXCEPT `ApplyTo`,
`CheckArguments`, `MessageDefinitions`, `MessageTags`, `ArgumentsOptions`,
and `SyntaxInformation`'s rum-specific shape — verify each against your WL
knowledge and mark Origin honestly. For rum-native heads judge on the three
axes intrinsically, not on WL parity.

## Inputs

- **Name list**: the 55 above. Derive the 159-name kernel list yourself
  (reference minus stdlib catalog) and confirm the 55 are all in it and that
  104 remain for units 9-10 — that reconciliation is verification step 1.
- **Kernel registration**: `crates/rum-runtime/src/runtime/builtin_catalog/`
  — `namespace.rs` (31 of the 55), `rules.rs` (18: assignment + compound),
  `debug.rs` (`Off`, `On`), `scoping.rs` (the four `Hold*` attribute
  symbols). Also `stdlib_attributes.rs` and `metadata.rs` (the attribute
  mirror — brief-193/204 had to keep it in sync; understand the mechanism,
  it decides how big a `Protected` fix batch really is).
- **Implementations**: `crates/rum-runtime/src/runtime/definitions/`
  (`definition_updates.rs`, `definition_catalog.rs`, `definition_clear.rs`,
  `definition_tags.rs`, `definitions.rs`, `application/`);
  `attributes.rs`; `options/` (`core.rs`, `builtins.rs`, `arguments.rs`,
  `defaults.rs`, `patterns.rs`); `messages/` (`builtins_messages.rs`,
  `builtins_messages_inspect.rs`, `message_catalog.rs`, `message_format.rs`,
  `message_selector_parsing.rs`); `lowering.rs` (the compound assignments and
  special-form hold behavior lower here — the key file for the hard-coded-hold
  trace); `model/definitions.rs` (the session definition store).
- **Contracts**: `docs/reference/builtins.md` (all 55 have entries — the
  Attributes/Capability lines are the reference for the coverage table);
  `docs/guide/` chapters on definitions/attributes if present; `AGENTS.md`.
- **The `Attribute` enum** (`kernel_api`): enumerate which attributes rum
  ACTUALLY has. Do not propose attributes that do not exist in the enum
  (e.g. WL's `SequenceHold`, `NHoldAll`) — a missing attribute KIND is a
  separate, larger design question; note it, do not fold it in.
- **WL comparison**: documented WL surfaces from your knowledge (attribute
  tables, list/pattern forms, message selector forms). Design reference ONLY —
  never run an external engine.

## Findings file format

Write `docs/dev/usability-pass/kernel-a-findings.md`. Same as units 1-7 (the
seven existing findings files are the template): the seven-column table
(`Builtin | Origin | Verdict | Axis | Finding | Proposal | Size`), `OK` rows
REQUIRED (the all-rows table IS the coverage proof), one row per axis-gap;
then an **Evidence appendix** (verbatim probe transcripts, stdin fed:
`printf '' | cargo run -q -p rum-cli -- --eval '…'`; batch related checks;
`file:line` citations acceptable for code-visible gaps) and a **Triage
sketch** (2-5 candidate fix batches ranked by script-user value, noting which
touch call planning / attribute-driven planning and therefore need the A/B
gate).

PLUS — unique to this unit — a dedicated section:

### `## Protected enforcement surface (whole-registry)`

The 612-name coverage picture: counts (protected / unprotected, kernel vs
stdlib split), the per-file breakdown of where a fix would land, the
classification of unprotected names (should-be-protected vs
deliberately-open, with the exceptions named and justified), and the
mechanism note (which registration constructors hard-set
`AttributeSet::empty()`, and whether the runtime attribute mirror +
reference-drift test must be updated in lockstep). This section is the input
to a mechanical cross-cutting fix batch — make it complete and machine-usable,
not prose.

For every row, mark clearly whether an apparent gap is a REAL usability
finding or an intentional design boundary (folk scope / WL-conformant default
/ hard-coded hold / session-state-is-Pure — an `OK` row with a one-line
"intended" note, NOT a gap).

## Probe budget

Cap ~40 probe evals total (55 builtins — the largest kernel unit so far, but
the `Attributes[...]` sweep is one batched eval and the reference file answers
most attribute questions with no eval at all). Expected wall time 40-60 min.
Batch related checks into one `--eval` list (all the protected-symbol
fail-closed probes together; all the list-form flexibility probes together).
Feed stdin (`printf '' | …`) so the CLI does not block. Anything needing more
than the cap becomes an open question, not a guess.

## Verification (MANDATORY — run all, paste verbatim)

```
# 1. count reconciliation — paste the numbers
grep -c '^### ' docs/reference/builtins.md                    # 612
# your derivation of stdlib catalog names (453) and the kernel diff (159)
# assert: the 55 unit-8 names are all in the kernel 159; 104 remain

# 2. coverage — every one of the 55 names appears as a table row;
#    must print COVERAGE-OK 55/55 (paste the check script + output)

# 3. the Protected coverage count, derived with a CALIBRATED parser
#    (paste the parser and the counts; the reference's per-attribute
#    backticks are the trap)

git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/kernel-a-findings.md` — the ONLY file
  in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`, the
  tracker, the other findings files, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies, no
  code changes.
- Every `gap` row is evidence-backed: a probe transcript or a `file:line`
  citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines
  (Testing Policy, `AGENTS.md`).
- The design boundaries are hard lines: do NOT propose an unbreakable
  immutability mode / sandbox definition-locking here (that is the host line);
  do NOT propose non-`Protected` attribute additions without marking them
  "needs A/B"; do NOT propose new attribute KINDS; do NOT propose changing a
  WL name's default behavior.
- **Calibrate your instrument before trusting it.** This repo's standing
  lesson (`diagnostic-tool-trust`): a parser or probe that under-reports is
  worse than no data. Claude's first `Protected` count was wrong for exactly
  this reason. Cross-check any script-derived count against a handful of live
  `Attributes[...]` probes and say that you did.
- Stop-loss: if the same probe misbehaves twice, STOP and report it.

## Report format

1. Count reconciliation (612 / 453 / 159 / 55 / 104) and the coverage check
   output verbatim (55/55).
2. The `Protected` enforcement surface: the calibrated counts, the
   classification, and the per-file fix map — this is the headline.
3. Top gaps by script-user value, one line each, safety first (any
   definition-mutation path that does NOT honor `Protected`; any silent
   destructive form).
4. The hard-coded-hold trace for `Set` (where the hold really lives; what an
   attribute addition would actually do).
5. The triage sketch (mark which batches need the microbench A/B gate).
6. Verification outputs verbatim.
7. Stop-loss / open questions (empty if none).
