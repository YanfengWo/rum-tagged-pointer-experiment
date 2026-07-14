# Brief 234 — per-request isolation + crash-only lifecycle contract

Implementation brief — host-embedding line, item b4 (see
`docs/dev/host-embedding-line.md`, concern (b) gates 3-4 and protocol
ask 2). Tests and documentation only; no evaluation-path or public-API
code changes. Two properties a protocol host relies on are currently
held by construction but under-tested: (1) **per-request isolation** —
state created in one request does not leak into the next when the host
resets between requests; (2) **crash-only lifecycle** — a session can be
abandoned in any state with no residue and no dependence on destructors,
because dun SIGKILLs a host on timeout/violation and relaunches cold.
This brief pins both with contract tests and states them in the guide.

## Current facts (verified 2026-07-15 — preserve, do not change)

- `HostSession::reset()` calls `build_inner_session(&self.config)`
  (`rum-host/src/session.rs:170`): it constructs a WHOLE NEW `Session`
  from the stored config, reinstalls core builtins + downstream
  installers, and drops every runtime-accumulated dimension. Config-
  carried state (resource limits, capability policy, contexts, file I/O
  and entropy providers, installers) survives; everything a request
  mutates does not.
- Runtime-accumulated dimensions all reset because the Session is
  rebuilt from scratch: `next_generated_symbol_id: 0`
  (`session_state.rs:399`) → Module gensym names repeat after reset;
  `prng_state: PrngState::default()` (`session_state.rs:393`) → the
  fixed-seed PRNG replays the same `RandomInteger` sequence after reset;
  definitions, attributes, options, messages, contexts-beyond-config,
  loaded packages all gone.
- Existing reset coverage is thin: `host_session_reset_clears_user_state`
  (`tests.rs:975`) checks ONE ownvalue; `..._recovers_poisoned_session`
  and `..._restores_original_config_not_mutated_state` cover poison and
  config. No test proves reset clears the full dimension set, and none
  states the isolation/crash-only contract.
- SIGKILL-safety is already structural: sandboxed sessions hold no
  providers, and the `std::fs` hygiene test (brief-233 neighbor) proves
  kernel library code never touches the filesystem — so a killed host
  leaves no temp files. This brief cites that mechanism; it does not add
  a heavyweight spawn+SIGKILL integration test (that belongs to the b5
  prototype, where a real host process exists — say so in the report).

## Required change

### 1. Multi-dimension reset completeness test

In `crates/rum-host/src/lifecycle_contract_tests.rs`, add a test that
mutates a session across every accumulation dimension, resets, and
proves it equals a fresh session. Establish in ONE session:

- ownvalue: `x = 41`
- downvalue: `f[n_] := n^2`
- attribute: `SetAttributes[g, Orderless]`
- option: `SetOptions[h, Foo -> 1]`
- message: `g::note = "text"`
- context change: `Begin["Custom`"]` (moves `$Context`)
- a gensym-advancing eval: `Module[{t}, t]` (bumps the gensym counter)
- a PRNG-advancing eval: `SeedRandom[7]; RandomInteger[100]`

Then `reset()` and assert:

- `session.definitions()` equals a freshly built `HostSession`'s
  `definitions()` (compare against a reference `HostSession::new()` —
  do NOT hardcode the list);
- `session.contexts()` equals the fresh reference's;
- probes: `x` → `Symbol("x")`; `f[3]` → unevaluated `f[3]`;
  `Attributes[g]` → empty list; `Options[h]` → empty; `$Context` →
  the default; and a post-reset `Module[{t}, t]` yields the SAME rendered
  gensym as the fresh reference does for the same expression (counter
  rewound); and `SeedRandom[7]; RandomInteger[100]` yields the SAME value
  before and after reset (PRNG rewound).

Confirm each probe's exact expected rendering by running it (mind the
`< /dev/null` stdin note below); do not guess renderings.

### 2. Per-request isolation contract test

Add a test modeling the reset-between-requests pattern dun uses:

- request 1: `counter = 1; counter` → `1`;
- `reset()`;
- request 2: `counter` → unevaluated `Symbol("counter")` (no leak);
- determinism: request 1 does `SeedRandom[1]; RandomInteger[10^6]` and
  records the value; after `reset()`, request 2 does the same and gets
  the IDENTICAL value — proving isolation reaches even the entropy
  stream (this is the property that lets a host pass explicit state as
  DATA rather than trusting hidden session carry-over).

Add a comment in the test tying it to the contract: explicit state is
passed as request data; implicit session state does not survive a reset.

### 3. Crash-only lifecycle contract test

Add a test proving a session abandoned mid-state does not contaminate a
subsequently constructed session (no process-global mutable state):

- build session A, mutate it heavily (several definitions), and drive it
  into the poisoned state via `eval_with_test_panic()` (the existing
  test hook), then `drop(a)`;
- build a fresh session B and prove it is completely unaffected: `2 + 2`
  → `4`, an undefined symbol stays a symbol, and `Module[{t}, t]` /
  `SeedRandom[3]; RandomInteger[9]` match a reference fresh session
  (B did not inherit A's gensym/PRNG advancement).

This is the library-level crash-only property: sessions are
self-contained, so a SIGKILL of the whole process (which dun does) can
never leave shared state behind. State that the OS-level
spawn+SIGKILL+no-temp-files integration test is deferred to b5.

### 4. Documentation

`docs/guide/embedding.md`: add a `## Per-Request Isolation` section
(place it after `## Reset`, before `## Snapshot And Restore`). State:

- `reset()` rebuilds the session from its configuration, dropping all
  user definitions, attributes, options, messages, non-config contexts,
  loaded packages, the gensym counter, and PRNG advancement, while
  keeping configured limits/policy/providers/installers;
- the isolation pattern for a host serving independent requests is
  reset-between-requests (or snapshot/restore when explicit carry-over
  is wanted); explicit state should be passed as request data, not
  relied on through implicit session carry-over;
- note the one snapshot/restore caveat already documented on those
  methods: a text snapshot captures user definitions, not the gensym
  counter or PRNG state — so reset-between gives the stronger
  determinism guarantee;
- crash-only: a host process may be terminated at any instant; the
  kernel keeps no temporary files (enforced by the `std::fs` hygiene
  test) and depends on no destructors for correctness, so a killed host
  leaves no residue and a relaunched one starts clean.

## Scope

MAY modify:
- `crates/rum-host/src/lifecycle_contract_tests.rs` (the three tests)
- `docs/guide/embedding.md` (the new section — this doc is IN scope,
  overriding the template default docs ban)

MUST NOT touch: any non-test source, `reset()`/`build_inner_session`,
snapshot/restore code, the existing reset/poison tests, anything else.
No new public API, no behavior change.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only the two MAY-listed files
```

- **No microbench A/B**: tests and documentation only; no
  evaluation-path code changed. State it.
- Probe note: `rum-cli` binds non-TTY stdin as `$ScriptInput`, so any
  probe run through the binary needs `< /dev/null` or it blocks forever.
  (The tests themselves use `HostSession`, not the binary, so this only
  affects any manual probing you do to confirm renderings.)

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Tests must assert against a fresh reference `HostSession`, not
  hardcoded dimension lists, so they stay correct if the default set
  changes.
- If any probe's real rendering differs from what this brief sketched
  (e.g. `Options[h]` empty form, a gensym rendering), use the REAL
  observed value and note the difference — do not force a wrong
  assertion.
- Stop-loss: if reset does NOT clear some dimension you test (a real
  leak), STOP and report it with the failing assertion — that is a
  finding, not something to paper over by weakening the test.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Test survey — the three tests, each dimension asserted, named.
3. Verification — commands with verbatim outputs (test totals).
4. Any probe whose real rendering differed from the brief's sketch.
5. Deviations / open questions (empty if none).
