# Brief 114 — file-slimming batch 1: profiling.rs (138KB) + stdlib catalog.rs (74KB)

## Goal

First batch of the file-slimming line (opened after the crate split completed).
Split the two largest debt files — `crates/rum-runtime/src/runtime/profiling.rs`
(138,027 chars) and `crates/rum-stdlib/src/catalog.rs` (74,489 chars) — into
module directories per the SPLITTING PRINCIPLES below. This is PURE
ORGANIZATION: zero behavior change, zero signature change, verbatim moves;
`cargo test --workspace` green (currently 1301/0); both target files replaced
by directories whose every file is ≤20k chars (ideally ≤10k).

## THE SPLITTING PRINCIPLES (user-stated; they govern every decision)

1. **Split by LOGIC and FUNCTION, never mechanically.** Each new module has ONE
   describable responsibility. A cohesive 22k module beats two broken 11k
   halves — the size policy (`docs/code-organization-guidelines.md` §File Size
   And Splitting Policy; unit = CHARACTERS; <10k preferred, 10-20k warning,
   20-35k split-plan, >35k debt) is a maintenance target, not a mechanical law.
   If a module you form stays >20k for cohesion reasons, keep it and justify it
   in the report.
2. **Optimize for incremental compilation**: separate high-churn regions from
   stable ones (e.g. counter/flag DECLARATIONS are stable; report FORMATTING
   churns), and never create circular references between sibling submodules —
   dependency flow inside the directory must be a DAG.
3. **Optimize for FUTURE REFACTORING**: boundaries must align with plausible
   future ownership moves. profiling submodules should be able to move out or
   be feature-gated WHOLESALE later (its stay/feature-gate fate is an open
   decision — your boundaries must not prejudge it but must not obstruct it
   either). catalog files split BY FAMILY because the family is the unit of
   future catalog-level change.
4. **Pure organization**: function/struct/static bodies move VERBATIM; no
   renames, no signature changes, no visibility widening beyond what module
   boundaries force (prefer `pub(super)`; the facade `mod.rs` re-exports so
   every EXTERNAL import path keeps working unchanged — callers outside the
   split directory must not change except possibly their `use` lines, and not
   even those if the facade mirrors the old surface).
5. **SPLIT MAP FIRST**: before editing, write the split map (target module →
   one-line responsibility → approx char size) and put it at the TOP of your
   report. If the map's reasoning changed during execution, show both.

## Context pointers

- Read `AGENTS.md` and `docs/code-organization-guidelines.md` (the policy
  section) first.
- `profiling.rs` (138k, 151 fns): env-gated cold-load diagnostic
  instrumentation — the `RUM_PROFILE_*` `AtomicU8` flag pattern, dozens of
  `AtomicU64` counters grouped by instrument domain (lexical substitution,
  module/scoping, argument substitution, builtin timing, rule/RHS timing,
  runtime-path timing), plus snapshot/report assembly
  (`RumProfileReset`/`RumProfileSnapshot` support). A plausible domain split
  (verify cohesion against the real code and adjust with justification):
  `flags` (env gates + registry), per-domain counter modules, `timing`
  (builtin/rule/runtime-path timers), `snapshot` (assembly/report). THE RED
  LINE: the disabled-path cost model must be untouched — a single relaxed
  atomic load returning early; do not restructure anything that could change
  inlining of the disabled fast-outs (bodies verbatim; same-crate module
  boundaries do not block inlining).
- `catalog.rs` (74k): 345 `BuiltinSpec` entries in per-family `pub(crate)`
  consts (`ARITHMETIC_BUILTINS`, `LOGIC_BUILTINS`, …) consumed by
  `lib.rs::install_core`. Natural split: `catalog/` directory, one file per
  family const, `mod.rs` re-exporting all consts — `install_core` and the
  drift test keep compiling unchanged.
- Both files are COLD (profiling is env-gated off; catalog is startup
  registration data) — no microbench is required; SAY so in the report and
  assert the disabled-path red line explicitly.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/profiling.rs` → `profiling/`
  (directory); `crates/rum-stdlib/src/catalog.rs` → `catalog/` (directory);
  `runtime/mod.rs` / `stdlib/src/lib.rs` ONLY if module declaration syntax
  requires it (`mod profiling;` stays valid for a directory — likely zero
  change); `use` lines inside the split directories themselves.
- MUST NOT touch: any caller outside the two directories (their imports must
  keep resolving via the facades), any behavior, any test (the drift test and
  all profiling tests must pass UNMODIFIED — if a test needs editing, the
  split is wrong; STOP), `Cargo.toml`/`Cargo.lock`, any `mma-hol` clone,
  `docs/**` except your report, `.git`.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
ls crates/rum-runtime/src/runtime/profiling.rs crates/rum-stdlib/src/catalog.rs 2>&1   # both: No such file (now directories)
# per-file size audit of the new directories — paste the full table:
find crates/rum-runtime/src/runtime/profiling crates/rum-stdlib/src/catalog -name "*.rs" | while read f; do wc -c "$f"; done | sort -rn
git diff --stat | tail -3
```

Every new file ≤20k chars (justify any exception per principle 1). Paste all
outputs verbatim. Also paste evidence that NO file outside the two directories
changed except (at most) the two parent mod-declaration sites: the
`git status --short -- crates/` list.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Clean-check probes: `git status --short -- crates/`.
- Minimal diff beyond the moves themselves; paste real verbatim outputs; never
  fake a green.
- Stop-loss: if a cohesive split cannot keep external callers unchanged, or
  the same compile error repeats twice, STOP and report the boundary that
  resists — that is design information, not failure.

## Report format (your final message)

1. THE SPLIT MAPS (both files): module → responsibility → chars; DAG note
   (who depends on whom inside each directory); how the boundaries serve
   incremental compilation and the future feature-gate/move decision.
2. What changed — file inventory (created/deleted), the untouched-callers
   evidence.
3. Verification — each command with exact verbatim output incl. the size
   table.
4. Verdict — cold-path red-line assertion; any >20k exceptions with
   justification; advice for batch 2 (the HOT files evaluation_cache.rs /
   definition_application.rs).
5. Stop-loss / open questions (empty if none).
