# brief-384 — the `rum-jupyter` ZeroMQ ordering flake

The flake `TODO.md` has tracked since brief-165 is diagnosed. This brief fixes
it. The diagnosis below is the director's, reproduced on the current tree; you
do not need to re-derive it, but you **do** need to confirm the reproduction
before and after your change.

## The diagnosis

**It reproduces on demand in release mode.** Thirty runs of
`cargo test -p rum-jupyter zeromq::tests::native --release`: **22 failed, all
of them the same single test**, no other test ever failing.

```
test zeromq::tests::native::frontend::control::
     native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... FAILED

thread '...' panicked at crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs:166:5:
assertion `left == right` failed
  left: Channel(Shell)
 right: Channel(Control)
```

In debug mode it is rare — the three historical sightings were all incidental
`cargo test --workspace` runs, and this line's own six-plus workspace runs
never hit it. **That asymmetry is itself a finding**: the test gate runs debug,
so this defect is nearly invisible to it.

**The mechanism.** `control.rs:148-165` sends a shutdown on the control
channel, sleeps 30 ms, sends an execute on the shell channel, sleeps 70 ms, and
then asserts that the *next* socket event is `Channel(Control)`.

The implementation's priority is correct and is not the bug:
`PriorityKernelSocketPoller::next_event` (`socket.rs:191-229`) lists `Control`
first in every branch. But it selects among channels that are **already
pending** — `session.has_pending_channel(channel)`. Native transport moves
messages into those buffers on background workers, so "sent first, after a
sleep" is not the same as "buffered first". When only the shell message has
landed at the moment of the poll, the poller correctly returns `Shell`, and the
test's `assert_eq!` on the *next* event fails.

So the test asserts a timing property ("the message I sent first arrives
first") while the implementation promises a priority property ("when both are
pending, control wins"). Release timing makes the two diverge often.

**It is the only place with this shape.** `wait_for_next_socket_event` — the
"next event must be X" helper — is used exactly once in the whole native
suite, at `control.rs:165`. The other twenty-one waits use
`wait_for_socket_event`, which loops until the expected event appears and is
order-insensitive by construction.

## Goal

Make the test assert the property the implementation actually promises, so it
is deterministic without weakening what it covers.

The test's subject, per its name, is that **shutdown stops shell work and
pending shell messages are reported** — the subsequent assertions on
`report.shell_messages`, `stopped_for_shutdown` and the pending counts are the
real content. The `first_event` assertion is a means, and as written it is
both too strong (it demands a race outcome) and beside the point.

The repair must still cover control priority rather than delete it. Establish
that **both channels are pending**, then assert that the poller yields
`Control` first. That is exactly the contract `next_event` implements, and it
is deterministic.

Decide the mechanism yourself and justify it — a helper that waits until both
channels report pending, or an equivalent that makes the precondition explicit.
If reaching `has_pending_channel` from the test requires widening visibility,
say what you widened and why the narrower option did not work.

## Scope

Files you MAY modify:

- `crates/rum-jupyter/src/zeromq/tests/native/frontend/control.rs`
- `crates/rum-jupyter/src/zeromq_native_test_support.rs`
- `crates/rum-jupyter/src/socket.rs` — **only** if the test needs a
  test-visible way to observe pending state; no change to polling behavior
- `crates/rum-jupyter/src/zeromq/tests/mod.rs`

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple — a call
site forced by a changed signature, a re-export forced by a moved item, a
visibility change forced by a new test-support helper — you MAY edit unlisted;
list each under "mechanical ripple". Anything changing behavior, contract, or a
test expectation in an unlisted file: STOP and report.

**Do not change polling or priority semantics.** `PriorityKernelSocketPoller`
is correct; this is a test-side repair. If you conclude the implementation is
actually wrong — for instance that a received shutdown should pre-empt shell
work already buffered — STOP and report that as a finding rather than changing
it. That would be a semantic decision for the director and the user, not part
of this brief.

**Do not touch the other twenty-one waits.** They are order-insensitive
already; converting them is scope creep.

Out of scope: the kernel, `Expr`/`Symbol` representation, `Cargo.lock`, `.git`,
`TODO.md`, `PROGRESS.md`, `AGENTS.md`, `docs/**`.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The reproduction gate — this is the real one

Debug mode cannot see this defect, so a green `cargo test --workspace` proves
nothing here. You must:

1. **Before your change**, run
   `cargo test -p rum-jupyter zeromq::tests::native --release` **thirty times**
   and report the failure count. Expect roughly 22/30. If it does not
   reproduce for you, STOP and report that — do not fix a defect you cannot
   see.
2. **After your change**, run the same thirty times and report the count. It
   must be **0/30**.
3. Paste both counts verbatim, with the command line used.

A fix that merely makes the race less likely — a longer sleep, a bigger
timeout, a retry — is not acceptable. The assertion must be true by
construction, not by timing. If your repair still depends on a duration, say
so explicitly and explain why it cannot be made deterministic.

### Coverage that must survive

- Control priority is still asserted, now under a stated precondition.
- The pending-message report assertions (`report.shell_messages`,
  `stopped_for_shutdown`, the pending counts) are unchanged.
- The other eight native tests stay green in both profiles.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. Testing Policy is a hard line: rum-vs-rum only, no other
  vendor's engine, no oracle harness.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape. Thirty release runs of this filter take well
  under a minute total; size accordingly.
- **Stop-loss** — report rather than improvise if: the flake does not reproduce
  for you; the deterministic precondition cannot be established without
  changing polling behavior; or you conclude the implementation rather than the
  test is wrong.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. Reproduction before the change — command and failure count out of thirty.
2. What changed — per file, line ranges, one-line why, and the mechanism you
   chose for the precondition with its justification.
3. Mechanical ripple — every file edited under the criterion above.
4. Verification — `fmt`, workspace counts, and the after-change count out of
   thirty release runs.
5. Stop-loss / open questions.
