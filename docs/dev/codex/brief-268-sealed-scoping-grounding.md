# Brief 268 — Sealed-scoping substrate grounding (diagnostic only)

## Goal

This is a **diagnostic / grounding brief — NO source change**. The
sealed-scoping line (design: `docs/dev/sealed-scoping-design.md`) touches
symbol resolution, a core evaluation path, so before any mechanism is
written we ground the substrate empirically and map every surface the
mechanism will have to touch. When you are done, a new findings document
`docs/dev/sealed-scoping/grounding-findings.md` exists and answers, with
**verbatim** `rum-cli` transcripts and exact `file:line` citations, the
six grounding questions in **Deliverable** below. The deliverable is
evidence + a written mechanism map, not code.

The framing throughout is ordinary engineering hygiene: we are verifying
the **distinctness invariant of the `Symbol.identity` field**, measuring
the **default-path cost** of name resolution, and enumerating which
**introspection surfaces** enumerate user symbols. No behavior changes.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines) and
  `docs/dev/sealed-scoping-design.md` (the design this grounds) before
  touching anything.
- The design's §1 makes a **substrate claim that must be checked, not
  assumed**: it says a `Module` gensym "carries `Some(identity)`" and that
  sealing can "reuse `allocate_fresh_symbol`". Director's code read
  suggests `Module` gensyms are instead **name-mangled** (`x$NN`,
  `identity: None`) and that `allocate_fresh_symbol` produces name-mangled
  symbols, NOT identity-bearing ones. Q1/Q4 below exist to confirm or
  refute this empirically. Report what the code and probes actually show —
  do not paper over a discrepancy with the design doc.
- Key files (roles):
  - `crates/rum-core/src/expr.rs` — `Symbol` representation: `identity:
    Option<NonZeroU64>` (line ~38), `Symbol::new` → `None` (~113),
    `Symbol::with_identity` (~118), equality `id == id && identity ==
    identity` (~185), and the serde impls (~225 serialize, ~248
    deserialize).
  - `crates/rum-runtime/src/runtime/resolution.rs` — `resolve_symbol_name`
    (~101), `active_symbol_for_name` (~129, the point that consults
    `symbol_generations`), `symbol_exists` (~141), `symbol_generation_key`
    (~488).
  - `crates/rum-runtime/src/runtime/definitions/definition_clear.rs` —
    `mark_symbol_removed` (~308): the ONLY writer of `symbol_generations`
    (a `Remove`-generation counter).
  - `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`
    — `fresh_rule_rhs_alpha_symbol` (~525): the other `with_identity`
    producer, using a `RULE_RHS_ALPHA_ID_MARKER | id` high-bit region.
  - `crates/rum-runtime/src/runtime/scoping/lexical/module.rs` —
    `eval_module` and the reusable body rewrite
    `substitute_lexical_bindings(&body, &renames, &BTreeSet::new())` over a
    `SymbolExprMap` rename map.
  - `crates/rum-runtime/src/runtime/scoping/unique.rs` —
    `allocate_fresh_symbol` → `allocate_generated_symbol` →
    `generated_symbol_candidate` (~36) = `Symbol::new(context,
    "{name}${id}")`. Confirm this is name-mangled (`identity: None`).
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs` —
    `enumerated_names_symbols` (~267) is the symbol-enumeration source of
    truth for `Names`; `names_matching_patterns` (~186) with its
    `explicit_context` bypass (~196) and `visible_contexts` filter (~199);
    `exact_symbol_name_exists` (~314) backs `NameQ`.
  - `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs`
    — `eval_definition` (~4), `eval_information` (~16),
    `definition_expr_for_symbol` (~152), `information_data_expr_for_symbol`
    (~245): the `Definition` / `Information` / `?` display path.
  - `crates/rum-runtime/src/runtime/capabilities.rs` — `Capability::Diagnostic`
    (`1 << 5`), the introspection-reveal gate the design §4 reuses.
- Acceptance is mechanical: the six questions each answered with verbatim
  evidence + citations, and a mechanism map. Prose interpretation is
  secondary to the transcripts.

## Scope

- Files you MAY modify: **exactly one** — create
  `docs/dev/sealed-scoping/grounding-findings.md` (new file; new
  directory). Plus appending to your own report. **NO other file, no
  source change.**
- Files/areas you MUST NOT touch: everything else — all crate source,
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, other `docs/**`, anything under
  `docs/dev/codex/` except appending to your own report, `.git`,
  `Cargo.lock`.
- You MAY build and run the release `rum-cli` for probes (`cargo build
  --release -p rum-cli` if `target/release/rum-cli` is stale; it exists at
  brief time). You MAY run `cargo test`/`grep`/`rg` read-only to locate and
  confirm code. You MAY read any source file. You change no source.

## Deliverable

`docs/dev/sealed-scoping/grounding-findings.md`, answering each question
with (a) the exact `file:line` citations and (b) verbatim `rum-cli`
transcripts where a probe is asked for. Use the release binary; show the
exact command and its exact output for every probe.

**Q1 — Identity provenance and the distinctness invariant.**
Confirm from the code AND a probe:
- `Symbol::new` yields `identity: None`; `with_identity` yields
  `Some(id)`; equality requires BOTH `id` and `identity` to match
  (`expr.rs`). So two symbols with identical `context`+`name` text but
  different `identity` are **distinct and non-equal**.
- Enumerate EVERY producer of a `with_identity` (non-`None`) symbol in the
  codebase (`rg 'with_identity'`), and for each state which mechanism it
  belongs to (Remove-generation; rule-RHS alpha-rename; deserialization;
  anything else) and the numeric range of the `id` it uses.
- Probe: demonstrate that evaluated rum source has **no syntax** to
  construct a `with_identity` symbol directly — a name written in source,
  `Symbol["`Foo`x"]`, and `ToExpression["`Foo`x"]` all route through
  `Symbol::new` / `resolve_symbol_name` and land on `identity: None` unless
  `symbol_generations` already holds an entry for that `(context, name)`.
  Show the transcript that establishes this (e.g. that a freshly-spelled
  name has no non-`None` identity).

**Q2 — Is a `Module` gensym distinguished by NAME or by IDENTITY?** (This
directly checks the design §1 claim.)
- Probe: evaluate something like `Module[{x}, Hold[x]]` and show the
  returned form (expected: a name-mangled `x$NN`).
- Probe: take that exact printed name `x$NN` and show whether spelling it
  back in source (typed directly, and via `Symbol["...$NN"]` /
  `ToExpression`) reaches the SAME symbol the `Module` produced — e.g. by
  giving the escaped gensym an own-value inside the module and reading it
  back by the spelled name outside. Report clearly: **is a `Module`
  gensym's uniqueness carried by its mangled name (source-reachable if the
  name is known) or by an `identity` field (not source-reachable)?**
- State the consequence in one sentence: whether `allocate_fresh_symbol`
  as-is would produce a **source-reachable** private (i.e. unsuitable as a
  real seal) or an unreachable one.

**Q3 — The `symbol_generations` / `with_identity` resolution path is the
identity-distinctness substrate.**
- Probe the `Remove`-generation behavior: capture a symbol reference
  (e.g. `held = Hold[f]` after giving `f` a value), `Remove[f]`, then show
  that the captured `f` and a freshly-spelled `f` afterwards are now
  **distinct** (the captured one carries the pre-Remove generation; the
  fresh one a new `Symbol::new`). Paste the transcript.
- From the code, state precisely: after this brief's mechanism registers a
  private symbol with a fresh `identity` and **does NOT** write
  `symbol_generations` for its `(context, name)`, what does
  `active_symbol_for_name(context, name)` return for that name? (Expected:
  `Symbol::new(context, name)` → `identity: None` → a **different** symbol
  than the sealed one.) This is the crux of the mechanism; confirm it from
  `active_symbol_for_name`'s exact branches.

**Q4 — Reuse surface for the mechanism.**
- `substitute_lexical_bindings`: give its exact signature/location and
  confirm it rewrites an `Expr` body by replacing symbols per a
  `SymbolExprMap`. Confirm it is reusable as-is for rewriting a source
  unit's body to reference sealed symbols (it is the same operation
  `Module` uses).
- `allocate_fresh_symbol` / `allocate_generated_symbol`: confirm the exact
  symbol they build (`Symbol::new` with a `${id}` name suffix) and state
  whether they are reusable for the seal or whether a **new
  identity-minting allocator** is required. If a new allocator is required,
  identify: which session counter/field it would draw from
  (`next_generated_symbol_id` is the name-suffix counter; is there a
  distinct identity counter, or must one be added?), and which numeric
  regions it MUST avoid to stay disjoint from Remove-generations (small
  ints keyed per name) and rule-RHS-alpha ids (`RULE_RHS_ALPHA_ID_MARKER`
  region — report the marker's exact value/bit position from
  `binding_substitution.rs`).

**Q5 — Introspection enumeration surface (what a sealed private must be
hidden from in batch 2).**
- List every code site that enumerates or exposes user symbols by name,
  with `file:line`. Start from `enumerated_names_symbols` (which session
  maps it draws from — `definitions`, `attributes`, `options`,
  `declared_stubs`, `message_definitions`, …) and trace the display path
  through `names_matching_patterns` (note the `explicit_context` branch
  that bypasses the `visible_contexts` filter — this is the
  `Names["`Pkg`Private`*"]` exposure path the design §4 calls out),
  `exact_symbol_name_exists` (`NameQ`), `Contexts`, and the
  `Definition`/`Information`/`?` handlers in
  `builtins_messages_inspect.rs`.
- For each, state in one line WHERE a "hide sealed privates unless
  `Capability::Diagnostic`" check would attach (which function, which
  filter point). Do NOT implement it — just locate it.

**Q6 — Snapshot round-trip and default-path cost.**
- Confirm from `expr.rs` serde that a `with_identity` symbol serializes its
  `identity` and deserializes back to the same `with_identity` (so sealed
  privates round-trip like the existing identity-bearing symbols). If the
  CLI exposes a session save/restore probe, show a transcript round-tripping
  a symbol whose identity is non-`None`; otherwise cite the serde code and
  the existing gensym round-trip test that proves it.
- **Default-path cost audit:** read `active_symbol_for_name` and
  `resolve_symbol_name` and state exactly what they do per name lookup
  today (note that `symbol_generation_key` already allocates two `String`s
  per call — pre-existing cost, NOT introduced here). Then state the hard
  constraint for the mechanism: **when no sealed unit has been registered,
  name resolution must run byte-identically to today** — i.e. any sealed
  state must be an empty map/None consulted only when non-empty, never an
  unconditional added branch/allocation on the hot resolution path. Point
  to where the mechanism's per-lookup check would sit and confirm a
  "sealed state empty → early return, zero added work" shape is achievable
  there.

**Mechanism map (written proposal, no code).** After Q1–Q6, write a short
"Batch-1 mechanism map" section: the concrete plan the director will turn
into the mechanism brief — the seam for source-unit registration (the
design proposes a `DefinePackage[...]` wrapper form; note where a new
builtin head registers), the two-pass scan (collect Public/Package
declarations; everything else Private), the identity-minting seal step
(new allocator per Q4 + `substitute_lexical_bindings` body rewrite +
install into `definitions`/`attributes` WITHOUT touching
`symbol_generations`), the resolution outcome per Q3, and the exact list
of the §5 source-reachability tests that batch 1 must include (each showing
a spelled private name lands on a distinct `identity: None` symbol → no
binding reached). Flag any open decision you could not settle from the
code (e.g. the identity collision-region choice) as a question for the
director.

## rum / Rust pitfalls (read twice)

1. **No source change.** This brief produces a document and transcripts.
   If you find yourself editing a `.rs` file, STOP — that is out of scope.
2. **A probe is the exact statement — nothing added.** Show the literal
   `rum-cli` command and its literal output. Do not paraphrase output; paste
   it. If a probe does not behave as the design predicts, that IS the finding
   — report it, do not massage it.
3. **Confirm the discrepancy honestly.** The director's read (Q2/Q4) is that
   `Module` gensyms are name-mangled, not identity-bearing, so the design
   §1 wording is inaccurate and `allocate_fresh_symbol` is NOT directly
   reusable as a seal. If the probes confirm this, say so plainly with the
   transcript; if they refute it, say that instead with the transcript.
4. **Citations must be real.** Every `file:line` you cite must be one you
   actually opened. Prefer `rg -n` to locate, then read to confirm.
5. **Stop-loss is real.** If a probe cannot be constructed after two honest
   attempts, record what you tried and what blocked it, and move on — do
   not fabricate a transcript.

## Verification (MANDATORY — you run it; iterate to evidence)

Diagnostic brief: there is no test gate (no source change). The evidence IS
the deliverable. Produce `docs/dev/sealed-scoping/grounding-findings.md`
with the six answered questions + mechanism map, and in your final report
paste the KEY verbatim transcripts (at least: the Q2 Module-gensym
name/reachability probe, the Q3 Remove-generation distinctness probe, and
the Q1 source-cannot-set-identity probe) plus the `rg 'with_identity'`
producer list for Q1.

If you rebuilt the binary, paste the `cargo build --release -p rum-cli`
final status line. Do not run the microbench (nothing is timed here).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the new
  findings doc in the working tree; the director gates and commits.
- Do NOT modify any file outside the single in-scope doc. If you believe
  you must, STOP and write that in the report instead.
- Touch NOTHING outside this repo; no network. The only commands you run
  are: reading files, `rg`/`grep`, `cargo build`/`cargo test` (read-only
  use), the release `rum-cli` for probes, and writing the one findings doc.
- **Testing Policy is a hard line** (`AGENTS.md`): no comparisons against
  other engines, no oracle harnesses — not that this brief needs any; every
  probe is rum observing its own documented behavior.
- **No new `unsafe`, no source change at all.**
- Minimal footprint: the findings doc and your report only. No drive-by
  edits.
- You MUST paste real verbatim transcripts. If a probe did not produce the
  evidence, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — "diagnostic only, no source change; created
   `docs/dev/sealed-scoping/grounding-findings.md`".
2. Verification — the key verbatim probe transcripts (Q1 source-cannot-set,
   Q2 Module gensym name/reachability, Q3 Remove-generation distinctness)
   and the `with_identity` producer list.
3. The finding / verdict — per question, one-line answers; and explicitly:
   is the design §1 substrate claim (Module carries identity;
   `allocate_fresh_symbol` reusable) CONFIRMED or REFUTED by the evidence?
4. Stop-loss / open questions — anything you could not settle (expected:
   the identity collision-region choice for the new allocator; flag it for
   the director).
