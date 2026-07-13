# Brief 223 — Usability audit: kernel C (system state / contexts / packages / paths / diagnostics / tests; 57 builtins)

Audit brief — sweep unit 10 of the Usability Pass: the THIRD and FINAL
kernel unit, closing the whole pass. Read `docs/dev/usability-pass.md`
FIRST: the three axes, the compat rule, the coverage discipline, and the
closed-unit ledgers (units 1-9). This is an AUDIT: you change NO behavior
and NO code; the only file you create is the findings document.

This unit owns the surface by which a rum program **touches the session's
surroundings**: contexts and packages, file-path math and loading, timing
and memory, diagnostics and output, tests, and the interactive-input
heads. Its center of gravity is the CAPABILITY-LABEL review — the
FS-purity line (no ambient authority) and the stateful↔pure separation
line (briefs 200-202) are LAW here, and this unit is where their
enforcement surface lives.

## Count reconciliation (do this first, and state it)

```
installed reference (### headings) = 612
stdlib catalog names = 453  →  612 − 453 = 159 kernel
159 − 55 (unit 8) − 47 (unit 9) = 57 (THIS unit — the exact complement
pinned in brief-214; reproduce the derivation and assert equality)
```

**Unit 10 = these 57 names** (authoritative; one row MINIMUM per name,
grouped as below, this order):

- **Contexts / packages (12):** `$Context`, `$ContextPath`, `$Packages`,
  `Begin`, `End`, `BeginPackage`, `EndPackage`, `Context`, `Contexts`,
  `DeclarePackage`, `Needs`, `ResolveContextAliases`
- **Symbol table (3):** `Names`, `NameQ`, `SymbolName`
- **Paths / files / loading (11):** `$Path`, `$InputFileName`,
  `DirectoryName`, `ParentDirectory`, `FileNameJoin`, `FileNameTake`,
  `FindFile`, `File`, `Get`, `Save`, `DumpSave`
- **Timing / memory / constraints (11):** `AbsoluteTime`,
  `AbsoluteTiming`, `Timing`, `RepeatedTiming`, `TimeConstrained`,
  `Pause`, `MemoryConstrained`, `MemoryInUse`, `MaxMemoryUsed`,
  `ByteCount`, `EvaluationData`
- **Diagnostics / output (8):** `Print`, `Echo`, `EchoFunction`,
  `EchoLabel`, `Monitor`, `Trace`, `TracePrint`, `Stack`
- **Profiling (2):** `RumProfileReset`, `RumProfileSnapshot`
- **Interactive input (3):** `Input`, `InputString`, `Dialog`
- **Tests (2):** `TestCreate`, `TestReport`
- **Markers / data heads (5):** `$Aborted`, `$Canceled`, `ByteArray`,
  `DiffObject`, `TemplateObject`

## Head start: what Claude already found (VERIFY, do not re-derive)

1. **Attributes**: 57/57 in the reference; only `$ContextPath`/
   `$Packages`/`$Path` lack `Protected` (the brief-210 writable-session-
   var exclusions; post-211 they honor an explicit `Protect`). Protection
   is NOT this unit's battleground.
2. **Capability map** (from the reference; the unit's spine):
   `Clock` (8): AbsoluteTime, AbsoluteTiming, EvaluationData, Pause,
   RepeatedTiming, **TestReport**, TimeConstrained, Timing.
   `Diagnostic` (4): MaxMemoryUsed, MemoryInUse, RumProfileReset,
   RumProfileSnapshot. `FileRead` (3): FindFile, Get, Needs.
   `FileWrite` (2): DumpSave, Save. `Pure` (40): everything else —
   including **`Input`, `InputString`, `Dialog`**.
3. **Probed green**: `Begin`/`End` context switching + `$Context`;
   `Names`/`NameQ`/`SymbolName`; pure path math (`FileNameJoin[{"a","b",
   "c.txt"}]` → `"a/b/c.txt"`, `DirectoryName`, `ParentDirectory`,
   `FileNameTake` — no FS touch); `Echo[42]` prints and returns 42;
   `Print` emits + `Null`; `$Aborted`/`$Canceled` inert; `ByteCount`
   works; `TestCreate[1+1, 2]` returns an **Association**; `Stack[]` →
   `{}` at top level.
4. **THE flagged finding — `Input[]` under a closed stdin**:
   ```
   printf '' | rum-cli --eval '{Input[], InputString["p> "], Dialog[]}'
   ! input required: ?
   error: input required: ?     (process EXIT CODE 0)
   ```
   Two distinct rows. (a) **Label/behavior inconsistency**:
   `Input`/`InputString`/`Dialog` are declared `Capability::Pure`, yet
   their behavior is to suspend the evaluation and ask the embedding
   application for a value — i.e. they depend on an interactive
   surrounding, which a `Pure` declaration says they do not. This is the
   same LABEL-KIND question the unit-4 `Exit` row raised (`Exit` was
   `Pure` while requesting host lifecycle control; the user relabelled it
   `Capability::Process`). Map the mechanism factually — the
   `InputRequired` suspension path, which capability profiles install
   these heads today, and what the embedding API does with the request
   (cite the rum-host tests) — classify honestly, and record it as a
   DECISION candidate. The Exit precedent was a USER decision; do NOT
   propose a unilateral relabel. (b) **Exit-code contract**: the CLI
   printed `error: input required` yet the process exited 0; `rum-cli`'s
   documented contract is "1 on syntax/load/evaluation failure". Confirm
   and file the mismatch.

## The scope guardrails (READ TWICE)

- **Capability labels are the unit's correctness axis.** This is an API-
  hygiene review, in the same spirit as the attribute-hygiene sweeps of
  units 6-8: does each head's DECLARED label match what the code
  actually does? For every non-Pure head: is the label the right KIND
  (Clock for wall-clock reads, Diagnostic for process introspection,
  FileRead/FileWrite for filesystem access)? For every Pure head:
  certify it reads NOTHING from outside the session — session-mediated
  output (`Print`/`Echo`/`Trace` accumulate in the session response) is
  Pure by the session-state doctrine (the Sow/Reap precedent), but
  VERIFY the output really flows through the session rather than direct
  stdout in kernel code (cite the seam). `MemoryConstrained` is `Pure`
  while `MemoryInUse` is `Diagnostic`: trace it — if budget enforcement
  reads process memory, the label is wrong; if it meters allocations
  internally, document why that is deterministic.
- **`TestReport` Clock review is ORDERED** (flagged by the stateful↔pure
  line at close, 2026-07-08): test-RUNNING is pure; only the report's
  elapsed-time field needs Clock. Record the design input (could timing
  be optional / a separate reader?) — a decision candidate, not a
  redesign here.
- **Package/context surface = design input for the parked Structured
  Package Format scoping half** (TODO.md WL15 item (d)): record factually
  how `Begin`/`BeginPackage`/`End`/`EndPackage`/`Needs`/`Get`/
  `DeclarePackage`/`$ContextPath` interact with symbol visibility TODAY
  (creation vs lookup, what `Private`` conventions exist, what
  `ResolveContextAliases` actually does — it is rum-native). Dedicated
  appendix section; record, do NOT design.
- **The loader path is mma-hol's spine.** `Get`/`Needs`/`$Path`/
  `$InputFileName` are exactly what the acceptance suite exercises for
  ~80 minutes. Any fix-batch proposal touching them must carry the flag
  "needs the mma-hol Strict gate at the director's gate" in the triage
  sketch.
- **`FileNames` is ABSENT** (discovered at the unit-9 mma-hol gate: the
  suite's discovery idiom silently no-opped). Record it as a
  candidate-addition row (long-tail class, FileRead capability, folk
  glob) — a candidate list entry, not an auto-gap; note the mma-hol
  run_all.wls consumer as the motivating case.
- **Folk scope.** `Trace`/`TracePrint`/`Stack`/`Dialog` stay folk — no
  stepper/debugger arms race, no WL `Dialog` subsession semantics unless
  they already exist (establish what rum's `Dialog` actually IS first).
  `DumpSave` is rum's snapshot format — document, don't extend. No
  package-manager ambitions on `Needs`.
- **Profiling heads are rum-native Diagnostic** — judge intrinsically;
  the env-gated cold-load pattern is law (no default-path cost); verify
  their no-op shape when profiling is off.
- **Session-state writes are Pure — correctly** (`Begin`/`End`/
  `DeclarePackage` mutate snapshotable session state). Do not file
  capability findings for context mutation. What IS worth checking:
  does context state survive `reset()`/snapshot correctly; does an
  unbalanced `Begin` without `End` leak the context switch past the
  request boundary (host isolation seam — record for the dun line).
- **Compat rule** (standing): WL names keep WL-conformant defaults;
  improvements via options or rum-native siblings. rum-native heads
  (`ResolveContextAliases`, `RumProfile*`, `DiffObject`,
  `TemplateObject`, `TestCreate`'s Association shape) judged
  intrinsically.
- **String-keyed option-discovery wart**: if any of these heads
  registers string-keyed options, row it and defer to the host line
  (standing).

## What IS a legitimate finding (three axes, tuned)

- **Safety**: label/behavior mismatches (the main event — the Input
  family is the worked example); silent failure modes in the loader
  (`Get` on a missing/malformed file — message + `$Failed`?);
  `Save`/`DumpSave` overwrite semantics (silent clobber?);
  `TimeConstrained`/`MemoryConstrained` budget-exceeded shape (WL returns
  `$Aborted` — and rum has the `$Aborted` marker in THIS unit; verify the
  pairing); CLI exit-code honesty (the Input finding); `Pause` bounds
  (negative / huge arguments).
- **Flexibility**: WL-documented forms to CHECK not assume —
  `Timing`/`AbsoluteTiming` return shapes; `TimeConstrained[expr, t,
  failexpr]` 3-arg form; `MemoryConstrained` 3-arg; `Names["patt`*"]`
  forms already landed in 213 (verify they cover context patterns);
  `Get`/`Needs` context-path resolution order (`$Path` honored? probe
  with a scratch file under /tmp); `Begin`/`BeginPackage` nesting and
  error recovery (unbalanced `End`); `Echo[expr, label]` /
  `EchoFunction[f]` WL forms; `SymbolName` on qualified names.
- **Convenience**: `Contexts[]` filtering; `Information`-style
  discoverability for packages; `TestReport`'s output shape usefulness
  (fields, failure detail); `Echo` label ergonomics.
- **Origin marking**: `ResolveContextAliases`, `RumProfileReset`,
  `RumProfileSnapshot`, `DiffObject`, `TemplateObject`, `TestCreate`
  (+`TestReport`'s rum shape) are rum-native; `$Aborted`, `$Canceled`,
  `ByteArray`, `File` and the rest are WL-inherited — verify each and
  mark honestly.

## Inputs

- **Registration**: `crates/rum-runtime/src/runtime/builtin_catalog/`
  (namespace.rs holds most of this unit; debug.rs the timing/echo group;
  find the exact spread and state it).
- **Implementations**: `runtime/namespace_tools/` (+ `names.rs`),
  `runtime/package_tools.rs`, `runtime/context_and_load_helpers.rs`,
  `runtime/builtins_context_loader*`, `runtime/file_io.rs` (the FileIo
  provider seam), `runtime/debug_tools/`, `runtime/profiling/`,
  `runtime/builtins_input.rs` (the Input family), `rum-host/src/`
  (sandboxed() capability sets, session isolation), `rum-cli/src/main.rs`
  (exit codes).
- **Contracts**: `docs/reference/builtins.md`; `docs/guide/` chapters on
  capabilities/sandboxing (the design-boundaries guide); `AGENTS.md`;
  `DISCLAIMER.md`.
- **WL comparison**: documented knowledge only; never run an external
  engine.

## Findings file format

Write `docs/dev/usability-pass/kernel-c-findings.md`. Same as units 1-9:
seven-column table (`Builtin | Origin | Verdict | Axis | Finding |
Proposal | Size`), OK rows REQUIRED (all-rows = coverage proof), one row
per axis-gap; **Evidence appendix** (verbatim probe transcripts, stdin
fed; `file:line` citations for code-visible facts); **Triage sketch**
(2-5 candidate fix batches ranked by script-user value, marking which
need the A/B gate and which need the mma-hol Strict gate).

PLUS — unique to this unit — TWO dedicated sections:

### `## Capability-label review (whole-unit)`

The 57-row table: declared label, what the implementation actually
touches (cite the seam), and a verdict (label matches behavior /
mismatched / decision-candidate). Add one column: which capability
profiles install the head today (the restricted `sandboxed()` profile vs
the trusted CLI profile), so a reader can see at a glance where a
label/behavior mismatch would surface. The Input-family analysis lives
here. This table is the input to a possible label fix batch and the
first concrete deliverable for the dun host-embedding line.

### `## Package-surface design input (parked SPF scoping half)`

Factual record: symbol creation vs lookup across `Begin`/`BeginPackage`/
`$ContextPath`; what loading (`Get`/`Needs`) does to contexts; what
`DeclarePackage`/`ResolveContextAliases` actually implement; where
visibility conventions (`Private``) stand today. Complete enough that
the parked Structured-Package-Format design pass never re-derives it.

## Probe budget

Cap ~45 probe evals (57 names; the reference answers the attribute and
capability questions with no eval). Batch related checks in `{...}`
lists; `Input`/`Dialog` probes go in their OWN commands (closed stdin, a
hang means timeout — report it); `Pause`/`TimeConstrained` probes use
sub-second budgets; `Save`/`DumpSave`/`Get` probes use /tmp scratch
paths ONLY (never repo paths). Expected wall time 45-60 min. More than
the cap becomes an open question, not a guess.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c '^### ' docs/reference/builtins.md          # 612
# the 453/159/55/47/57 derivation with the complement equality assert
# coverage check: COVERAGE-OK 57/57 (script + output pasted)
git status --short          # ONLY the new findings file (+ /tmp scratch)
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/kernel-c-findings.md` — the ONLY
  file in the repo diff. Scratch files under /tmp only.
- MUST NOT touch: any crate code, tests, guide, reference, TODO.md, the
  tracker, other findings files, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no code changes.
- Every `gap` row is evidence-backed (probe transcript or `file:line`).
  Never claim a behavior you did not run. Baseline-verify before
  asserting any "X already does Y" row (the unit-9 standing lesson).
- WL comparison from documented knowledge only.
- Design boundaries are hard lines: no capability relabels proposed as
  done-deals (decision candidates only, with the Exit precedent cited);
  no debugger/package-manager designs; no SPF design (input only); no
  proposals that add default-path cost to profiling-off runs.
- Stop-loss: if the same probe misbehaves twice, STOP and report it. If
  an `Input`/`Dialog` probe hangs past the timeout, report the hang as a
  finding rather than retrying variations.
- **Tooling note (rev2)**: the first dispatch of this brief was killed
  mid-run by an upstream content filter in the Codex CLI while it was
  running rum-host capability tests — no file was written. This brief is
  a routine API-hygiene audit of an open-source expression evaluator's
  own declared capability labels (the same kind of sweep units 6-8 ran
  for attributes). Keep your write-ups in that engineering register:
  describe what a label says versus what the code does. If a filter
  interrupts you again, WRITE THE FINDINGS FILE FIRST with whatever you
  have (partial is fine, mark it partial) before running further test
  commands, so the work is never lost.

## Report format

1. Count reconciliation (612/453/159/55/47/57 + complement equality) and
   COVERAGE-OK 57/57, verbatim.
2. The capability-label review headline: mislabels and decision
   candidates, the Input-family analysis first.
3. Top gaps by script-user value, one line each, safety first.
4. The package-surface design-input summary (three sentences max here;
   the appendix carries the detail).
5. Triage sketch (A/B and mma-hol gate flags marked).
6. Verification outputs verbatim.
7. Stop-loss / open questions (empty if none).
