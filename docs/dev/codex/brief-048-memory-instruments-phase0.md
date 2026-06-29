# brief-048: Memory line Phase 0 — instruments (`MaxMemoryUsed[]` + cross-engine memory comparator)

## Goal

Build the **yardstick** for the new memory-management line (`docs/memory-management-plan.md`,
Phase 0). Two deliverables, both instruments — NO optimization in this brief:

1. **`MaxMemoryUsed[]` builtin** (currently inert in rum; WK returns the session
   peak-memory high-water mark). Implement the 0-arg form as a sampled high-water
   mark over the existing structural memory proxy (`current_memory_proxy_bytes`).
2. **A cross-engine rum-vs-WK memory comparator script** (`scripts/core-memory-microbench.sh`)
   — the memory analog of `scripts/core-language-microbench.sh`. Same workload run
   in BOTH rum and WolframKernel, reporting per-workload {`MemoryInUse` delta,
   `MaxMemoryUsed`, peak-RSS delta} for each engine and the rum/WK ratio.

When done: the comparator REPRODUCES the plan's headline calibration —
`Do[Module[{x}, x=i^2; x], {i,1,20000}]` shows a rum `MemoryInUse` delta of
**~6 MB vs WK ~80 B**, and `memGlobal = Range[10000]` shows the ~8x dense-list
representation factor — so we have a measured baseline to gate every later memory
phase against.

This is an instrument brief: deliverable (1) is a code change with a `cargo test`
gate; deliverable (2) is a script whose EVIDENCE is the reproduced comparison table.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` hard line),
  `docs/memory-management-plan.md` (the whole plan; Phase 0 is "Recommended start"),
  and the `2026-06-29 (LATEST)` top block of `docs/next-session-handoff.md`.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. The
  `MemoryInUse[]` / `ByteCount[]` values are a **structural proxy** (a `size_of`
  walk of the `Session` maps), NOT a real allocator number — they are
  deterministic and reproducible, and they DO see the Module-gensym definition
  leak. They are NOT absolute-comparable to WK's real-kernel numbers; **only
  DELTAS are comparable across engines.** Peak RSS (captured externally by the
  script via `/usr/bin/time -l`) is the true total and the only thing that sees the
  symbol-interner `&'static` leak the proxy is blind to.
- Key files:
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` — the proxy
    (`current_memory_proxy_bytes:32`, `&self`) + the builtin impls
    (`eval_memory_in_use:18`, `eval_byte_count:4`). You add the peak bump here and
    a new `eval_max_memory_used`.
  - `crates/rum-runtime/src/runtime/model.rs` — `pub struct Session` (`:300`); the
    `print_sink` field (`:342`) is the precedent for a non-serde, non-snapshot field.
  - `crates/rum-runtime/src/runtime/session_state.rs` — the single Session
    construction site (`print_sink: None` at `:390`); init the new field here.
  - `crates/rum-runtime/src/runtime/metadata.rs` — the `BuiltinHandler` enum
    (`MemoryInUse,` at `:551`).
  - `crates/rum-runtime/src/runtime/eval_dispatch/surface.rs` — dispatch
    (`BuiltinHandler::MemoryInUse => self.eval_memory_in_use(head, args),` at `:115`).
  - `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs` — the `MemoryInUse`
    `BuiltinSpec` (`:101`).
  - `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs` — the empty-args
    syntax group (`"MemoryInUse" | "Stack" => ...` at `:186`).
  - `crates/rum-runtime/src/runtime/tests/debug_tools_resources.rs` — where the
    `MemoryInUse`/`ByteCount` tests live; add the `MaxMemoryUsed` tests here.
  - `scripts/core-language-microbench.sh` — the TIMING comparator to mirror in
    STRUCTURE (arg parsing, per-engine run loop, TSV outputs, `RUM_CORE_MICRO`
    tagged-line parse). You are writing the memory analog.

## Part A — `MaxMemoryUsed[]` builtin (rum-runtime)

### A1. `Session` field (interior-mutable high-water mark)

`model.rs`, `pub struct Session` (`:300`): add a field

```rust
pub(super) peak_memory_proxy_bytes: std::cell::Cell<usize>,
```

`Cell<usize>` (NOT `usize`) so the peak can be bumped through the `&self` receiver
of `current_memory_proxy_bytes` without rippling `&mut self` through its callers.
`Session` is ALREADY `!Sync` (the `print_sink: Box<dyn FnMut>` field), and `Cell`
is `Send`, so this does not change `Session`'s auto-traits. It is NOT serializable
state, NOT part of any snapshot, and NOT part of `LoaderSnapshot`
(`session_state.rs:131`) or any rollback — leave it monotonic (a session-wide
diagnostic high-water mark, exactly like WK's `MaxMemoryUsed[]`, which Abort does
not reset). Do NOT add it to any snapshot/restore path.

`session_state.rs`, the construction site next to `print_sink: None` (`:390`):
```rust
peak_memory_proxy_bytes: std::cell::Cell::new(0),
```

### A2. Bump the high-water mark inside the proxy (`memory.rs`)

In `current_memory_proxy_bytes(&self) -> usize` (`memory.rs:32`), just before
`total` is returned (after line 234's `total`), record the high-water mark and
return:

```rust
        let peak = self.peak_memory_proxy_bytes.get().max(total);
        self.peak_memory_proxy_bytes.set(peak);
        total
```

This makes EVERY proxy computation a sample point: `MemoryInUse[]`
(`eval_memory_in_use`), `MaxMemoryUsed[]` (below), and the `MemoryConstrained`
checks (`constraints.rs:134,159`) all bump the peak. This adds NO default-path
cost — `current_memory_proxy_bytes` is never on the hot eval path; it runs only
when one of those explicitly-invoked builtins computes it.

### A3. The builtin impl (`memory.rs`, same `impl Session` block as `eval_memory_in_use`)

```rust
    pub(in crate::runtime) fn eval_max_memory_used(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        // Sample the current proxy (this bumps the high-water mark), then return
        // the peak seen across all sample points so far this session.
        let _ = self.current_memory_proxy_bytes();
        Ok(Expr::integer(BigInt::from(
            self.peak_memory_proxy_bytes.get() as u64,
        )))
    }
```

Mirror `eval_memory_in_use` exactly (same arg-count guard returning the call
unevaluated, same `Expr::integer(BigInt::from(... as u64))` shape). The 0-arg form
is the only form Phase 0 needs; the 1-arg `MaxMemoryUsed[expr]` form (peak
additional bytes during an evaluation) requires intra-eval sampling and is OUT OF
SCOPE — returning the call unevaluated for any args is the correct Phase-0
behavior.

### A4. Register the builtin (3 edits)

- `metadata.rs`, `BuiltinHandler` enum: add `MaxMemoryUsed,` immediately after
  `MemoryInUse,` (`:551`).
- `eval_dispatch/surface.rs`: add
  `BuiltinHandler::MaxMemoryUsed => self.eval_max_memory_used(head, args),`
  immediately after the `MemoryInUse` arm (`:115`).
- `builtin_catalog/debug.rs`: add, immediately after the `MemoryInUse` spec (`:101`):
  ```rust
      BuiltinSpec::ordinary(
          "MaxMemoryUsed",
          "Returns the peak session memory-proxy byte count high-water mark subset.",
          BuiltinHandler::MaxMemoryUsed,
      ),
  ```
  (`ordinary`, NOT `ordinary_with_hold` — like `MemoryInUse`.)
- `namespace_tools/syntax.rs:186`: change
  `"MemoryInUse" | "Stack" => vec![syntax_argument_pattern(Vec::new())],`
  to `"MemoryInUse" | "MaxMemoryUsed" | "Stack" => vec![syntax_argument_pattern(Vec::new())],`.

### A5. Tests (`tests/debug_tools_resources.rs`)

Add a test (mirror the existing `MemoryInUse` tests in this file for the
`eval_text` + `IntegerQ`/value-extract style):

- `MaxMemoryUsed[]` returns a positive `Integer` (`IntegerQ[MaxMemoryUsed[]]` is
  `True`; value > 0).
- `MaxMemoryUsed[] >= MemoryInUse[]` evaluated at the same point (peak is never
  below current) — assert via `MaxMemoryUsed[] >= MemoryInUse[]` is `True`.
- Non-decreasing across growth: capture `m1 = MaxMemoryUsed[]`; evaluate a
  definition-growing workload (e.g. `Do[gmem[i] = i, {i, 1, 300}]`); capture
  `m2 = MaxMemoryUsed[]`; assert `m2 >= m1` is `True` AND `m2 > m1` (300 ownvalues
  grow the proxy — confirm the high-water mark actually moved).
- `MaxMemoryUsed[x]` (any arg) returns unevaluated `MaxMemoryUsed[x]` (the
  arg-count guard).

## Part B — cross-engine memory comparator (`scripts/core-memory-microbench.sh`)

A NEW executable script mirroring `scripts/core-language-microbench.sh` in
STRUCTURE: `#!/usr/bin/env bash` + `set -euo pipefail`, the same `--output-dir` /
`--engine {rum|wolfram|both}` / `--rum-cli` / `--wolfram` option parsing, a
per-engine run loop, and TSV outputs under the output dir. Differences from the
timing script (read carefully):

### B1. Per-workload SEPARATE process, wrapped in `/usr/bin/time -l` for peak RSS

Unlike the timing script (one `.wls` running all labels in one process), the
memory comparator runs **each workload in its own fresh process** so peak RSS is
attributable per workload. For each (engine, workload) run the engine DIRECTLY
under `/usr/bin/time -l` (the engine must be the direct child so its peak RSS is
what `time -l` reports — do NOT interpose `run-bounded.py`/`timeout`, which would
report the wrapper's RSS instead). Workloads are small and bounded, so no external
timeout is needed.

- rum: `/usr/bin/time -l "${rum_cli_bin}" --eval "${program}" >stdout 2>stderr`
- WK: `/usr/bin/time -l "${wolfram_bin}" -code "${program}" >stdout 2>stderr`
  - default `wolfram_bin` to `wolframscript` at `/usr/local/bin/wolframscript`
    (per `docs/memory-management-plan.md`), overridable with `--wolfram`. (Both
    `MemoryInUse[]` and `MaxMemoryUsed[]` exist in WK.)

`time -l` writes its report (including `maximum resident set size`) to stderr;
parse the integer on the `maximum resident set size` line from the stderr file
(treat it as bytes — macOS reports bytes). If the line is absent or unparseable,
leave the RSS column blank rather than failing the script.

### B2. The per-workload program (identical WL for both engines)

For a workload with `<label>` and held statement `<expr>`, the program is exactly:

```
b0 = MemoryInUse[]; <expr>; b1 = MemoryInUse[]; mx = MaxMemoryUsed[]; Print["RUM_MEM\t<label>\t", b0, "\t", b1, "\t", mx]
```

`b1 - b0` is the `MemoryInUse` delta (the small `b0`/`b1`/`mx` ownvalue defs are a
constant in both engines, negligible vs the deltas of interest — note this in a
comment). Parse only lines whose first tab-field is `RUM_MEM` (mirror the
timing script's `RUM_CORE_MICRO` awk parse); ignore the engine's echoed final
value and any other output.

### B3. Workload set (embed these exactly)

| label | `<expr>` | purpose |
| --- | --- | --- |
| `baseline` | `Null` | per-engine startup RSS floor (subtracted to get RSS deltas) |
| `module-leak-20000` | `Do[Module[{x}, x = i^2; x], {i, 1, 20000}]` | the headline gensym leak (expect rum proxy-delta ~6 MB, WK ~80 B) |
| `retain-range-1000` | `memGlobal = Range[1000]` | retained dense list, ~8x representation factor |
| `retain-range-10000` | `memGlobal = Range[10000]` | retained dense list, larger scale |

(`Range[n]` alone is discarded and shows ~0 delta — it MUST be retained via the
`memGlobal =` assignment so the proxy/RSS reflect the live list.)

### B4. Outputs (under `--output-dir`, mirror the timing script's TSV style)

- `raw/<engine>.<label>.stdout` / `.stderr` per run.
- `measurements.tsv`: one row per (engine, label) —
  `engine  label  mem_before  mem_after  mem_delta  max_memory  peak_rss_bytes`.
- `comparison.tsv`: one row per label —
  `label  rum_mem_delta  wk_mem_delta  mem_delta_ratio  rum_max_memory  wk_max_memory  max_memory_ratio  rum_rss_delta  wk_rss_delta  rss_delta_ratio`
  where `*_rss_delta` = that label's `peak_rss_bytes` minus the same engine's
  `baseline` `peak_rss_bytes` (so RSS is a workload-attributable delta), and each
  `*_ratio` = rum / wk (blank when the WK denominator is 0 or missing). `baseline`
  itself has rss_delta 0 by construction.
- Echo the output dir path at the end (like the timing script).

Keep the WL/shell honest: the workload statement in the program is the table's
`<expr>` verbatim between the two `MemoryInUse[]` calls — nothing else added.

## Scope

- Files you MAY modify / create:
  - `crates/rum-runtime/src/runtime/model.rs` (the `Session` field),
  - `crates/rum-runtime/src/runtime/session_state.rs` (init the field),
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` (peak bump +
    `eval_max_memory_used`),
  - `crates/rum-runtime/src/runtime/metadata.rs` (enum variant),
  - `crates/rum-runtime/src/runtime/eval_dispatch/surface.rs` (dispatch arm),
  - `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs` (`BuiltinSpec`),
  - `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs` (the empty-args group),
  - `crates/rum-runtime/src/runtime/tests/debug_tools_resources.rs` (tests),
  - `scripts/core-memory-microbench.sh` (NEW, `chmod +x`).
- MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` acceptance standard — never
    edit it to pass (see `AGENTS.md`); no diagnostic edits there;
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` and any public type/serde shape;
  - the hot eval path / `eval_*`/`resume_*` evaluator flows (the only runtime change
    is the new builtin + the proxy high-water bump, both off the default path);
  - `LoaderSnapshot` / any snapshot/rollback path (the peak field is monotonic
    diagnostic state, not snapshot state);
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (incl. `docs/dev/codex/**`)
    except appending to your own report;
  - `.git`, git config, `Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path. The peak bump lives
  ONLY inside `current_memory_proxy_bytes`, which runs only when
  `MemoryInUse`/`MaxMemoryUsed`/`MemoryConstrained` is explicitly evaluated — never
  per eval step. No new per-call atomic/Mutex/allocation/branch on the hot path.

## rum / Rust pitfalls (read twice)

1. **`Cell`, not `usize`.** The proxy is `&self`; a plain `usize` field would force
   `&mut self` and ripple through `eval_memory_in_use` and the `constraints.rs`
   callers. `Cell<usize>` keeps `&self`. If a `Sync` bound surfaces at compile (it
   should not — `Session` is already `!Sync` via `print_sink`), STOP and report.
2. **Deltas only, never absolute.** The comparator's headline is the rum/WK ratio of
   `MemoryInUse` DELTAS (and RSS deltas vs the `baseline` floor). Do NOT compare
   absolute `MemoryInUse[]` across engines — rum's proxy (~37 KB at startup) and
   WK's real number (~62 MB) are different scales by construction.
3. **Retain to measure representation.** `Range[1000]` discarded shows ~0 delta;
   the workload assigns `memGlobal = Range[...]` so the list stays live. Keep the
   assignment.
4. **`time -l` wraps the ENGINE directly.** Peak RSS is the direct child's RSS. Do
   not interpose a runner/timeout between `time -l` and the engine, or RSS is the
   wrapper's. Workloads are bounded; no timeout needed.
5. **A new `BuiltinHandler` variant must compile everywhere.** Adding the enum
   variant can break any exhaustive `match` over `BuiltinHandler` elsewhere;
   `cargo build`/`cargo test` across the workspace catches it. Add the dispatch arm
   and the catalog spec so the variant is wired, not dead.
6. **Minimal diff.** One field + one init + the proxy bump + one builtin fn + the 3
   registration edits + the syntax line + the tests + the new script. No
   reformatting, no drive-by renames.
7. **Stop-loss.** If `cargo test` fails twice for the same reason, or the comparator
   does NOT reproduce the ~6 MB-vs-~80 B headline (e.g. rum shows ~0 or WK shows
   megabytes), STOP and report the numbers — do not tune the workload to fit.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo build --release -p rum-cli

# Comparator — reproduce the headline calibration:
chmod +x scripts/core-memory-microbench.sh
bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_brief_048_mem
echo "=== comparison.tsv ==="; cat /tmp/cdx_brief_048_mem/comparison.tsv
echo "=== measurements.tsv ==="; cat /tmp/cdx_brief_048_mem/measurements.tsv

# Sanity: MaxMemoryUsed[] is live and >= MemoryInUse[]:
./target/release/rum-cli --eval 'b=MemoryInUse[]; Do[g[i]=i,{i,1,500}]; {b, MemoryInUse[], MaxMemoryUsed[], MaxMemoryUsed[] >= MemoryInUse[]}'
```

Paste every line verbatim: the `cargo fmt --check` result, the
`cargo test --workspace` summary line (e.g. `test result: ok. N passed; 0 failed`),
the full `comparison.tsv` and `measurements.tsv`, and the rum-cli sanity line. If
any run does not reach green / does not produce the table, say so explicitly —
never fake it.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in the
  working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write that in
  the report instead.
- Full machine access, but touch NOTHING outside this repo, no network. Commands:
  file edits in scope, `cargo`, the release `rum-cli`, `wolframscript` (for the
  comparator's WK side), `/usr/bin/time`, and `bash`/standard shell for the script.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- Stop-loss: same failure twice, or the headline does not reproduce → STOP and report.
- Minimal diff; no drive-by reformatting/renames/comment churn outside the task.
- You MUST paste the real verbatim verification output. If a run did not reach green
  / did not produce the evidence, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why (the `Session`
   field, the init, the proxy bump, `eval_max_memory_used`, the 3 registration
   edits, the syntax line, the tests, the new script).
2. Verification — each command run, with the exact verbatim output (`cargo fmt`,
   the `cargo test --workspace` result line, `comparison.tsv`, `measurements.tsv`,
   the rum-cli sanity line).
3. The finding / verdict — does the comparator reproduce the headline
   (`module-leak-20000`: rum `MemoryInUse` delta ~6 MB vs WK ~80 B;
   `retain-range-*`: the rum/WK ratio and whether it lands near ~8x)? State the
   measured numbers.
4. Stop-loss / open questions — where you stopped and why, anything unsure (empty
   if none).
