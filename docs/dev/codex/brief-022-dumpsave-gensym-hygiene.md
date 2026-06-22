# brief-022: DumpSave context-dump must exclude Module-generated gensyms (snapshot hygiene, #2)

## Goal

When `DumpSave` (and `Save`) serialize a CONTEXT (e.g.
`DumpSave[file, "HOL`Parser`"]` or `DumpSave[file, Evaluate[listOfContextStrings]]`),
the enumerated symbol set must EXCLUDE Module-generated temporary gensyms
(`base$NNN` — a `$` that is not the first character and is immediately followed
by a digit, e.g. `a$1`, `baseConv$17`, `xEq$45`). rum never garbage-collects
Module locals, so they accumulate in their context forever and are dumped: even
the Parser-layer snapshot has 13277 such gensyms (98% of names), and the
`HOL`Auto`Simp`Private`` context inflates a through-`Pair` dump to ~402MB. WL
does not save these (they are Temporary). When done, a context dump skips
gensyms by default, keeps intentional `$`-PREFIXED globals (`$simpDepthLimit`,
`$rumBaseConvCache`), the through-`Pair` Stable snapshot is small (the prior
WL-scratch experiment got ~500KB) and restores correctly (`pair_tests.wl`
green), with zero added cost on the evaluation path.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` HARD LINE) and
  the PROGRESS entry `2026-06-22 Snapshot reliability test (dimension A)` (#2 is
  diagnosed there) plus the older `402MB` analysis (search PROGRESS for `402MB`:
  the WL-scratch experiment removed `HOL`*$*` before dumping and got a 500KB
  through-`Pair` snapshot with `pair_tests` `42/0` after restore — that is the
  soundness evidence and the size target).
- Key files:
  - `crates/rum-runtime/src/runtime/package_tools.rs` — `DumpSave`/`Save` live
    here. `symbols_in_context` (around line 284) enumerates a context's symbols;
    `collect_save_symbols` routes a context-string / list-of-contexts to it.
    **The filter belongs in the context-enumeration path** (`symbols_in_context`),
    so an EXPLICIT `DumpSave[file, someGensym]` is still honored — only context
    enumeration skips gensyms (WL-faithful).
  - `crates/rum-core/src/...` `Symbol` — use `symbol.name()` (the short name,
    context stripped) for the pattern test. Do NOT change the `Symbol`/`Expr`
    type or serde shape.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. Acceptance
  is mechanical: the named outputs and tests decide.

## The fix (precise)

In `symbols_in_context` (the context-enumeration path only), drop any symbol
whose SHORT name is a Module gensym. Gensym test: the short name contains an
index `i > 0` such that `name[i] == '$'` and `name[i+1]` is an ASCII digit
(`0`..`9`). This matches `a$1`, `baseConv$17`, `x$2$3`; it does NOT match a
leading-`$` intentional global (`$simpDepthLimit`, `$rumBaseConvCache` — `$` at
index 0) nor a `$` not followed by a digit. Put the predicate in a small private
`fn is_module_gensym_name(name: &str) -> bool` and filter the enumeration.

This is a DUMP-TIME change only: it must not touch Module evaluation or add any
per-evaluation cost. Do NOT try to fix the live gensym leak (Module locals never
being GC'd) here — that is a separate, larger evaluator change and is explicitly
out of scope for this brief.

Soundness note (state it in your report): this assumes no RETAINED saved
definition depends on a gensym's value. The WL-scratch experiment (PROGRESS,
`pair_tests 42/0` after a gensym-stripped redump) is the evidence that this
holds for the mma-hol Stable world; your `Pair` verification below re-confirms
it. If `pair_tests` regresses after restore, STOP and report — the exclusion is
not sound as-is.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/package_tools.rs` (the filter + predicate).
  - ONE test file under `crates/rum-runtime/src/runtime/tests/` for the
    regression test (synthetic Module-leak dump).
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` standard — for the
    `Pair` verification you MUST copy the clone to a throwaway scratch dir and
    build there (the build writes `bootstrap.mx` into the root); NEVER write to
    `/Users/fft/Developments/mma-hol-acceptance`;
  - the Module/gensym creation path or any `eval_*` hot path (zero eval-path
    cost — this is dump-time only);
  - `crates/rum-core` `Symbol`/`Expr`/serde shape;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your
    own report, `.git`, git config, `Cargo.lock`.

## Deliverable

- The `is_module_gensym_name` predicate + the `symbols_in_context` filter.
- A synthetic regression test (Module leak → context dump excludes the gensyms,
  keeps a leading-`$` global).
- In your report: the synthetic before/after, the Parser-layer numbers, and the
  through-`Pair` size + `pair_tests`-after-restore result.

## rum / Rust pitfalls (read twice)

1. **Only the context-enumeration path.** Filter in `symbols_in_context`, not in
   the explicit-symbol or `Names[...]`-pattern branches. `DumpSave[f, a$1]`
   (explicit) must still save `a$1`.
2. **Keep leading-`$` globals.** `$simpDepthLimit` MUST survive (the `$` is at
   index 0 → not a gensym). Test this explicitly.
3. **Zero eval-path cost.** No change to Module evaluation, gensym creation, or
   any per-call path. Dump-time only.
4. **Use the SHORT name.** Test `symbol.name()` (context already stripped), not
   the full `ctx`name`.
5. **Minimal diff**; no drive-by reformatting.
6. **Stop-loss:** if `pair_tests` regresses after a gensym-excluded restore, or
   the through-`Pair` dump is still huge, STOP and report — do not widen the
   pattern to chase it.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
```

Synthetic (fast):
```
BIN=./target/release/rum-cli
rm -f /tmp/g.mx
$BIN --eval 'Sandbox`f[n_] := Module[{a, b}, a = n; b = a + 1; a + b]; Sandbox`f[5]; Sandbox`f[6]; Sandbox`$keep = 42; DumpSave["/tmp/g.mx", "Sandbox`"]; "ok"'
grep -c '\$[0-9]' /tmp/g.mx          # expect 0 — no gensyms dumped
grep -c 'Sandbox`\$keep' /tmp/g.mx   # expect 1 — leading-$ global kept
```

Parser layer (fast — use a SCRATCH copy, never the pristine clone):
```
SCRATCH=/tmp/cdx022-mma-hol
rm -rf "$SCRATCH"; rsync -a --exclude='.git' --exclude='*.mx' /Users/fft/Developments/mma-hol-acceptance/ "$SCRATCH"/
$BIN --eval '$ScriptCommandLine = {"tests/build_snapshot.wls", "Parser"}; Get["'"$SCRATCH"'/tests/build_snapshot.wls"];'  2>&1 | grep -iE 'KB|contexts'
ls -l "$SCRATCH"/bootstrap.mx        # expect MUCH smaller than the ~11.9MB pre-fix Parser snapshot
$BIN --eval '$ScriptCommandLine = {"tests/run_fast.wls", "parser"}; Get["'"$SCRATCH"'/tests/run_fast.wls"];' 2>&1 | grep -iE 'passed|failed'   # expect passed: 78, failed: 0
```

Through-`Pair` (the prize — Stable mode load is ~85s; allow a few minutes):
```
$BIN --eval '$ScriptCommandLine = {"tests/build_snapshot.wls", "Pair"}; Get["'"$SCRATCH"'/tests/build_snapshot.wls"];' 2>&1 | grep -iE 'KB|contexts|loaded'
ls -l "$SCRATCH"/bootstrap.mx        # expect << 402MB (target order ~hundreds of KB to a few MB), NOT hundreds of MB
$BIN --eval '$ScriptCommandLine = {"tests/run_fast.wls", "pair"}; Get["'"$SCRATCH"'/tests/run_fast.wls"];' 2>&1 | grep -iE 'passed|failed'   # expect failed: 0
```

Loop edit -> `cargo test` -> fix until green; then run the three verification
blocks and paste their verbatim output. The through-`Pair` size + `pair_tests`
failed:0 is the acceptance.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- NEVER write to or edit `/Users/fft/Developments/mma-hol-acceptance` — use a
  throwaway scratch copy for all snapshot builds.
- Full machine access, but touch NOTHING outside this repo (except the named
  `/tmp` scratch), no network. Commands: file edits within Scope, `cargo`, the
  release `rum-cli`, `rsync`/`ls`/`grep` for the scratch verification,
  `python3`.
- Minimal diff; no drive-by churn.
- Stop-loss: `pair_tests` regression after restore, or no size drop -> STOP and
  report.
- You MUST paste real verbatim output; if something is not green, say so — never
  fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — each command, verbatim output (test counts, the synthetic
   grep counts, Parser/Pair sizes, the `pair_tests`/`parser` pass lines).
3. The verdict — through-`Pair` dump size before/after (vs ~402MB) and
   `pair_tests` failed:0 after restore; confirm leading-`$` globals survived.
4. Stop-loss / open questions — anything unsure (esp. any soundness concern).
