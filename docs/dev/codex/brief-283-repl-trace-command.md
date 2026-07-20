# Brief 283 — `:trace` REPL command (design Option A)

Implements Option A from `docs/dev/repl-trace-view-design.md`, ruled by
the user on 2026-07-20. **Read that document first** — its §1 is
measured grounding, not description, and §2 Option A is the spec.

Everything below was verified on `962d6b6`.

## What to build

```
:trace EXPR                  whole trace, truncated
:trace --form=PAT EXPR       kernel form filter
:trace --depth=N EXPR        kernel levelspec
:trace --full EXPR           no truncation
```

Flags may combine. They precede the expression, matching the existing
`:format [--line-width N] EXPR` shape — follow that command's parsing
style rather than inventing a new one.

### Pipeline

1. Build rum source that calls the kernel's `Trace` (it is `{HoldAll,
   Protected}` and `Capability::Pure` — no capability gate needed, and
   it works in a sandboxed profile).
2. Evaluate it through the existing `session` handle that
   `execute_repl_command` already receives.
3. Walk the returned `HostTyped` and render an indented tree.
4. Truncate unless `--full`.

### Rendering

`Trace[expr]` returns a nested `List` of `Hold[...]`, where nesting is
subevaluation depth:

```
Trace[(1+2)*3] → List[Hold[Times[Plus[1, 2], 3]],
                      List[Hold[Plus[1, 2]], Hold[3]],
                      Hold[9]]
```

In the typed view this arrives as `HostTyped::List`, with each
`Hold[...]` leaf surfacing as `HostTyped::Other { head: "Hold", text }`
(`Hold` has no specialized typed view). So: recurse on `List`,
increasing indent one level per nesting; print each leaf's `text`.

**Render REPL-side from the structured value. Do NOT call
`TracePrint`** — the REPL owns its output stream (a hard line from the
host-embedding work), and rendering from the structured value keeps the
output under snapshot control.

Depth safety: `MAX_TYPED_DEPTH = 64` (`rum-host/src/model/outcome.rs:11`).
A trace nested deeper collapses into `Other` rather than `List`. Handle
that by printing that subtree's rendered `text` as a single leaf — do
not panic, do not silently drop it. If you can construct a trace that
reaches the cap, pin the behavior in a test; if you cannot reach it
with a reasonable expression, say so in the report.

Truncation prints a final line naming the count and the escape hatch,
e.g. `… N more steps (:trace --full to see all)`. Default limit: start
at **40 rendered lines** and pin it in a snapshot, so changing it later
is a deliberate act.

### The `--form` bare-symbol rewrite (the point of the flag)

Measured behavior of the kernel filter:

| form | result |
|---|---|
| `_Plus` | filters correctly |
| `_Integer`, literal `3`, `_` | filter correctly |
| **`Plus`** (bare symbol) | **`List[]`** — matches nothing |
| **`Plus[__]`** | **full trace** — matches everything |

The bare-symbol case is a trap: `--form=f` reads like "steps involving
`f`" and silently yields nothing. The `Plus[__]` case is NOT a `Trace`
bug — it is brief-275's `OneIdentity` promoted-singleton matching
working as designed (`MatchQ[9, Plus[__]]` → `True`;
`MatchQ[9, f[__]]` → `False`).

**Required behavior**: when `--form=` receives a bare symbol name
(an identifier with no pattern syntax), rewrite it to `_<name>` and
print one line saying so before the trace, e.g. `filtering on _f`.
Anything else passes through verbatim. **Do not change kernel
semantics** — this rewrite lives entirely in the REPL.

### `--depth=N`

Maps to `Trace`'s levelspec argument. `Trace[expr, form, levelspec]` is
the accepted shape, so a form is needed in that position — use `_` when
`--form` was not given. **Probe the exact levelspec spelling rum
accepts before wiring it** (`Trace[(1+2)*3, _Plus, All]` is known to
work); do not guess a spelling from WL habit.

### Errors and side effects

- A malformed expression must render through the existing failure path
  (`render_host_failure` / the same rendering the REPL already uses),
  not a bespoke message.
- `Trace[expr]` genuinely evaluates `expr`, so messages and prints can
  occur. Render them the way ordinary evaluation does — reuse the
  existing helpers, do not invent a second rendering.
- `:trace` with no expression prints usage, matching how `:parse` and
  `:lower` handle their empty forms.

## Mechanical ripple you MUST handle

Adding `:trace` to `help_text()` **will break the frozen `:help`
snapshot** in `crates/rum-repl/src/tests.rs`
(`repl_handles_help_and_quit`), which brief-282 pinned two commits ago.
That is expected and correct — it is the snapshot doing its job. Update
that expected text to the new actual output.

This is the ONLY pre-existing snapshot that should change. **If any
other frozen snapshot changes, STOP and report** — it would mean
`:trace` altered a surface it has no business touching.

## Tests

Add snapshot tests for `:trace` to the same file, in brief-282's style:
full-output `assert_eq!`, expected text **captured from real runs, not
hand-written**. Cover at minimum:

- plain `:trace` on a small expression;
- `--form=` with a pattern form (e.g. `_Plus`);
- `--form=` with a BARE symbol, pinning the rewrite notice line;
- `--depth=`;
- truncation (an expression long enough to trip the default limit) and
  the same expression with `--full`;
- `:trace` with no argument (usage);
- a malformed expression (failure rendering).

**Negative-calibrate three of them** exactly as brief-282 required:
perturb the rendering each pins, confirm the test fails, revert,
confirm green. Paste each. A snapshot that cannot fail is worthless.

## Scope

- Files you MAY modify:
  - `crates/rum-repl/src/commands.rs` (command variant, parsing,
    execution, `help_text()`);
  - `crates/rum-repl/src/lib.rs` — ONLY if dispatch genuinely requires
    it; prefer keeping everything in `commands.rs`;
  - `crates/rum-repl/src/tests.rs` (new tests + the one `:help`
    snapshot update).
- Files/areas you MUST NOT touch: `crates/rum-host/**` (no new host
  API — Option A deliberately needs none), `crates/rum-runtime/**` (no
  kernel semantic change, in particular none to `Trace`),
  `crates/rum-terminal/**`, `Cargo.toml` of any crate (no new
  dependencies), `docs/**`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`,
  `.git`, `Cargo.lock`.
- If you conclude the feature needs a `rum-host` or kernel change, STOP
  and report — that is a design question, not an implementation
  decision. (The design doc §1.5 records a second, host-level trace
  channel that would need such a change; it is explicitly OUT of scope
  for this brief.)
- No new `unsafe`.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-repl
cargo test --workspace
```

Baseline is `1654 passed / 0 failed` on `962d6b6`; state your totals
and the new count. Known sandbox caveat: loopback-bind ZeroMQ tests may
fail under the sandbox; if so run and paste the two supplemental splits
(`--workspace --exclude rum-jupyter`; `-p rum-jupyter --lib -- --skip
zeromq::tests::native`) and say so. Claude re-runs the authoritative
gate outside the sandbox.

Paste the actual rendered output of at least these, as your own
verification that the view is readable:

```
:trace 1+1
:trace (1+2)*3
:trace --form=Plus (1+2)*3      (exercises the bare-symbol rewrite)
:trace --form=_Plus (1+2)*3
```

No microbench: REPL-layer feature, no evaluation hot path touched.

## Stop-loss

- The typed view cannot be walked into a readable tree (e.g. the shape
  is not what §1.2/the Rendering section describes) → STOP, report what
  you actually observed.
- The feature appears to need a host or kernel change → STOP, report.
- More than the one `:help` snapshot changes → STOP, report which.

Revert everything and report. (`git checkout --` fails under the
sandbox with `Unable to create '.git/index.lock': Operation not
permitted` — restore by scoped patch instead and say so.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, file by file.
2. Verbatim verification output (fmt, `-p rum-repl` and workspace
   totals, the four rendered probes).
3. The three negative calibrations, each with its failure output.
4. Confirmation that exactly one pre-existing snapshot (`:help`)
   changed, and the levelspec spelling you probed and used.
5. Stop-loss / open questions.
