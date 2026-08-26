# brief-358 — finish brief-357: test placement and the measurement gate

## Goal

brief-357's implementation is in the working tree and sound: `cargo test -p
rum-runtime` reports 831 passed, 2 failed, and both failures are the same
environment problem, not a behavior problem. Finish the job.

**Do not redo the implementation.** Keep the working tree's borrowed cursor,
the no-key cache probe, the threaded `Session`, and the removal of the
unreachable `PartSelector::Key` arm. Only the two red tests and the missing
measurement gate are in scope.

## Director's ruling on the stop-loss

`Key` is a stdlib builtin, so a source-level `Key[...]` selector cannot resolve
in a runtime-only `Session::new()`. **Move the source-level exact
value/message matrix and the suspension test into `rum-stdlib`, using
`core_session()`** — that is where association evaluation is installed, and it
is the same placement the association tests took in `b94a5a4a`.

Keep in `rum-runtime` only what does not need source-level `Key`: the
duplicate-key rule, the cache invalidation test, the below-threshold test, and
the wide-association reads. Those four are already green; leave them where
they are.

## Deliverable

- The two red tests, rewritten in `rum-stdlib` against `core_session()`,
  covering what brief-357 required: a present key, an absent key (exact
  `Missing[...]` shape), duplicate keys, a positional index in and out of
  range, negative indices, `Span`, `All`, mixed `Multi`, nested associations,
  below-threshold associations, and every `pmix`/`partw`/`partd` message with
  the same text, order and count as before the change. Plus the suspension
  test: a `Key[InputString[...]]` selector suspends and resumes into the
  original `Part` call.
- `cargo test --workspace` green.
- The measurement gate brief-357 never reached:
  - **Allocation counters first**: show the per-read entry clones are gone.
    A counter that does not move is stronger than any timing.
  - Stable microbench labels for a keyed read and a positional read at two
    association widths.
  - `--scale 5` A/B with the same-binary noise floor reported **first**, then
    base versus candidate. The property to demonstrate is that per-read cost
    **no longer scales with association width** — report both widths and the
    ratio between them on the candidate, not just the speedup.
  - Same-tier neighbors that must not move:
    ```
    association-set-key-incremental-small   association-set-key-incremental-large
    association-small-literal-repeat        deletecases-wide-association-no-match
    set-part-fill-small                     set-part-fill-large
    matcher-single-tiny                     symbol-ownvalue-read
    ```
    Name the untouched controls; never the suite median.

## Scope

- Files you MAY modify: `crates/rum-stdlib/src/tests/**`;
  `crates/rum-runtime/src/runtime/tests/association_part_cursor.rs` (to remove
  what moves out); `scripts/core-language-microbench.sh`. Implementation files
  only if a test reveals a real defect — and if one does, report it before
  changing anything.
- **The general accept/stop criterion**: *any behavior change at all is a
  stop.* brief-357 changes cost, not answers.
- MUST NOT touch: `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Do not weaken an assertion to make it pass.** If a moved test fails for a
   reason other than builtin registration, that is a finding about the
   implementation — report it, do not adjust the expectation.
2. **Messages are part of the contract**: same text, same order, same count,
   including when several selectors fail in one call.
3. **Counters before timings.**
4. **A 200-read probe at 32 000 keys took about 2.4 s before the change** —
   size your bounds from that, and expect the candidate to be much faster.
5. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report — but note that the previous stop-loss was an environment
   problem the ruling above resolves.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What moved and what changed — per file.
2. Anything a moved test revealed about the implementation (empty if nothing).
3. Verification — each command with verbatim output.
4. The exact-value and exact-message matrix.
5. The A/B — allocation counters first, then rows at two widths with the
   candidate's width ratio, the noise floor, and the controls named.
6. Stop-loss / open questions.
