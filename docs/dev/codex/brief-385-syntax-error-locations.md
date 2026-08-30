# brief-385 — syntax error locations are computed per form and rendered per file (revision 1)

**Revision 1.** The first dispatch stopped correctly: it found a pinned
assertion the brief had missed and, as instructed, reported it instead of
updating it. The director has now ruled on that pin — see **The pinned
assertion, ruled** — and authorizes exactly that one change. Every other
pinned location that moves is still a stop.


`TODO.md` has carried this since the second sweep, deliberately excluded from
the splitter work because fixing it changes user-visible output and would have
voided that change's byte-identical acceptance criterion. That reason has
expired.

## The defect

Director's re-probe on the current tree, release CLI:

```
$ cat script.rum
a = 1
b = 2
c = 3
d = 1 ! 2          <- the offending ! is at 4:7

$ rum-cli --file script.rum
error: trailing input at bytes 6..7
 --> 2:1
```

Both the line and the column are wrong. `6..7` is the offset of `!` **within
its own form** (`d = 1 ! 2`), and byte 6 of the whole file happens to be line
2, column 1 — so the reported location is that of an unrelated character.

`render_syntax_error` (`crates/rum-syntax/src/document.rs:88-100`) resolves the
`SyntaxError`'s span against `self`, a `Document` over the entire file, while
the span itself carries form-local coordinates. **When the failing form is the
first one the two coordinate systems coincide**, which is why this looks
correct in every existing test.

Measured surface:

| Path | Behavior |
|---|---|
| `--file`, failure in the first form | correct (coincidence) |
| `--file`, failure in any later form | line and column both wrong |
| stdin script, later form | same |
| `--eval` (single form) | correct |
| **`Get`** | `Get::syntax: trailing input at bytes 6..7` — **no line or column at all**, and the byte range is form-local |

So there are two distinct problems: a wrong location where one is rendered, and
a missing location on the `Get` path.

**The pinned assertion, ruled.**
`eval_script_pins_the_second_form_syntax_error_message`
(`crates/rum-runtime/src/runtime/tests/context_and_loader.rs:882`) asserts
`trailing input at bytes 6..7` for `eval_script("x = 1\nw = 2 !")`, and its
comment calls the form-relative span a contract. **It is not a contract; it is
a regression guard whose anchor happens to be the defect.** Its stated purpose
is that brief-308's two-state form handling "cannot *silently* alter
diagnostics" — this change alters them loudly and with a reason, which is the
opposite case. And the decisive argument is on the rendering side:
`render_syntax_error` resolves that span against a `Document` over the whole
file, so the renderer already assumes file coordinates. The two systems
disagree and the span is the wrong one.

**You are authorized to update this one assertion** to the file-coordinate
value, and required to:

- keep its other assertion — that the first form still ran before the error
  surfaced — which is the actual two-state contract and is independent of
  spans;
- rewrite its comment to say the span is in file coordinates as of brief-385,
  and why, so the next reader does not restore the old anchor;
- leave every other pinned location alone. **If any other one moves, that is
  still a stop-loss** — this authorization covers this single assertion.

**A finding about the tests, which you must act on.** Ten assertions in the
repository pin `--> N:M`, and **all of the ones this brief's grep found are
`1:x`**
(`document.rs:497`, `cli.rs:178`, `terminal/src/lib.rs:289`,
`session_requests/api.rs:38`, `input_resume.rs:217`, `repl/src/tests.rs:363`
and `:455`, `left_chain_limits.rs:97`). That grep was incomplete — the first
dispatch found a REPL assertion at `2:4` (`rum-repl/src/tests.rs:946`) that it
missed, covering a multi-line **single** form. Confirm that one is unaffected
rather than assuming it: a single form's coordinates coincide with the input's,
so it should not move, and if it does that is a finding.

Still, not one existing assertion uses a multi-form script with a failure after
the first form. That shape is why the defect survived. Your new tests must
cover it; do not let the suite keep its blind spot.

## Goal

A syntax error reports the location of the offending text in the file the user
wrote, on every path that reports one.

**Fix the coordinates at the source, not at the renderer.** A span that leaves
the parser should already be in file coordinates, so that every consumer —
the CLI renderer, `Get`, the host diagnostic surface, the REPL — is correct
without each having to remember to compensate. If you conclude the span cannot
carry file coordinates and the offset must instead be supplied at render time,
say why, and then make it impossible for a consumer to forget it.

For `Get`, render a line and column rather than a byte range. Keep the message
tag (`Get::syntax`) and the structure; propose the exact wording in your report
— the director gates it. A byte range in the message text is acceptable only in
addition to a line and column, never instead.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/document.rs`
- `crates/rum-syntax/src/parser/script.rs`
- other `crates/rum-syntax/src/**` needed for span coordinates
- `crates/rum-runtime/src/runtime/context_and_load_helpers.rs`
- `crates/rum-runtime/src/runtime/session/session_helpers.rs`
- the runtime, CLI, REPL and syntax test modules covering syntax diagnostics

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple — a call
site forced by a changed signature, a struct literal forced by a new field, a
re-export forced by a moved item — you MAY edit unlisted; list each under
"mechanical ripple". Anything changing behavior, contract, or a test
expectation in an unlisted file: STOP and report. When unsure, it is the
second kind.

Out of scope: the evaluator, `Expr`/`Symbol` representation, public serde
shapes, `Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`,
`docs/**`.

## Required first step — inventory the surface

Before changing anything, enumerate and report:

- Every path that produces a syntax diagnostic — `--file`, stdin script,
  `--eval`, `Get`/`Needs`, the REPL, the host `session_requests` surface — and
  for each: does it render a location, is that location currently right, and
  against which coordinate system.
- Every existing assertion that pins a location, and whether your change moves
  it. The ten listed above are all `1:x` and should be unaffected; if any of
  them moves, that is a finding to report before you proceed, not a snapshot to
  update.

If a path turns out already correct, say so and leave it alone.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### Test matrix

The blind spot is the point, so cover it explicitly:

- A failure in the **second** form of a multi-form script, and in a **later**
  form still (the four-line probe above), asserting the exact line **and**
  column.
- The same script through stdin.
- The same script through `Get`, asserting a line and column appear.
- A failure in the **first** form, asserting the location is unchanged from
  today — this is the case that was accidentally right, and it must stay right.
- `--eval` unchanged.
- A failure whose form spans multiple lines, so the form's own internal line
  offset is exercised, not just its starting byte.
- CRLF line endings, if the location code inspects line breaks at all; if it
  does not, say so instead of adding a test that proves nothing.

**Do not update a pinned location to match new output without reading it.** A
snapshot updated to whatever the code now prints records the behavior rather
than checking it — and this whole defect is a case of output nobody checked. If
an existing assertion moves, report it with the before and after and say why
the new value is right.

### Correctness of the excerpt

`render_span_excerpt` prints the offending line with a caret. Confirm it points
at the right character in the multi-form case, not just that the `-->` header
is right — the excerpt uses the same span and would be wrong in the same way.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the evaluation path. Span bookkeeping happens on the
  parse path; if a fix would add per-node cost to parsing, say what it costs.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- **Stop-loss** — report rather than improvise if: spans cannot carry file
  coordinates without changing a public type the brief does not list; an
  existing pinned location **other than the one authorized above** moves; or
  `Get`'s message structure cannot gain a location without changing its tag.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The surface inventory — every diagnostic path, its current location
   behavior, and whether it needed a change.
2. What changed — per file, line ranges, one-line why, and where the
   coordinate translation now happens.
3. Mechanical ripple — every file edited under the criterion above.
4. Verification — `fmt`, workspace counts, and the new multi-form cases with
   their asserted line and column.
5. The proposed `Get` wording, for the director to gate.
6. Stop-loss / open questions.
