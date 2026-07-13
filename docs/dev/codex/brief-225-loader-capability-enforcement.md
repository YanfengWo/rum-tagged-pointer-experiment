# Brief 225 — Loader FileRead check at the actual file-access seam

Fix brief — unit-10 batch 2 (correctness, L). Source:
`docs/dev/usability-pass/kernel-c-findings.md` (top-gap #1, the
`BeginPackage`/`DeclarePackage` capability rows, Evidence E3, and the
capability-label review table). This is an API-hygiene fix in the same
spirit as unit 8's protection-recheck (brief-209): a capability check
that exists on one entry path is missing on the others that reach the
same effect.

## The inconsistency (code-confirmed at the director's gate)

rum's `FileRead` capability check lives in EXACTLY ONE place — the
builtin dispatcher (`eval_dispatch/mod.rs:124-127`): a restricted policy
that does not allow `spec.capability` returns `sandbox_denied_builtin`
before the handler runs. `Get` and `Needs` are `Capability::FileRead`,
so a DIRECT `Get["f.wl"]` under a restricted policy is correctly denied
there.

But the loader's internal Rust methods `eval_get` / `eval_needs`
(`builtins_context_loader.rs:155`, `:201`) check only `has_file_io()`
(provider presence), NOT the capability policy. Two internal callers
reach them WITHOUT passing through the dispatcher, so the `FileRead`
check never runs:

1. `eval_begin_package` loads each declared dependency by calling
   `self.eval_needs(...)` directly (`builtins_context_loader.rs:308-310`).
2. Declared-package stub resolution autoloads by calling
   `self.eval_needs(...)` directly from symbol evaluation
   (`eval_core/symbol.rs:50-56`, gated behind
   `declared_stubs.contains(&symbol)`).

So `BeginPackage["P\`", {"Dep\`"}]` and evaluating an unresolved declared
stub both trigger a provider-backed file read on a path where the direct
`Get`/`Needs` call is denied.

**Honest boundary (state it, don't oversell it):** the default
`sandboxed()` config installs NO file provider
(`rum-host/src/config.rs:171`), so a bare sandbox reads nothing through
this path. The inconsistency matters for the composed config
`sandboxed().with_file_io(provider)` — a legitimate embedding where the
application sandboxes a plugin's capabilities but supplies a controlled
file provider; there, `FileRead`-labelled heads are expected to honor
the policy, and today the loader-internal path does not. This is a
label/behavior consistency fix, NOT a claim that the sandbox is broken.

## Required change

- Move the `FileRead` authorization to the ACTUAL file-access seam so
  every caller is covered uniformly. Add a single private guard (e.g.
  `ensure_capability_allowed(Capability::FileRead)` returning the same
  denial shape the dispatcher uses) and call it at the top of `eval_get`
  and `eval_needs`, BEFORE `has_file_io()`. Then all three paths
  (dispatch, BeginPackage dependency, stub autoload) are gated by the one
  check at the seam.
- A denied load must FAIL, not silently succeed: return the
  dispatcher's denial value / `$Failed`-consistent shape + the same
  `General::sandbox` message the dispatcher emits ("… is not permitted in
  this sandbox."), and the caller must propagate it — `BeginPackage`
  must NOT enter the package or push the denied context onto
  `$ContextPath` when a dependency load is denied (coordinate with
  brief-226's fail-closed work; if the rollback is bigger than this
  brief, at minimum do not SILENTLY proceed — emit the denial and return
  a failure the caller sees).
- **No double-charging, no observable change on the direct path**: a
  direct `Get`/`Needs` call still hits the dispatcher check first and
  never reaches `eval_get`/`eval_needs` under a restricted policy, so its
  behavior is byte-identical. The new guard only fires for the internal
  callers. Confirm this in the report (trace the direct path).
- **Unrestricted (trusted CLI) policy is byte-identical**:
  `capability_policy.is_unrestricted()` short-circuits the guard exactly
  as it does at dispatch. Loading works exactly as today for the CLI and
  for mma-hol.

## Method

1. Read brief-209 (the unit-8 `store_held_update` protection-recheck) —
   same shape: a check present on one path, missing on the sibling that
   runs the effect. Mirror its "recheck at the effect seam" resolution.
2. Add the guard; wire it into `eval_get` + `eval_needs`; verify the
   dispatcher check is now redundant-but-harmless for direct calls
   (do NOT remove it — defense in depth, and it carries the pre-handler
   denial shape).
3. Baseline-verify the preservation pins BEFORE and after: unrestricted
   `Get`/`Needs`/`BeginPackage`/declared-stub autoload all still load
   (use /tmp scratch package files).

## Perf gate + acceptance

- **Microbench A/B required** — the stub-autoload guard sits in symbol
  evaluation (`eval_core/symbol.rs`). Expected result: no measurable
  change, because the new check is behind the existing
  `declared_stubs.contains(&symbol)` gate (ordinary symbol evaluation
  never reaches it) — but prove it. Build the A/B binaries IN THE REPO
  PATH (checksums must be reproducible). Watch labels:
  `symbol-ownvalue-read`, `downvalue-exact-miss`, the whole hot panel;
  profiling env OFF, idle machine, STOP-and-report under load.
- **mma-hol Strict gate at MY gate** — this changes `Needs`/loader
  behavior; the director runs the full acceptance suite (the ~2h patched
  run) before crediting. Note it in the report; do not run it yourself.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes (use /tmp scratch .wl files), before/after:
#  - unrestricted: Get / Needs / BeginPackage[..,{dep}] / declared-stub
#    autoload all load (byte-identical)
#  - the host-level restricted+provider config: a rum-host test that
#    sandboxes the policy, installs a provider, and asserts BeginPackage
#    dependency load and stub autoload are now DENIED (mirror the
#    existing sandboxed_host_config_blocks_filesystem_builtins_at_dispatch
#    test — add the internal-caller cases)
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/builtins_context_loader.rs`
  (`eval_get`/`eval_needs` guard + the BeginPackage dependency-denial
  propagation)
- `crates/rum-runtime/src/runtime/eval_core/symbol.rs` (only if the
  autoload caller must handle a denial return; keep the touch minimal)
- `crates/rum-runtime/src/runtime/capabilities.rs` (only if the guard
  helper belongs there; name it)
- Tests: `rum-host` capability tests (the
  `sandboxed_host_config_blocks_filesystem_builtins_at_dispatch` family)
  + runtime loader tests; list each file; one new test file allowed
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md` only
  if a contract note is warranted (MERGE)

MUST NOT touch: the dispatcher check itself (leave it — defense in
depth), the `Input`/`Dialog` capability question (separate decision),
`FileWrite` heads (Save/DumpSave — separate; this brief is FileRead
only), the FileIo provider seam, `sandboxed()`'s no-provider default,
anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`, no capability-KIND changes (labels stay; this brief
  ENFORCES the existing `FileRead` label, it does not relabel anything).
- Unrestricted policy byte-identical; direct-call path byte-identical.
- Baseline-verify every preservation pin.
- One A/B for the batch; profiling OFF; STOP under load.
- Stop-loss: if gating at the seam cannot be done without changing the
  unrestricted load path, STOP and report.

## Report format

1. The change: the guard, the seam, the three now-covered paths; the
   direct-path trace showing byte-identity.
2. Test survey + the new restricted+provider denial tests.
3. Probe panel before/after verbatim (unrestricted loads unchanged;
   restricted+provider internal loads now denied).
4. A/B table + noise-floor verdict + raw paths + binary checksums.
5. Verification outputs verbatim; a note that the mma-hol Strict gate is
   the director's to run.
6. Deviations / open questions (empty if none).
