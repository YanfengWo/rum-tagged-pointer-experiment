# Brief 242 — named embedding profiles: hermetic() + scripting(), capability surfaces pinned

Implementation brief — embedding API line, item a5, the line's last
queued item (see `docs/dev/embedding-api-line.md`; profile-set ruling
by the user 2026-07-16: minimal set — `hermetic` + `scripting`, no
consumer-less presets). The mechanism increment is deliberately near
zero; the value is NAMED CONTRACTS plus capability surfaces pinned by
exhaustive tests so they cannot drift silently.

## Pre-diagnosed facts (verified this session)

- `HostConfig::sandboxed()` (`rum-host/src/config.rs:180-186`) =
  `CapabilityPolicy::allow([Pure])` + no FileIo + no entropy.
- `HostConfig::default()` (`config.rs:155-170`) = unrestricted policy
  (`CapabilityPolicy::default()` = `unrestricted()`) + `StdFileIo` +
  `StdEntropySource` + core builtins — the de-facto trusted profile,
  just unnamed.
- The deterministic-PRNG split (stateful↔pure line):
  `RandomInteger`/`RandomChoice`/`SeedRandom[seed]` carry NO
  capability (Pure; fixed-seed deterministic; b4 pinned that a reset
  session replays the same stream); only `RandomBytes`/`ReseedRandom`/
  `RandomPassword` are `Capability::Random`
  (`rum-stdlib/src/catalog/random.rs:62,69,76`). Therefore
  `sandboxed()` is ALREADY mechanically hermetic — same input, same
  output.
- Capability universe (8, bitmask full): `Pure`, `FileRead`,
  `FileWrite`, `Clock`, `Random`, `Diagnostic`, `Process`,
  `Interactive` (`rum-runtime/src/runtime/capabilities.rs:2-13`).
- `rum-cli`'s `build_host_session` currently hand-wires
  `default().with_file_io(StdFileIo...).with_entropy_source(...)` —
  redundant (default already carries both).

## Required change

### Part A — the two named profiles (`rum-host/src/config.rs`)

1. **`HostConfig::hermetic()`** — implemented as `Self::sandboxed()`
   (shared implementation TODAY; separate functions so the promises
   can diverge without breaking either). Rustdoc is the deliverable —
   the determinism contract: same input → same output across sessions
   and resets; fixed-seed deterministic PRNG available
   (`RandomInteger` works and replays identically); no wall clock, no
   host entropy, no file I/O, no process control, no interactive
   suspension, no diagnostics; intended for build-step / compile-time
   evaluation. Cross-reference `sandboxed()` (the UNTRUSTED-code
   framing of the same surface) and state that the two may diverge in
   the future.
2. **`HostConfig::scripting()`** — explicit construction of what
   `default()` means: unrestricted capability policy + `StdFileIo` +
   `StdEntropySource` + core builtins. Rustdoc: the trusted
   local-scripting preset (the shape of `rum-cli`/`rum-repl`); full
   ambient authority — never hand untrusted code a scripting session
   (point to `sandboxed`/`hermetic`). Implementation may simply be
   `Self::default()` today, but write it as an explicit builder chain
   OR document that `default()` is its definition — your pick, say
   which and why in the report.

### Part B — capability surfaces pinned (tests)

3. In `rum-host/src/tests.rs`: for EACH of the three named profiles
   (`sandboxed`, `hermetic`, `scripting`), one test that walks ALL
   EIGHT capabilities and asserts the dispatch verdict per capability
   through a representative builtin probe (reuse the existing
   capability-probe patterns in the test file; e.g. Pure → `1+1`,
   FileRead → `FileNames`/`Get`, FileWrite → `Save`-shape, Clock →
   `AbsoluteTime[]`, Random → `ReseedRandom[]`, Diagnostic → the
   diagnostic builtin the tests already use, Process → `Exit`,
   Interactive → `InputString[]` suspension). The point: adding a
   ninth capability or changing any preset's surface must turn one of
   these tests red. Also pin: hermetic/sandboxed have no providers
   (provider-dependent probe fails), scripting has both (probe
   succeeds).
4. Hermetic determinism smoke: two `hermetic()` sessions evaluating
   the same source (including a `RandomInteger` stream) produce
   byte-identical rendered results.

### Part C — consumers + docs

5. `rum-cli/src/main.rs`: `build_host_session()` body becomes
   `HostSession::builder().with_config(HostConfig::scripting()).build()`
   (kills the redundant hand-wiring). The 27 CLI test expectations
   stay frozen.
6. `docs/guide/embedding.md`: a "Profiles" section — the three named
   presets, each one's promise, and a two-line chooser (untrusted code
   → sandboxed; deterministic build steps → hermetic; trusted local
   scripting → scripting).

## Scope

MAY modify:
- `crates/rum-host/src/config.rs`
- `crates/rum-host/src/tests.rs`
- `crates/rum-host/src/lib.rs` (docs only, if needed)
- `crates/rum-cli/src/main.rs` (the `build_host_session` body only)
- `docs/guide/embedding.md`

MUST NOT touch: `sandboxed()`'s behavior, the `Capability` enum /
`CapabilityPolicy`, anything in the kernel crates, `rum-repl`/
`rum-jupyter`/`rum-terminal`, CLI/REPL test expectations, `docs/**`
beyond the listed file.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: the change is config constructors plus a
  once-per-process construction call in `rum-cli`; no request-path or
  evaluator code is touched, and the in-process label loops never
  reach construction; state this in the report.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Behavior-preserving for all existing surfaces: no existing test
  expectation may change.
- Stop-loss: if a capability probe cannot be expressed with existing
  builtins/test patterns, STOP and report which one rather than
  inventing new kernel surface.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. The scripting() implementation choice (explicit chain vs default()
   alias) and why.
3. The 8-capability probe table per profile (capability → probe →
   verdict).
4. Test survey — each new test, named.
5. Verification — commands with verbatim output (test totals).
6. The no-A/B justification (one sentence).
7. Deviations / open questions (empty if none).
