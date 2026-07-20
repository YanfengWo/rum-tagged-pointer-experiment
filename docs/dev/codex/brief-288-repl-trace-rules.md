# Brief 288 — `:trace --rules` (surface `TraceDetail` in the REPL)

`TraceDetail` landed in brief-287 (`1c7776a`): it keeps the four fields
`Trace` discards, so it can answer "which of my overlapping definitions
actually fired". This brief gives it a REPL surface.

Design context: `docs/dev/trace-detail-design.md` and
`docs/dev/repl-trace-view-design.md`. Grounding below verified on
`1c7776a`.

## What to build

A `--rules` flag on the existing `:trace` command:

```
:trace --rules EXPR
```

It composes with every existing flag (`--form=`, `--depth=`, `--full`),
which is free: `TraceDetail` mirrors `Trace`'s whole argument surface.

The only pipeline change is which symbol gets called —
`trace_source` (`commands.rs:354`) builds `Trace[…]` today; with
`--rules` it builds `TraceDetail[…]` with the same argument
positions. The bare-symbol `--form=f` → `_f` rewrite and its notice
line, the 40-line truncation, and `--full` all keep working unchanged.

## Rendering — the real work

`Trace` yields bare `Hold[…]` leaves, so today one leaf is one line.
`TraceDetail` yields an `Association` per step, which arrives as
`HostTyped::Association(Vec<(HostTyped, HostTyped)>)` with `Hold[…]`
values surfacing as `Other { head: "Hold", text }`.

Rendering the raw association would be a wall of text — exactly what the
design doc warned about. `render_trace_leaf` needs an `Association` arm
that produces ONE compact line per step.

Measured input (this is real output, captured before writing this brief):

```
f[x_] := general; f[x_Integer] := integer; TraceDetail[f[2] + 1]

List[List[Assoc{Step:RuleApplied, Source:DownValue, Input:Hold[f[2]],
                RuleLeft:Hold[f[Pattern[x, Blank[Integer]]]],
                Result:Hold[integer]},
          Assoc{Step:EvaluationPass, Before:Hold[f[2]],
                After:Hold[integer]}],
     Assoc{Step:EvaluationPass, Before:Hold[Plus[f[2], 1]],
           After:Hold[Plus[1, integer]]}]
```

Target rendering (recommended shape — settle exact spacing yourself,
then pin it):

```
rum> :trace --rules f[2] + 1
  f[2] → integer   [DownValue f[x_Integer]]
  f[2] → integer
f[2] + 1 → 1 + integer
```

Per step kind, the line must carry:

| `Step` | line content |
|---|---|
| `RuleApplied` | `Input → Result`, then the rule identity: `Source` and `RuleLeft` |
| `EvaluationPass` | `Before → After` |
| `DefinitionStored` | `Symbol = Value` plus `Kind` |

Rules:
- Indentation continues to come from `List` nesting depth, exactly as
  today — do not derive it from the association.
- `Hold[…]` wrappers are peeled from every expression-valued field, the
  same one-level peel `render_trace_leaf` already does (brief-284).
  **Peel exactly one level** — the inner `Hold` in a user's own
  `Hold[…]` must survive.
- If an association lacks an expected key, or `Step` is a value you do
  not recognize, render what you can and do not panic — a future event
  kind must degrade, not crash. Say in your report how you handled it.

## Mechanical ripple

Adding `--rules` to `help_text()` **will break the frozen `:help`
snapshot**, as brief-283 did. Expected; update it. **If any other
pre-existing snapshot changes, STOP and report** — `--rules` must not
alter plain `:trace` output at all.

## Tests

New snapshots in `crates/rum-repl/src/tests.rs`, brief-282 style
(full-output `assert_eq!`, expected text **captured from real runs, not
hand-written**):

- `:trace --rules` on a small expression;
- **the headline case**: several overlapping `DownValue`s, asserting the
  rendered line names the definition that actually fired. This is why
  the flag exists;
- a `DefinitionStored` step (e.g. `:trace --rules g = 5`);
- `--rules` combined with `--form=` (including the bare-symbol rewrite)
  and with `--depth=`;
- truncation with `--rules`, and the same with `--full`;
- a regression pinning that plain `:trace` (no `--rules`) is byte-for-byte
  unchanged.

**Negative-calibrate two of them**: perturb the rendering each pins
(e.g. drop the `Source` from the rule line; change the arrow), confirm
the test fails, revert, confirm green. Paste both.

## Scope

- MAY modify: `crates/rum-repl/src/commands.rs`,
  `crates/rum-repl/src/tests.rs`.
- MUST NOT touch: `crates/rum-repl/src/lib.rs` unless dispatch genuinely
  forces it (say so if it does); `crates/rum-runtime/**` (no kernel
  change — `TraceDetail` is finished); `crates/rum-host/**`,
  `crates/rum-terminal/**`, `crates/rum-jupyter/**`; any `Cargo.toml`
  (no new dependencies); `docs/**`; `TODO.md`; `PROGRESS.md`;
  `AGENTS.md`; `.git`; `Cargo.lock`.
- No new `unsafe`.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-repl
cargo test --workspace
```

Baseline is `1668 passed / 0 failed` on `1c7776a`; state your totals.
Sandbox ZeroMQ caveat applies; Claude re-runs the authoritative gate
outside the sandbox.

Paste the rendered output of at least these, as your own check that the
view reads well:

```
:trace --rules 1+1
:trace --rules f[2] + 1          (with f[x_] and f[x_Integer] defined)
:trace --rules g = 5
:trace --rules --form=Plus (1+2)*3
:trace (1+2)*3                    (regression: plain :trace unchanged)
```

No microbench: REPL-layer only.

## Stop-loss

- A pre-existing snapshot other than `:help` changes → STOP, report.
- The association cannot be rendered to one readable line per step
  without a kernel change → STOP, report (do NOT change `TraceDetail`).
- The work needs a MUST-NOT file → STOP, report.

(`git checkout --` fails under the sandbox with `Unable to create
'.git/index.lock': Operation not permitted` — restore by scoped patch
instead and say so.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, file by file.
2. Verbatim verification output, incl. the five rendered probes.
3. The two negative calibrations.
4. How you handled a missing/unknown key, and confirmation that exactly
   one pre-existing snapshot (`:help`) changed.
5. Stop-loss / open questions.
