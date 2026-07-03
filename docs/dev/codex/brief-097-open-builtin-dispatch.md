# Brief 097 — Open the builtin dispatch: delete the closed `BuiltinHandler` enum

## Goal

Stage 1 of the crate-rebalance line (`docs/crate-rebalance-plan.md`). Today every
builtin must be a variant of the closed ~470-variant `BuiltinHandler` enum
(`crates/rum-runtime/src/runtime/metadata.rs:277`), dispatched through a three-file
chained `match` and an exhaustive per-variant capability `match`. Replace this with
an OPEN representation: `BuiltinSpec` carries the handler as a plain fn pointer and
the capability as a data field. The enum, the three match files, and the capability
match are DELETED; after your change `grep -rn "BuiltinHandler" crates/` returns
ZERO hits. This is a pure representation refactor: ZERO behavior change
(`cargo test --workspace` green, capability sandbox behavior identical, microbench
neutral).

## Context pointers

- Read `AGENTS.md` and `docs/crate-rebalance-plan.md` first.
- Key files (all verified 2026-07-03; the enum is referenced ONLY in these):
  - `crates/rum-runtime/src/runtime/metadata.rs` — `BuiltinHandler` (line 277,
    ~470 variants) + `BuiltinSpec` (Copy struct: `name`/`summary`/`attributes`/
    `dispatch`/`handler: Option<BuiltinHandler>`) + its const constructors.
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` —
    `build_call_dispatch_plan` (resolves the spec ONCE per plan),
    `try_dispatch_builtin_call` (capability check via `handler.capability()`,
    profiling hooks, `sandbox_denied_builtin`), `run_builtin_handler`.
  - `crates/rum-runtime/src/runtime/eval_dispatch/{surface,symbolic,control_functional}.rs`
    — the chained match (~530 lines): each arm is `BuiltinHandler::X =>
    self.eval_x(head, args)`. Non-standard arms exist ONLY in `surface.rs` and
    `control_functional.rs`: args-only calls (`self.eval_trace(args)`,
    `self.eval_timing(args)`, `self.eval_print(args)`, …) and inline bodies
    (`Identity`, `HoldLiteral`, `Unevaluated`, `Evaluate`, `Sequence`).
    `symbolic.rs` is verified 100% standard-shape.
  - `crates/rum-runtime/src/runtime/capabilities.rs` — `Capability` /
    `CapabilityPolicy` (public, re-exported by rum-host — keep them) + the
    `impl BuiltinHandler { fn capability(self) }` exhaustive match (~500 lines,
    line 107) to delete. Non-Pure variants today: FileRead = {FindFile, FileNames,
    FileExistsQ, FileByteCount, FileDate, Import, Get, Needs}, FileWrite =
    {FileTemplateApply, Save, DumpSave}. Re-verify this list against the match
    BEFORE deleting it and paste the verified list in your report.
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs` + `builtin_catalog/*.rs`
    (11 files) — 513 const `BuiltinSpec` entries naming `BuiltinHandler::X`.
  - `crates/rum-runtime/src/runtime/session_state.rs` — `CallDispatchPlan`
    (field `builtin_handler: Option<BuiltinHandler>`, line ~35) + the
    `run_call_dispatch_stage` plumbing; `EvalError` (public).
  - `crates/rum-runtime/src/runtime/resolution.rs` —
    `cached_builtin_dispatch_spec_for_symbol` returns `Option<BuiltinSpec>` BY
    VALUE; `crates/rum-runtime/src/runtime/model.rs` — `BuiltinSpecMap` stores
    `BuiltinSpec` by value. Copy-ness of `BuiltinSpec` must be preserved.
  - `crates/rum-runtime/src/runtime/eval_core.rs` uses the cached spec only via
    `.is_some()` — no variant-identity reads anywhere outside the dispatch match.
- `BuiltinSpec` has NO serde; snapshots never serialize handlers. All types in the
  new fn-pointer signature (`Session`, `Expr`, `EvalError`) are already `pub` via
  `runtime/mod.rs`'s `pub use`.

## The design (follow it; if a step is impossible, STOP and report)

1. In `metadata.rs` add
   `pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>;`
2. `BuiltinSpec`: `handler: Option<BuiltinFn>`; NEW field `capability: Capability`.
   All existing const constructors set `capability: Capability::Pure`; add
   `pub const fn with_capability(mut self, capability: Capability) -> Self`.
   Keep `Clone, Copy, Debug`. (`PartialEq/Eq`: see pitfall 4.)
3. Catalog entries: replace `BuiltinHandler::X` with the handler fn.
   - Standard arms: use the method path directly, e.g.
     `BuiltinSpec::ordinary("Plus", "…", Session::eval_plus)`. A fn item coerces
     to a fn pointer in const arrays, and `pub(super)`/`pub(in crate::runtime)`
     methods are visible in `builtin_catalog/*` (descendant of `runtime`).
   - Non-standard arms (args-only + inline bodies, ~25 total): write adapter fns
     in ONE new file `crates/rum-runtime/src/runtime/builtin_adapters.rs`, named
     `builtin_<snake_name>`, signature exactly `BuiltinFn`, body EXACTLY the old
     match arm's body (e.g. `builtin_timing` calls `session.eval_timing(args)`;
     `builtin_evaluate` keeps its `suspend_with_frame` shape verbatim). Register
     the module in `runtime/mod.rs` alongside its siblings. Do NOT change any
     `eval_*` method signature or body in this brief — adapters absorb every
     mismatch.
   - The 11 non-Pure entries get `.with_capability(Capability::FileRead)` /
     `FileWrite` per the verified list.
4. `CallDispatchPlan` / dispatch plumbing: replace the
   `builtin_handler: Option<BuiltinHandler>` carriage with the resolved dispatch
   data — recommended: `Option<BuiltinSpec>` (it is Copy and already resolved once
   in `build_call_dispatch_plan`). Constraints: stays Copy-cheap, NO per-call
   HashMap re-lookup (keep the resolve-once-at-plan-build shape), the capability
   check in `try_dispatch_builtin_call` reads the spec field, and
   `sandbox_denied_builtin`'s `format!("{handler:?}")` fallback becomes the spec
   `name`.
5. Delete: the `BuiltinHandler` enum; the `impl BuiltinHandler` block in
   `capabilities.rs`; `run_builtin_handler_surface/_symbolic/_control_functional`
   and their files (`surface.rs`, `symbolic.rs`, `control_functional.rs` should end
   up deleted; whatever survives of `run_builtin_handler` is a one-line fn-pointer
   call). Let the compiler drive the remaining signature fixes.
6. Final grep gate: `grep -rn "BuiltinHandler" crates/` → zero hits.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/metadata.rs`
  - `crates/rum-runtime/src/runtime/capabilities.rs`
  - `crates/rum-runtime/src/runtime/eval_dispatch/` (all files; deleting
    `surface.rs`/`symbolic.rs`/`control_functional.rs` is expected)
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs` and
    `crates/rum-runtime/src/runtime/builtin_catalog/*.rs`
  - `crates/rum-runtime/src/runtime/builtin_adapters.rs` (NEW)
  - `crates/rum-runtime/src/runtime/session_state.rs`,
    `crates/rum-runtime/src/runtime/resolution.rs`,
    `crates/rum-runtime/src/runtime/session_api.rs`,
    `crates/rum-runtime/src/runtime/model.rs`,
    `crates/rum-runtime/src/runtime/eval_core.rs`,
    `crates/rum-runtime/src/runtime/mod.rs` — ONLY the mechanical carriage/type
    edits the compiler forces + the new `mod` line; list every such edit in the
    report.
  - Existing test files under `crates/**` ONLY if a test names `BuiltinHandler`
    directly (report which).
- Files/areas you MUST NOT touch:
  - the `eval_*` builtin implementations across the runtime (bodies AND
    signatures) — this brief moves NO implementation code;
  - anything under any `mma-hol` clone (NEVER edit the acceptance standard);
  - `crates/rum-core/**` (`Expr`/`Symbol` representation), any serde shape;
  - `rum-host`/`rum-cli`/`rum-repl`/`rum-jupyter`/`rum-stdlib` sources — they must
    compile UNCHANGED against the new runtime (their `Capability`/
    `CapabilityPolicy` re-exports keep working);
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending your own
    report; `.git`, `Cargo.toml`/`Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path (no new
  allocation/branch/lock per call; the fn-pointer call replaces the match — that
  is the whole runtime delta).

## Deliverable

- The refactor described above, in the working tree (no commit).
- Verified capability list extracted from `capabilities.rs` before deletion.
- `cargo fmt --check` + `cargo test --workspace` verbatim totals.
- Microbench base-vs-new per-label table (see Verification).
- The zero-hit `grep -rn "BuiltinHandler" crates/` output.

## rum / Rust pitfalls (read twice)

1. **Behavior-identical means identical.** Message text, evaluation order,
   sandbox-denial results (`$Failed` + `General::sandbox`), profiling hooks
   (`record_profile_builtin_application`, `builtin_timing_start`) all stay exactly
   as they are. Any test-output difference is a defect, not a drift to accept.
2. **Suspension-safety.** The `Evaluate` arm's `suspend_with_frame` body must move
   into its adapter VERBATIM — do not collapse it into an eager call.
3. **Copy-ness.** `BuiltinSpec` is stored by value in `BuiltinSpecMap` and the
   dispatch-spec cache; `Option<BuiltinFn>` keeps it Copy. Do not box anything.
4. **fn-pointer comparisons.** Derived `PartialEq`/`Eq` on structs now containing a
   fn pointer (`BuiltinSpec`, `CallDispatchPlan`) may trigger the
   `unpredictable_function_pointer_comparisons` lint. All copies of a spec
   originate from one catalog entry, so pointer equality is stable here: if the
   lint fires, keep the semantics and silence it locally at the type with a
   one-line comment (or a manual `PartialEq` via `std::ptr::fn_addr_eq`) — do NOT
   restructure the code around the lint, and do NOT add a crate-level allow.
5. **Const-context coercion.** `Session::eval_plus` (a fn item) coerces to
   `BuiltinFn` inside `const NUMERIC_BUILTINS: &[BuiltinSpec]`. If some method's
   receiver/signature does not match exactly, that entry belongs in
   `builtin_adapters.rs` — do not "fix" the method.
6. **One variable per measurement**; profiling env OFF for all timing numbers.
7. **Stop-loss is real.** Same failure twice, or a microbench regression you cannot
   attribute → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

BEFORE touching any file, capture the baseline (ONE run — probe budget):

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_097_micro_base
```

Then after the refactor:

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-host        # spotlight: capability-policy tests must pass unchanged
grep -rn "BuiltinHandler" crates/   # must print NOTHING
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_097_micro_new
```

Paste verbatim: the workspace test totals, the empty grep, and a per-label
base-vs-new seconds table from the two `measurements.tsv` files. Sub-millisecond
labels jitter run-to-run; judge the stable tiers. Any hot-tier label consistently
worse than ~±8% is a defect to investigate or report — not to ship silently.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in the
  working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write that
  in the report instead.
- Full machine access, but touch NOTHING outside this repo, no network. Only file
  edits within Scope, `cargo`, the microbench script, and `python3` for parsing.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- **No new `unsafe`** in `rum-core`/`rum-runtime` (zero today).
- **Hot-path A/B is mandatory here** — this brief changes the innermost builtin
  dispatch (match jump → indirect call); the microbench table above is the gate.
- Probe budget: the two microbench runs above are the budget; ONE re-run of a
  suspicious label is allowed, then report.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside the
  task. (The 513 catalog entries + adapters + deletions are the task.)
- You MUST paste real verbatim verification output. If a run did not reach green,
  say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why; the verified
   capability list; every compiler-forced edit outside the core files.
2. Verification — each command with exact verbatim output (test totals, empty
   grep, microbench table).
3. The verdict — behavior-identical claim + the microbench delta on the hot tiers.
4. Stop-loss / open questions (empty if none).
