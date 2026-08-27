# brief-370 — stop degrading the argument handle at the builtin boundary

## Goal

Step 1 of plan-369. Change the builtin handler ABI to borrow `&SharedArgs`
instead of `&[Expr]`, and stop the ordinary evaluation path from paying for a
suspension capture it usually never uses. Language behavior and the suspension
representation are unchanged: this brief adds no frame type, deletes no
suspension API, and leaves the closure queue as the sole mechanism.

## The handle is already there — this is un-discarding it, not adding it

Both dispatch paths hold a `SharedArgs` and drop it on the floor one call
before the handler:

- `crates/rum-runtime/src/runtime/session/session_state.rs:90` —
  `raw_args: SharedArgs`.
- `:106` — `evaluated_shared_args(&self) -> &SharedArgs` already exists; `:102`
  `evaluated_args()` is nothing but `evaluated_shared_args().as_ref()`.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:82` passes
  `state.raw_args.as_ref()`, and `:116` passes `state.evaluated_args()`.
- `crates/rum-runtime/src/runtime/metadata.rs:289` —
  `pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> ...`.

`SharedArgs` is `Arc<Vec<Expr>>` (`crates/rum-core/src/expr.rs:638`) with
`AsRef<[Expr]>` at `:717`, so every helper that genuinely wants a slice keeps
working through `args.as_ref()`.

## The measured cost (plan-369 §2, reproduce it before you start)

A differential between an eager-capturing builtin (`Switch`) and a
lazy-capturing control (`CompoundExpression`) isolates one eager bulk copy:
**63,808 bytes** on a width-2003 call, against `1,998 x 32 = 63,936` predicted
by one extra `Vec<Expr>` payload. A width-2003 ordinary `Switch` pays about
63.8 KiB for a suspension capture that is never used.

**The applicable counter is the allocation counter, not `INTO_VEC_CLONE`** —
because the boundary has already degraded `SharedArgs` to a slice, the copy
bypasses `SharedArgs::into_vec` entirely and `into_vec_total` stays 0. This
corrected my own assumption in brief-369 and is worth carrying forward:
`INTO_VEC_CLONE` becomes meaningful *after* this brief, as the detector for
accidental COW/unwrap fallback.

## What to change

1. **`BuiltinFn` becomes `fn(&mut Session, &Expr, &SharedArgs) -> ...`**, and
   the two dispatch sites pass their handles instead of degrading them. There
   is to be **no dual ABI** — not a second handler type, not a shim that
   accepts both.

2. **The handlers follow** — roughly 578 `args: &[Expr]` signatures in
   `rum-stdlib` plus the runtime's own. This is compiler-forced and mechanical:
   where a body only reads a slice, `args.as_ref()` at the top is the whole
   migration.

3. **Read-only captures stop copying.** With the handle in hand,
   `let call_args = args.to_vec();` becomes an `Arc` bump. Convert every such
   capture.

4. **Eager capture sites become lazy.** `suspend_with_result_frame_lazy`
   already exists (`crates/rum-runtime/src/runtime/suspension/core.rs:162`)
   and builds the frame only in the `InputRequired` arm — but **only 9 sites
   use it while 307 still capture eagerly**. The mechanism for the whole 63.8
   KiB waste has been in the tree all along and was simply never propagated.

   **Convert all of them, not a sample.** Report the count you converted and
   the count remaining, and if any site genuinely cannot be made lazy, name it
   with the reason rather than leaving it silently.

## Scope

MAY modify: `crates/rum-runtime/src/**` and `crates/rum-stdlib/src/**` as the
global signature change forces, plus focused tests.

MUST NOT touch:

- `crates/rum-core/src/expr.rs` — `Expr`/`Symbol`/`SharedArgs` representation
  and any public serde shape. This brief consumes `SharedArgs`; it does not
  change it;
- the suspension queue representation, `SuspendedEvaluation`, or any frame
  type — plan-369 steps 371-374 own those;
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs`,
  `debug_tools/resources/memory.rs`, `crates/rum-derive/**`;
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.toml`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Never rebuild a `SharedArgs` from a slice.** `SharedArgs::new(x.to_vec())`
   reintroduces exactly the copy this brief removes. If a call site seems to
   need it, the handle was lost further up — fix it there, or STOP and report.
2. **Do not clone a handle unconditionally** to satisfy an owned downstream
   signature. An `Arc` bump is cheap but not free, and on the ordinary path it
   should not happen at all.
3. **Special-form and ordinary semantics differ** — special forms receive raw
   arguments, ordinary builtins receive evaluated ones. Preserve that
   distinction exactly; they are two different handles.
4. **No default-path instrumentation.** Counters stay behind the existing
   env-gated cold path.
5. **This changes no language behavior.** If a workspace test's result
   changes, something is wrong — investigate rather than adjusting the test.
6. **Stop-loss is real.** If the signature change cascades somewhere it should
   not, STOP and report with the tree as it stands.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test -p rum-stdlib --offline
cargo test -p rum-host --offline
cargo test --workspace --offline
```

Baseline is `8e97ba41`: **2150 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`.

**Counters first, timings second** — this project's standing rule is that if
the deterministic counters do not move, do not quote a time.

- Re-run plan-369 §2's four exact probes (`Switch` narrow/wide and the
  `CompoundExpression` control narrow/wide), each asserting the result `7`.
- **The 63.8 KiB width slope must be gone.** Paste the before/after
  differential table. This is the acceptance criterion for the whole brief.
- Report `shared_args_alloc` and `shared_expr_alloc`: they must not increase.
  Explain any nonzero `into_vec_clone` — after this change it is the detector
  for accidental COW fallback.
- Report the eager-to-lazy conversion count and what remains.

Then the full microbench A/B, profiling env OFF, three runs per arm, since
dispatch itself changes:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief370-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief370-new
```

Paste affected labels (`matcher-single-tiny`, `matcher-single-wide-small`,
`matcher-single-wide-large`, `replaceall-simple`, `replace-wide-no-match`,
`replacepart-wide-rule-list`, `count-wide-no-match`, `do-count-null`, the
callback sort family) and the named controls (`sort-default-wide`,
`sortby-default-wide`, `reversesort-default-wide`, `keysort-default-wide`,
`keysortby-default-wide`, `sort-symbol-default-width4000`,
`sort-symbol-default-width16000`), with the per-label spread across the three
runs. Never normalize against the suite median.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; name
  the prior same-shape duration each timeout is derived from. Persist raw
  microbench output before parsing.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the ABI, the handler migration count, the capture
   conversions, the eager-to-lazy count.
2. The differential table, before and after, with the width slope.
3. Counters — `shared_args_alloc`, `shared_expr_alloc`, `into_vec_clone`.
4. Correctness — commands, totals, `removed_or_changed=`.
5. The A/B — affected rows, controls, three-run spread, verdict.
6. Stop-loss / open questions (empty if none).
