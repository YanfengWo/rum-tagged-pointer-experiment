# Brief 253 — Registry enforcement: debug assertion at the push_message seam

## Goal

Keep the registry honest going forward: in debug builds, every
`push_message(symbol, tag, …)` from builtin code must name a
(symbol, tag) pair that the registry or a user MessageName definition
knows. The whole workspace suite running in debug thereby becomes a
continuous completeness proof — a future push_message site with an
unregistered tag fails loudly at test time instead of silently rotting
the catalog. Zero release-path cost: the check compiles away outside
`debug_assertions`. This is the closing brief of the registry line
(mechanism 245, fills 246-248, docgen 249, carriers 252 all landed).

## Context pointers

- Read `AGENTS.md` first.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:165` —
  `push_message`, the single seam every Table-A site funnels through.
- `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:13-27` —
  `eval_message`, the USER-driven emitter (`Message[Set::custom, …]`
  pushes arbitrary pairs by design) — it must bypass the check.
- Registered-spec lookup and `message_definitions` — both already
  consulted by the brief-245 catalog merge; reuse those lookups.
- Out of the seam by design (do not chase): the EvalError resource
  family constructs `RuntimeMessage` directly (its pairs are registered
  on `General` since brief-252); `debug_tools/test_report/evaluation.rs`
  constructs normalized `RuntimeMessage` values for report data, not
  emissions.

## REVISION 2 (2026-07-17, after the second stop-loss)

The second run showed the 11 rejected pairs come from rum-runtime unit
tests whose sessions register MINIMAL test-local specs and never install
the stdlib catalog — there is no fallback to inherit messages from, so
the fallback fix cannot cover them. RULING: the enforcement rule is
refined to the repository's established hybrid-session philosophy
(empty declaration = fall through): **a spec with an EMPTY `messages`
slice is exempt** — only a spec that DECLARES messages is held to
completeness. Honest trade-off, accepted: a future builtin that forgets
`.with_messages` entirely escapes the assertion (docgen/review catch
that); the real catalog surface is covered because full-session suites
(stdlib installs the catalog; 489 message-bearing specs all declare)
run under the strict check. Keep the `with_fallback_dispatch`
messages-inheritance fix (correct for overlay sessions) and the probe
test symbol switch — both already in the tree.

## Design (pinned — implement exactly)

1. Inside `push_message`, a `#[cfg(debug_assertions)]` block AFTER the
   `message_enabled` early-return, allowing the push if ANY of:
   (a) the symbol does NOT resolve to a registered builtin spec (user
   symbols and held-call heads — the open families stay free);
   (a') the resolved spec's `messages` slice is EMPTY (undeclared —
   minimal/test-local specs fall through, per Revision 2);
   (b) the resolved spec's `messages` slice contains the tag;
   (c) a user `MessageName` definition exists for the exact key.
   Otherwise `debug_assert!(false, …)` naming the pair and pointing to
   the owning catalog file.
   The firing-path test must therefore use a spec WITH declared
   messages — e.g. a plain kernel session emitting `("Set", "bogus")`
   (Set is kernel-cataloged with six declared pairs); the exempt-path
   tests cover a test-local empty-slice spec and a user symbol.
2. `eval_message` switches to a new `pub(in crate::runtime)`
   `push_message_unchecked` (the current body, no assertion);
   `push_message` keeps its signature so all ~114 builtin sites are
   covered without edits.
3. Release builds: the cfg block compiles out — zero new instructions,
   no allocation, no lookup on the emit path.

## REVISION (2026-07-17, after the first dispatch's stop-loss)

The first run proved the assertion works and exposed two PRE-EXISTING
seams (12 debug failures, zero real registry gaps — the 11 cataloged
pairs' rejection list is kept as evidence in the dispatch log):

1. **Hybrid test sessions lose messages on overlay**:
   `with_fallback_dispatch` (`metadata.rs:452-463`) inherits
   dispatch/handler/capability but NOT `messages`, so a test-local
   registration overlaying a catalog builtin yields an empty messages
   slice and the assertion fires on legitimate emissions. FIX (now in
   scope, pinned): in `with_fallback_dispatch`, when `self.messages`
   is empty, inherit `fallback.messages` — mirroring the existing
   handler-fallback semantics.
2. **`General::stop` in `session_config.rs:245`**: the
   `trace_message_and_print_accessors_expose_and_take_buffers` test
   exercises buffer accessors; its symbol choice was incidental.
   FIX (pinned): switch that probe emission to a non-builtin symbol
   (e.g. `"probe"`), updating its tag assertion accordingly — do NOT
   use the unchecked variant there.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/session/session_helpers.rs` — the
    seam + the unchecked variant.
  - `crates/rum-runtime/src/runtime/messages/builtins_messages.rs` —
    the one-line eval_message switch.
  - `crates/rum-runtime/src/runtime/metadata.rs` — MECHANICAL: the
    `with_fallback_dispatch` messages-fallback fix pinned above.
  - `crates/rum-runtime/src/runtime/tests/session_config.rs` — the
    pinned probe-symbol switch only.
  - Any test expectation legitimately moved by the messages fallback
    (hybrid sessions now list catalog messages for overlaid builtins)
    — empirically verified, old→new listed per test.
  - `crates/rum-runtime/src/runtime/tests/messages.rs` — new tests: an
    open-family push (user symbol) stays allowed; a user-defined
    MessageName on a builtin tag stays allowed; the assertion fires for
    a registered builtin + unregistered tag (use
    `#[should_panic]`-style or `catch_unwind` per the file's precedent
    for panic tests — match existing conventions).
  - IF (and only if) the assertion exposes registry gaps in the
    existing suite: the owning catalog file gains the missing entry
    (MECHANICAL, evidence cited per pair) — each such addition is a
    FINDING to list in the report. Never weaken the assertion to make a
    test pass.
- MUST NOT touch: emission-site texts/tags, mechanism/merge code,
  docgen + the generated reference (drift stays green untouched —
  unless a gap-fix catalog addition changes it, in which case
  regenerate and show the additions-only diff), `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, other `docs/**` except appending to your
  own report, `.git`, `Cargo.lock`.

## Deliverable

- The seam change + eval_message switch + tests.
- Green `cargo fmt --all --check`; `cargo test --workspace` with only
  the 9 known sandbox loopback failures, plus
  `cargo test --workspace --exclude rum-jupyter` fully green — in
  debug, so the whole suite runs under the assertion.
- The findings list: every registry gap the assertion exposed (or an
  explicit "none").
- A cfg proof that release is untouched: `cargo build --release`
  succeeds and the assertion block is `#[cfg(debug_assertions)]`-gated
  (cite the lines).

## rum / Rust pitfalls (read twice)

1. The assertion must not allocate or lookup in release — everything
   inside the cfg block.
2. `message_enabled` short-circuit ordering: a disabled (Off'd) message
   returns early today — keep the assertion AFTER it so Off semantics
   are unchanged (an Off'd bogus pair not asserting is acceptable and
   keeps the disabled path cost identical).
3. Do not convert any existing test to release mode to dodge a panic —
   a firing assertion is a finding (see Scope).
4. No timing gate; the seam is not the eval hot path, and the release
   path is unchanged by construction.
5. Stop-loss: if the suite surfaces MANY gaps (>10 pairs), STOP after
   listing them — bulk gap-filling may deserve its own brief.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace                      # debug; 9 known loopback failures only
cargo test --workspace --exclude rum-jupyter
cargo build --release -p rum-cli
```

Paste totals, the findings list, and the cfg citation verbatim.

## Hard rules

- No `git commit`/branch/push/config. Leave changes in the working tree.
- Nothing outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`,
  the release `rum-cli`, `python3`.
- Testing Policy hard line (`AGENTS.md`). No new `unsafe`. Minimal diff.
- Verbatim outputs only.

## Report format (your final message)

1. What changed — seam diff summary, eval_message switch, tests.
2. Verification — verbatim totals (loopback failures called out), the
   release-build proof, cfg citation.
3. Findings — registry gaps exposed (each with site + added entry), or
   "none".
4. Stop-loss / open questions.
