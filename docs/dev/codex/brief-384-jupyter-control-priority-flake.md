# brief-384 — the `rum-jupyter` ZeroMQ ordering flake (revision 1)

**Revision 1.** The first dispatch stopped correctly on this brief's own
stop-loss: thirty release runs passed in Codex's environment, so it refused to
fix a defect it could not see. That was the right call, and the fault is the
brief's — the defect is environment-dependent, and the repair's correctness
does not actually depend on reproducing it. Revised below: reproduction is now
reported, not required, and the acceptance criterion is a structural argument
the director then verifies in an environment where the flake does reproduce.


The flake `TODO.md` has tracked since brief-165 is diagnosed. This brief fixes
it. The diagnosis below is the director's, reproduced on the current tree. You
do not need to re-derive it, and you are not expected to reproduce it — see
**Acceptance is a structural argument** below for what does decide this brief.

## The diagnosis

**It reproduces in release mode in the director's environment, and not in
Codex's.** Director, on a 16-core Darwin host: 22 failures in 30 runs, then 14
in 20, of `cargo test -p rum-jupyter zeromq::tests::native --release` —
**always the same single test**, no other test ever failing. Codex's first
dispatch measured 0 failures in 30 runs of the identical command with the tests
confirmed running rather than skipped.

`--test-threads` is not the variable: at 1, 2, 4 and 9 threads the director
measured 6/10, 3/10, 7/10 and 4/10, with no monotonic trend. Serial execution
still fails, so the race is inside a single test — background workers moving
messages into the channel buffers against the main thread's poll — not a
contention between tests. Some environment difference (scheduler, loopback
stack) evidently keeps the control message punctual in the sandbox. **Race
conditions surface more readily on faster paths, so a green run there is weak
evidence.**

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

### Acceptance is a structural argument, not a reproduction

Debug mode cannot see this defect and your environment apparently cannot
either, so no run count you can produce will demonstrate the fix. **The
acceptance criterion is therefore structural**, and the director verifies it
afterwards in an environment where the flake reproduces at roughly 70%.

What you must deliver:

1. **The argument.** State plainly why the revised assertion cannot fail
   regardless of the order in which the two messages reach their buffers.
   Name the precondition you establish, and show it is established before the
   assertion rather than assumed. If the argument needs a timing assumption of
   any kind, you have not finished.
2. **Run it anyway and report the number.** Thirty release runs before and
   after, with the command line. Both are expected to be 0/30 in your
   environment; report them as corroboration, not as proof, and do not treat
   0/30 before the change as a reason to stop this time.
3. `cargo test --workspace` green, and the other eight native tests green.

A fix that merely makes the race less likely — a longer sleep, a bigger
timeout, a retry — is not acceptable, and here it would also be undetectable.
The assertion must be true by construction. If your repair still depends on a
duration, say so explicitly and explain why it cannot be made deterministic.

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
- **Stop-loss** — report rather than improvise if: the deterministic
  precondition cannot be established without changing polling behavior; or you
  conclude the implementation rather than the test is wrong. **Not reproducing
  the flake is no longer a stop** — it is the expected outcome in your
  environment and is why acceptance is structural.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The structural argument — the precondition, where it is established, and
   why the assertion cannot fail on either arrival order.
2. Reproduction counts before and after — command and counts out of thirty,
   reported as corroboration.
3. What changed — per file, line ranges, one-line why.
4. Mechanical ripple — every file edited under the criterion above.
5. Verification — `fmt` and workspace counts.
6. Stop-loss / open questions.
