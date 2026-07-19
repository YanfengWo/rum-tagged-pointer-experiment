# Brief 269 — Sealed packages, batch 1: the mechanism spine

## Goal

Implement the `DefinePackage` source-unit registration mechanism per
`docs/dev/sealed-scoping-design.md` (rewritten 2026-07-19 — read THAT
version) §§1-3, 5, 7: the three-head registration chain
(`DefinePackage` + inert `Public`/`PackageScope`), the two-pass scan,
the sealed-identity allocator (bit-62 partition) + source-unit registry
(+ the session-snapshot struct ripple), the body rewrite/install with
failure cleanup and re-registration replace, the `mark_symbol_removed`
identity-partition guard, and the §5 source-reachability test set. When
you are done: a package's undeclared bound symbols are private —
reachable through the unit's own (rewritten) body, while every spelled
form of their name (direct source, `Symbol[...]`, `ToExpression`,
string-built) resolves to the ordinary `identity: None` symbol and
reaches no sealed binding. **The name resolver is not modified at all**
— `resolve_symbol_name` / `active_symbol_for_name` /
`symbol_generations` behavior is byte-identical; the seal works because
sealed identities are simply never registered there. Batch 1 does NOT
hide names (introspection hiding is batch 2); only reachability is in
scope. Snapshot emission (design §6) is batch 3 — batch 1 only stores
what §6 needs (the held wrapper form, epoch, indexes).

## Context pointers

- Read `AGENTS.md`, `docs/dev/sealed-scoping-design.md` (the 2026-07-19
  version), and `docs/dev/sealed-scoping/grounding-findings.md`
  (brief-268 — the substrate evidence this design stands on) before
  touching anything.
- Key files (roles):
  - `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs` —
    the kernel scoping catalog (`With`/`Module`/`Unique`/`Block`
    `BuiltinSpec`s with `.with_messages(...)` + `.protected()`).
    **`Module`'s entry is your template** for `DefinePackage`
    (`special_with_hold`); mirror its wiring end to end.
  - `crates/rum-runtime/src/runtime/builtin_adapters.rs` — adapter fns
    (line ~332: `session.eval_module(args)`); add the `DefinePackage`
    adapter and the two inert-head adapters here (mirror how `Hold`-like
    inert heads return their call unevaluated, if such an adapter
    exists; otherwise return `Expr::call(head.clone(), args.to_vec())`).
  - `crates/rum-runtime/src/runtime/scoping/lexical/module.rs` —
    `eval_module` + `resume_module_initializers`: the
    **suspension-safe sequential-evaluation pattern** your install loop
    must mirror (`suspend_with_result_frame_lazy` + a resume method that
    continues at index k+1).
  - `crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs`
    — `pub fn substitute_lexical_bindings(expr, bindings, shadowed)`:
    the body rewrite; reuse as-is (it handles nested lexical shadowing).
  - `crates/rum-runtime/src/runtime/resolution.rs` —
    `active_symbol_for_name` (~129): READ ONLY. You must not change it.
  - `crates/rum-runtime/src/runtime/definitions/definition_clear.rs` —
    `mark_symbol_removed` (~308): gains the identity-partition guard;
    `clear_symbol_state` (~253) and `remove_symbol_state` (~300): the
    cleanup calls (mirror `remove_symbol_state`'s all-`true` flags).
  - `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
    (~197-216) — the existing definition-target classification the scan
    reuses to find the bound symbol of an assignment form.
  - Session field four-site mirror pattern (grep `next_generated_symbol_id`
    — this brief-232-class ripple is mandatory):
    `model/session.rs:63` (internal snapshot struct),
    `session/session_state.rs:152` (decl) + `:403` (init),
    `session/session_helpers.rs:59` (snapshot clone) + `:88` (restore).
  - `crates/rum-core/src/expr.rs` — `Symbol::with_identity` (READ ONLY;
    `rum-core` is not touched by this brief).
- Acceptance is mechanical: the named tests + `cargo test --workspace`
  green + the microbench runs; prose claims decide nothing.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/scoping/packages.rs` — **NEW**: the
    mechanism (types, scan, allocator, install, session methods).
  - `crates/rum-runtime/src/runtime/scoping.rs` — mod decl only.
  - `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs` — the
    three new specs + message declarations.
  - `crates/rum-runtime/src/runtime/builtin_adapters.rs` — adapters.
  - `crates/rum-runtime/src/runtime/model/session.rs`,
    `crates/rum-runtime/src/runtime/session/session_state.rs`,
    `crates/rum-runtime/src/runtime/session/session_helpers.rs` — the
    new session fields, four-site mirror (MECHANICAL).
  - `crates/rum-runtime/src/runtime/definitions/definition_clear.rs` —
    the `mark_symbol_removed` guard only.
  - `crates/rum-runtime/src/runtime/tests/sealed_packages.rs` — **NEW**
    test file — plus the tests mod file that registers it.
  - `docs/reference/builtins.md` — regenerated output only (the way
    previous briefs regenerate it; the drift test defines the
    expectation; expected count 634 → 637).
  - CONDITIONAL, MECHANICAL ONLY (use only if a compile error or an
    existing test failure forces it; list each use in your report):
    `crates/rum-runtime/src/runtime/builtin_catalog.rs` (spec
    aggregation), attribute-mirror / vocabulary test files that pin the
    builtin name or attribute sets (kernel or rum-stdlib side), the
    docgen source if the reference regen requires a limits note.
- Files/areas you MUST NOT touch: `crates/rum-core/**` (the
  `Expr`/`Symbol` representation is frozen), `resolution.rs` outside of
  reading, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` other than
  the regenerated reference, anything under `docs/dev/codex/` except
  appending to your own report, `.git`, `Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path. The
  whole design point: no resolver change, no new per-lookup branch. The
  new session fields are consulted only by the new builtins.

## The mechanism, pinned (implement exactly this)

**Constants + fields.** In `packages.rs`:
`pub(in crate::runtime) const SEALED_IDENTITY_MARKER: u64 = 1 << 62;`
(bit 63 belongs to rule-RHS alpha ids, small per-name integers to
`Remove` generations — see grounding Q4). New session fields (all four
mirror sites): `sealed_units: Vec<SourceUnit>`,
`next_sealed_symbol_id: u64` (init 0), `next_source_unit_epoch: u64`
(init 0). `SourceUnit` (Clone): `package_context: String`, the ORIGINAL
held wrapper `Expr` (pre-rewrite, for the batch-3 snapshot),
`epoch: u64`, `public: Vec<Symbol>`, `package_scope: Vec<Symbol>`,
`private_index: BTreeMap<String, Symbol>` (readable name → sealed
symbol).

**Allocator.** `next_sealed_symbol_id += 1; Symbol::with_identity(
private_context, name, SEALED_IDENTITY_MARKER | next_sealed_symbol_id)`
where `private_context` = `` "Pkg`Private`" `` (the package context +
`` Private` ``).

**`DefinePackage[ctxString, args...]`** — kernel builtin, `Module`
wiring class, `HoldAllComplete` if the kernel spec API supports that
hold variant (else `HoldAll`; record the fallback), `.protected()`.

- Validate `ctxString`: a String atom, non-empty, ends with `` ` ``,
  valid per the existing context validation/normalization helpers →
  else message `argform`, return the call inert (standard invalid-call
  convention).
- **Declarations**: among the remaining held args, every top-level
  `Public[{syms...}]` / `PackageScope[{syms...}]` call is a declaration
  regardless of position; each head at most once (else `decl` message +
  `$Failed`); contents must be plain symbols whose context (as lowered)
  == the session's current context — a foreign-context name (e.g. a
  System-resolved name) is rejected with `decl` + `$Failed`. All other
  args, in order, are the body forms.
- **Pass 1 (scan, unevaluated)**: walk the body forms,
  `CompoundExpression`-flattened. Collect bound base symbols from the
  recognized definition-bearing forms: the assignment families
  (`Set`/`SetDelayed`/`TagSet`/`TagSetDelayed`/`UpSet`/`UpSetDelayed` —
  reuse the existing definition-target classification for the base/tag
  symbol), `SetAttributes`/`SetOptions`-class first-arg symbol targets,
  and `MessageName` assignment owners. Only symbols whose as-lowered
  context == the current context are package-local candidates;
  explicitly foreign-qualified symbols are references and are never
  rewritten. Unrecognized top-level forms contribute no bindings (the
  documented scan-determined contract; do not try to be clever).
- **Classify**: declared Public → target `Symbol::new("Pkg`", name)`;
  declared PackageScope → `Symbol::new("Pkg`PackageScope`", name)`;
  scan-bound ∖ declared → Private → sealed allocator. Declared but
  never bound is allowed (no error, no stub).
- **Pass 2 (install)**: build ONE `SymbolExprMap` (as-lowered
  package-local symbol → target symbol expr, covering all three
  tiers); rewrite each body form with
  `substitute_lexical_bindings(form, &map, &BTreeSet::new())`; evaluate
  the rewritten forms **sequentially and suspension-safely, mirroring
  `resume_module_initializers`** (`suspend_with_result_frame_lazy` + a
  resume method continuing at the next form — an `InputRequired`
  suspension must round-trip through a real resume frame, never be
  swallowed).
- **Failure contract**: any pre-install phase error (validation, scan,
  classify) happens before any state change — atomic. A body form
  evaluating to `Err` (other than the suspension flow): clear the state
  of every sealed symbol THIS registration minted
  (`clear_symbol_state` all-flags, mirroring `remove_symbol_state`'s
  call shape), record nothing in the registry, push message `failed`,
  return `$Failed` (the in-band value; not an EvalError).
- **Success**: push the `SourceUnit` (epoch = `next_source_unit_epoch`
  += 1) and return the package context as a String expr.
- **Re-registration** (same `package_context` already in
  `sealed_units`): clear the OLD unit's sealed symbols' state, replace
  the registry entry IN PLACE (same Vec slot — emission order is a §6
  contract), fresh epoch, fresh identities. Public/package symbols are
  simply overwritten by the new body (WL-style; no clearing).
- **`Public` / `PackageScope`** — kernel inert heads: `HoldAll`,
  `.protected()`, evaluation returns the call unevaluated. No messages.
- **`mark_symbol_removed` guard** (`definition_clear.rs`): skip the
  `symbol_generations` insert when the removed symbol's identity has
  the `SEALED_IDENTITY_MARKER` bit set (the write inherits removed
  identities via `max(current, removed) + 1`, which would move the
  name's generation into the sealed numeric region — a partition-
  hygiene defect). Everything else in the function is unchanged.
- **Messages** declared on the `DefinePackage` spec (`with_messages`):
  `argform`, `decl`, `failed` — every `push_message` tag you emit must
  be in the declared set (the registry's debug assertion enforces
  declared-or-exempt; an undeclared tag will fail tests).

## Deliverable

- The mechanism above, compiling and green.
- Tests in `sealed_packages.rs` (names indicative; write real
  assertions, not printouts). Use a standard test-session constructor;
  the probe package throughout can be:
  `DefinePackage["Pkg`", Public[{f}], f[x_] := helper[x] + 1;
  helper[x_] := x * 2]` (so `Pkg`f[3]` → 7 via the sealed `helper`).
  1. `public_reaches_sealed_private`: register; `Pkg`f[3]` evaluates
     to 7.
  2. `spelled_private_name_is_ordinary`: after registration, evaluate
     `` Pkg`Private`helper[3] `` (stays unevaluated — no binding),
     `Symbol["Pkg`Private`helper"]`, `ToExpression["Pkg`Private`helper"]`,
     and `ToExpression["Pkg`Priv" <> "ate`helper"]` — each resolves to
     a symbol whose `identity()` is `None` (assert at the Rust level)
     and is `!=` the registry's sealed symbol; applying it does not
     reach the sealed definition.
  3. `decoy_assignment_is_distinct`: assign
     `` Pkg`Private`helper = 99 ``; `Pkg`f[3]` still 7; the spelled
     name reads 99 (its own ordinary binding).
  4. `private_state_inside_package`: a package whose public setter
     writes a private own-value and whose public getter reads it —
     works through the public interface; the spelled private name still
     resolves ordinary.
  5. `package_scope_is_fqn_reachable`: a `PackageScope[{h}]` symbol is
     reachable as `` Pkg`PackageScope`h `` (convention tier pinned).
  6. `remove_sealed_writes_no_generation`: Rust-level — fetch the
     sealed symbol from the registry, `remove_symbol_state` on it,
     assert `symbol_generations` has NO entry for
     `` ("Pkg`Private`", "helper") `` and the spelled name still
     resolves with `identity: None`.
  7. `registration_failure_cleans_sealed_state`: a body whose later
     form fails (e.g. an uncaught `Throw` after a private definition):
     result `$Failed` + `failed` message; no registry entry; the minted
     sealed symbol's state is cleared; the spelled name is ordinary.
  8. `re_registration_replaces_in_place`: register v1 (`f` → old
     value), re-register with a changed body; `Pkg`f` follows the new
     body; the old sealed symbol's state is cleared; epoch increased;
     registry has ONE entry for the context.
  9. `declaration_validation`: duplicate `Public[...]` heads → `decl` +
     `$Failed`; a declared name that lowers into `System`` (e.g.
     `Public[{Map}]`) → `decl` + `$Failed`.
  10. `internal_snapshot_round_trips_registry`: mirror an existing
      internal snapshot/rollback test (the `symbol_generations`
      pattern): take the internal snapshot, register a package, restore
      the snapshot, assert `sealed_units` and both counters returned to
      the pre-registration values.
- Regenerated `docs/reference/builtins.md` (634 → 637) with the three
  new entries carrying summary + messages.

## rum / Rust pitfalls (read twice)

1. **The resolver is frozen.** If your diff touches
   `resolve_symbol_name` / `active_symbol_for_name` /
   `symbol_generation_key`, you have left the design — STOP.
2. **Suspension-safety is not optional.** The install loop mirrors
   `resume_module_initializers`; converting an `InputRequired` into a
   plain error or swallowing it corrupts the suspension stack.
3. **The four-site session-field mirror is mandatory.** Grep
   `next_generated_symbol_id`; your three new fields appear at every
   site it appears (decl, init, snapshot clone, restore). Missing the
   helpers file = internal rollback silently dropping sealed state.
4. **Message tags must be declared.** Every tag you `push_message` is
   in the spec's `with_messages` set, or the debug assertion fails the
   suite.
5. **Hold discipline.** `DefinePackage` args must arrive unevaluated
   (`special_with_hold`, `Module`'s class). If the body evaluated
   before the scan, every test here would still pass EXCEPT the
   held-ness ones — so test 1's `helper[x_] := x * 2` uses `:=`
   (SetDelayed) and test 4 proves state flows through the public
   interface only.
6. **No owned-tree deep clones in loops** — rewrite once per form,
   `Arc`-clone where the existing helpers do.
7. **Minimal diff.** No drive-by renames/reformat outside the listed
   files. `rum-core` untouched. No new `unsafe` anywhere.
8. **Stop-loss is real.** Same step failing twice for the same reason →
   STOP and report (working tree intact), do not thrash.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then the microbench, twice, on your candidate build (run-to-run noise
documentation — the true base-vs-candidate A/B is the director's gate
step, done with two separately built binaries):

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief269_run1
bash scripts/core-language-microbench.sh --output-dir /tmp/brief269_run2
```

Paste the per-label `rum_seconds` rows for the symbol/downvalue-tier
labels from both runs. Expected: ordinary run-to-run scatter only (this
change adds no code to any evaluated path that existing labels
exercise).

Also paste, verbatim: the test names + counts from the workspace run,
and the three reference entries as regenerated.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all
  changes in the working tree; the director gates and commits.
- Do NOT modify files outside Scope; if you believe you must, STOP and
  write that in the report instead.
- Touch NOTHING outside this repo; no network. Commands: file edits in
  Scope, `cargo`, the release `rum-cli`, the microbench scripts,
  `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no external-engine comparisons of any kind.
- **No new `unsafe`** (`rum-core`/`rum-runtime` are zero-`unsafe`).
- **Probe/timeout discipline**: bound long runs with
  `scripts/run-bounded.py`; derive timeouts from prior runs of the same
  shape.
- You MUST paste real verbatim outputs. A run that did not reach green
  is reported as such — never faked.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim: fmt, workspace test counts, the two
   microbench row sets, reference entries.
3. Verdict — mechanism behavior summary against the §5 reachability
   list; anything that deviates from this brief's pinned spec.
4. Stop-loss / open questions — where you stopped and why (empty if
   none). List any CONDITIONAL-file use with its forcing error.
