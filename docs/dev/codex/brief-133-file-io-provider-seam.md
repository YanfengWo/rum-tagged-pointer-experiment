# Brief 133 — Host-injected file-I/O provider; kernel std::fs-free by construction

Implementation brief. Second step of `docs/dev/fs-purity-plan.md` (read it
first). Behavior for the trusted default configuration must stay
byte-identical — the existing loader and persistence tests are the contract.

## Goal

`rum-core`/`rum-runtime`/`rum-stdlib` contain no `std::fs` outside test code,
enforced by a new hygiene test. The five remaining filesystem builtins
(`Get`, `Needs`, `FindFile`, `Save`, `DumpSave`) perform their I/O through a
small provider trait in the kernel API. The real `std::fs` provider lives in
`rum-host`; the trusted default installs it, `HostConfig::sandboxed()` does
not. `rum-cli`/REPL behavior is unchanged. Capability enforcement is
untouched (it still fires first at dispatch); provider absence is a second,
construction-level layer with a defined, tested outcome.

## Design

1. **Trait** (kernel API, e.g. `runtime::kernel_api::FileIo`):
   ```rust
   pub trait FileIo: Send {
       fn read_to_string(&self, path: &Path) -> Result<String, FileIoError>;
       fn probe(&self, path: &Path) -> FileProbe;   // exists / is_file, for FindFile & $Path resolution
       fn write_string(&self, path: &Path, contents: &str) -> Result<(), FileIoError>;
   }
   ```
   Adjust the exact shape to what the current loader/persistence code
   actually needs — the trait should be the MINIMAL closure over today's
   `std::fs` usage in `runtime/mod.rs` (and anything left in stdlib), not a
   speculative VFS. Pure path-string helpers (`FileNameJoin`,
   `DirectoryName`, …) do not touch the trait.
2. **Session plumbing**: the session holds `Option<Box<dyn FileIo>>` (or
   `Arc`, matching how session state is shared today), settable via the
   existing config/builder path. With NO provider and a policy that permits
   the capability, the five builtins emit one consistent message (suggest
   `General::noio`, wording like "no file provider installed") and return
   `$Failed`. Note: under `sandboxed()` the capability denial at dispatch
   still fires FIRST, so sandbox behavior/messages are unchanged.
3. **Real provider**: `StdFileIo` in `rum-host`, used by the trusted default
   `HostConfig` and exposed publicly. `HostConfig` gains a `file_io` choice
   (default Std for trusted; None for `sandboxed()`); document it in the
   existing rustdoc style (missing_docs is on).
4. **rum-cli / rum-repl**: install `StdFileIo` in their session
   construction. If `rum-cli` does not currently depend on `rum-host`,
   prefer adding the workspace-internal dependency over duplicating the
   provider; note what you chose in the report.
5. **Kernel tests**: runtime/stdlib tests that exercise Get/Save install a
   small test-local provider defined in test code (test code MAY use
   `std::fs` — the hygiene rule below permits it).
6. **Hygiene test**: a `#[test]` (suggest in `rum-runtime`) that walks
   `crates/rum-core/src`, `crates/rum-runtime/src`, `crates/rum-stdlib/src`
   at test time (via `CARGO_MANIFEST_DIR`-relative paths) and fails if the
   literal `std::fs` appears in any file whose path does NOT contain
   `/tests/`. Failure message explains the rule. Keep the rule this simple
   and path-based; if an inline `#[cfg(test)] mod tests` in a non-tests file
   needs fs, move that test to the tests tree instead of complicating the
   rule.

## Scope

- MAY modify: `crates/rum-runtime/src/**` (kernel_api trait, session config/
  state, the five builtins' I/O call sites, tests incl. the hygiene test),
  `crates/rum-stdlib/src/**` (any leftover `std::fs`, e.g. a stale import in
  `strings.rs`; tests), `crates/rum-host/src/**` (`StdFileIo`, `HostConfig`
  wiring, rustdoc, tests), `crates/rum-cli/src/**` +
  `crates/rum-repl/src/**` (provider installation only; plus `Cargo.toml`
  dependency additions if needed), `docs/guide/embedding.md` (HostConfig
  field list + one sentence) and `docs/guide/resource-limits-and-sandbox.md`
  (one sentence on the construction-level layer).
- MUST NOT touch: evaluator/dispatch stages beyond the five builtins' I/O
  call sites, matcher, `docs/reference/` (capability classes unchanged — if
  the drift test complains you changed catalog text, revert that), scripts,
  `.git`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
rg -n 'std::fs' crates/rum-core/src crates/rum-runtime/src crates/rum-stdlib/src
# ^ every hit must be under a /tests/ path — paste the list
cargo test -p rum-runtime <hygiene test name> -- --exact
printf 'BeginPackage["GuidePkg`"]\nanswer = 42\nEndPackage[]\n' > /tmp/rum-guide-pkg/GuidePkg.rum
cargo run -q -p rum-cli -- --eval 'Needs["GuidePkg`", "/tmp/rum-guide-pkg/GuidePkg.rum"]; GuidePkg`answer'   # expect 42
cargo run -q -p rum-cli -- --eval 'saved[x_] := x + 3; DumpSave["/tmp/rum-133-dump.rum", saved]' && cargo run -q -p rum-cli -- --eval 'Get["/tmp/rum-133-dump.rum"]; saved[4]'   # expect 7
cargo test -p rum-host 2>&1 | tail -3    # sandbox contract tests green
cargo test -p rum-host --doc 2>&1 | tail -2
git status --short
```

Also add and run one NEW rum-host test: a permissive-policy session with NO
provider evaluates `Get["/nonexistent"]` → the defined `General::noio` +
`$Failed` outcome (paste it).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Trusted-default behavior byte-identical: existing loader/persistence tests
  must pass UNCHANGED (do not edit their assertions; test-provider injection
  happens in shared test setup only).
- No new `unsafe`. No default-path cost (the provider is consulted only
  inside the five builtins' I/O paths, never per evaluation step). If you
  find yourself touching an eval/dispatch hot path, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Design as landed — trait shape, session plumbing, provider wiring, the
   cli/repl dependency choice.
2. What changed — per file.
3. Verification — all commands verbatim, incl. the hygiene-test hit list and
   the no-provider outcome.
4. Stop-loss / open questions (empty if none).
