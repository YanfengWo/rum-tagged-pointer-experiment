# Brief 122 — Diagnostic capability: hide internal instrumentation from restricted hosts

## Goal

Implement the settled sub-decision (2026-07-04): the profiling/debug
instrumentation STAYS in the kernel (runtime-gated), and its BUILTIN surface
is hidden from restricted embedders via the capability system instead of
feature-gating. Add a `Diagnostic` variant to `Capability`
(`crates/rum-runtime/src/runtime/capabilities.rs`, brief-095 machinery) and
tag the diagnostic builtins' specs with it. Restricted policies (e.g.
rum-host's read-only/Pure profiles) then deny them exactly like FileRead —
existing `General::sandbox` denial message, `$Failed` result. Unrestricted
sessions (the default) are COMPLETELY unaffected. Zero behavior change
outside restricted-policy sessions; workspace green (1301/0 + new tests).

## Context pointers

- `Capability` is a u8 bitmask enum (Pure/FileRead/FileWrite; add
  `Diagnostic = 1 << 3`). `CapabilityPolicy::allow([...])` is allowlist-based,
  so existing restricted profiles (rum-host `config.rs` Pure-only, the
  {Pure, FileRead} test profile) automatically exclude the new capability —
  verify, don't assume.
- Tagging mechanism: `BuiltinSpec::with_capability(Capability::Diagnostic)`
  on catalog entries (the brief-102+ pattern; specs for these builtins live
  in the KERNEL's residual catalog — `builtin_catalog/debug.rs` most likely;
  find them by name).
- The CORE tag set (mandatory): `RumProfileReset`, `RumProfileSnapshot`
  (rum-internal instruments). AUDIT the rest of the debug catalog against
  this criterion and propose the full list in your report: tag a builtin iff
  it is (a) a rum-internal instrument, or (b) it exposes HOST-SIDE resource
  information a sandboxed guest should not see (`MemoryInUse`,
  `MaxMemoryUsed` qualify — they read process-level state). Do NOT tag
  general-purpose WL language facilities (Timing/AbsoluteTiming — guest's
  own timing; TestCreate/TestReport; Trace/TracePrint; ByteCount — it
  measures the guest's own expression). Borderline calls go in the report
  with one-line reasoning.
- Tests: extend the existing capability tests (rum-runtime capabilities
  tests + rum-host policy tests at `crates/rum-host/src/tests.rs`) with:
  a Pure-only session gets `$Failed` + `General::sandbox` from
  `RumProfileSnapshot[]`; an unrestricted session still works; a policy
  explicitly allowing Diagnostic works.
- rum-host re-exports `Capability` — the new variant is automatically
  available; update the rum-host `lib.rs` doc comment's capability list
  (doc comments are code, in scope).

## Scope

- MAY modify: `runtime/capabilities.rs` (enum + tests);
  the kernel catalog file(s) holding the diagnostic specs (tag additions
  only); `crates/rum-host/src/lib.rs` (doc comment) + `crates/rum-host/src/tests.rs`;
  `kernel_api.rs` only if the compiler demands.
- MUST NOT touch: the profiling/debug implementation code (this brief tags
  specs, it does not move code), stdlib catalogs (diagnostic specs are
  kernel-resident; if you find one in the stdlib catalog, REPORT it — do not
  tag across the boundary without noting it), dispatch machinery, any
  `mma-hol` clone, `docs/**` except your report, `.git`,
  `Cargo.toml`/`Cargo.lock`.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-host
cargo run -p rum-cli --release -- --eval "First[RumProfileSnapshot[]] // Head" 2>/dev/null | tail -1   # unrestricted CLI still works (report actual output shape)
```

Plus paste the new capability tests' names and the full tag list with
per-builtin reasoning.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, `grep`,
  `python3`. Never edit any `mma-hol` file. **No new `unsafe`.**
- No default-path cost: the capability check already exists per dispatch;
  you are adding a bit, not a branch.
- Stop-loss: same compile error twice, or the tag criterion producing a
  surprising list (>12 builtins) → STOP and report.

## Report format

1. The full tag list (builtin → criterion (a)/(b) → catalog file), incl.
   borderline rejections with reasoning.
2. What changed — per file.
3. Verification — verbatim outputs incl. new test names.
4. Verdict — restricted-vs-unrestricted behavior assertion.
5. Stop-loss / open questions.
