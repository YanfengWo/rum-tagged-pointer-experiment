# Brief 183 — Gate Exit behind Capability::Process (sandbox-safety fix)

Implementation brief. Executes the user's 2026-07-07 decision from the
unit-4 audit (tracker `docs/dev/usability-pass.md`, unit-4 triage;
evidence E9 in
`docs/dev/usability-pass/control-logic-rewrite-findings.md`). `Exit`
requests host-level termination (`EvalError::Exit` →
`SessionRequestError::Exit`) but is registered as `Capability::Pure`
(the `ordinary` default), so `sandboxed()` installs it — a sandboxed
computation can emit a host-termination request (a DoS-shaped escape a
naive host could act on). Host lifecycle control is ambient authority,
not Pure — the same logic that put Clock/Random behind capabilities in
the FS-purity line. Fix: relabel `Exit` to `Capability::Process`.

## Change

1. In `crates/rum-stdlib/src/catalog/control.rs`, the `Exit` entry:
   chain `.with_capability(Capability::Process)` on its `BuiltinSpec`
   (mirror how a Clock/Random/FileRead stdlib builtin declares its
   capability — find the existing `.with_capability(...)` precedent in
   the stdlib catalogs and match the exact form; `Capability` is
   available via `rum_runtime::kernel_api::Capability` or wherever the
   catalog already imports it). Behavior under a trusted policy
   (allows `Process`) is UNCHANGED — the CLI/REPL `Exit` still works.
   Under `sandboxed()` (Pure-only), `Exit` is no longer installed /
   dispatched.
2. If the kernel attribute vocabulary or a capability cross-check needs
   `Exit`'s capability updated to stay consistent, do so (report what).

## Tests

- A rum-host test in the sandboxed-config test group (see
  `crates/rum-host/src/tests.rs` — the
  `sandboxed_host_config_blocks_clock_builtins_at_dispatch` /
  `..._blocks_diagnostic_builtins_at_dispatch` pattern): assert that
  under `HostConfig::sandboxed()` a call to `Exit[0]` is blocked at
  dispatch (mirror the existing clock/diagnostic blocking assertion),
  and that a trusted config still allows it.
- Keep the existing `Exit` behavior tests (arity `intm`/`argt`
  messages, `EvalError::Exit { code }` propagation under a trusted
  policy) passing unchanged.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# trusted CLI: Exit still terminates with the code (exit status)
printf '' | cargo run -q -p rum-cli -- --eval 'Exit[7]'; echo "cli-exit-status=$?"
# capability shows in the generated reference as Process now:
grep -A3 '^### Exit' docs/reference/builtins.md
git status --short
```

Expected: fmt clean; workspace green (report totals — new sandboxed
test added, existing Exit tests pass); the CLI `Exit[7]` still exits
with status 7 (trusted policy unchanged); the regenerated reference
shows `Capability: Process` for `Exit`. If the reference is generated
and needs regenerating for the capability line, do so.

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/control.rs` (Exit spec),
  the kernel vocabulary/capability cross-check file ONLY if a test
  requires it, `crates/rum-host/src/tests.rs` (new sandboxed test),
  `crates/rum-stdlib/src/tests/**` if an Exit capability test lives
  there, `docs/reference/builtins.md` (regenerated for the capability
  line).
- MUST NOT touch: `Exit`'s runtime behavior (still raises
  `EvalError::Exit { code }`), other builtins, the guide, `TODO.md`,
  the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- The ONLY behavior change: `Exit` is no longer available under
  `sandboxed()` (Pure-only); under any policy that grants `Process` it
  is byte-identical to today. No change to the exit code, the arity
  messages, or the `EvalError::Exit` propagation.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the `.with_capability` form used and the
   precedent mirrored; any vocabulary/cross-check update.
2. Verification — all commands verbatim (incl. the CLI exit status and
   the reference capability line).
3. Stop-loss / open questions (empty if none).
