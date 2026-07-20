# Brief 284 — peel the `Hold` wrapper from `:trace` leaves

Small follow-up to brief-283. **The cause is a gap in my brief, not in
the execution**: brief-283 said "print each leaf's `text`", and that is
exactly what was built — but the leaf's `text` includes the `Hold[…]`
wrapper that `Trace` puts around every recorded step.

Today:

```
rum> :trace 1+1
Hold[Plus[1, 1]]
Hold[2]
```

Wanted:

```
rum> :trace 1+1
Plus[1, 1]
2
```

The wrapper exists only to keep the recorded step unevaluated. It
carries no information for a reader and appears on 100% of lines, which
is a lot of noise in a debugging view.

Two precedents say peeling is the house style:

- the kernel's own `TracePrint` renders `Plus[1, 2]`, not
  `Hold[Plus[1, 2]]`;
- `HostValue.text` is documented as "canonically rendered expression
  text **with one top-level `HoldForm` peeled**" — the project already
  treats a display-only hold wrapper as something a rendering boundary
  removes.

## What to change

In `crates/rum-repl/src/commands.rs`, where a trace leaf is rendered
(currently `HostTyped::Other { text, .. } => text.clone()` in the
trace-rendering path, around line 429):

When the leaf is `HostTyped::Other { head, text }` **and `head` is
`Hold`**, render the wrapper's contents instead of the whole text.
Peel exactly ONE level.

Constraints:

- Peel only for `head == "Hold"`. Any other head renders unchanged.
- Peel only in the `:trace` rendering path. `render_typed_text` may be
  shared with other commands — **check its call sites before editing
  it**; if it is shared, do the peeling in the trace renderer rather
  than changing shared behavior, and say which you did.
- If the text does not have the expected `Hold[` … `]` shape, fall back
  to rendering it unchanged rather than slicing blindly.
- Nested `Hold[Hold[x]]` (if it can even occur) peels only once.

## Tests

Every `:trace` snapshot added by brief-283 will change. Update them all
from **actual captured output**, not by hand-editing the strings — the
whole point of brief-282's discipline is that expected text is measured.

The `:help` snapshot must NOT change. Nor must any non-`:trace`
snapshot. If one does, STOP and report.

Negative-calibrate once: perturb the peeling (e.g. peel two levels, or
peel for the wrong head) and confirm a `:trace` snapshot fails, then
revert and confirm green. Paste it.

## Scope

- Files you MAY modify: `crates/rum-repl/src/commands.rs`,
  `crates/rum-repl/src/tests.rs`.
- MUST NOT touch: `crates/rum-host/**`, `crates/rum-runtime/**`,
  `crates/rum-terminal/**`, any `Cargo.toml`, `docs/**`, `TODO.md`,
  `PROGRESS.md`, `AGENTS.md`, `.git`, `Cargo.lock`.
- No kernel change. In particular do NOT change what `Trace` returns —
  this is purely a REPL rendering fix.
- No new dependencies, no new `unsafe`.

## Verification

```
cargo fmt --check
cargo test -p rum-repl
cargo test --workspace
```

Baseline is `1662 passed / 0 failed` on `c031f89`; the count must not
change (no tests added or removed). Sandbox ZeroMQ caveat applies as
usual; Claude re-runs the authoritative gate outside the sandbox.

Paste the rendered output of:

```
:trace 1+1
:trace (1+2)*3
:trace --form=Plus (1+2)*3
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, and whether you peeled in the trace renderer or in
   a shared helper (with the call-site check that decided it).
2. Verbatim verification output, including the three rendered probes.
3. The negative calibration.
4. Stop-loss / open questions.
