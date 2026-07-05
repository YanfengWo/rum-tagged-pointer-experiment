# Brief 130 — rustdoc pass (rum-host), Cargo metadata, top-level housekeeping

Implementation brief. Seventh and last Codex step of
`docs/dev/release-transformation-plan.md` before the M2 final sweep.
Documentation and manifests only — **zero code-semantics changes**.

## Goal

`rum-host`'s public API is fully rustdoc-documented (crate root quick start +
every public item) with `#![warn(missing_docs)]` enabled and zero missing-docs
warnings; the other library crates have crate-level `//!` overviews; every
workspace crate has release-shape Cargo metadata (description, inherited
license, keywords/categories); the stray top-level
`ISSUE-sandbox-profile-fs-builtins.md` is archived. Workspace tests and
doctests stay green.

## Context pointers

- `docs/guide/embedding.md` — the narrative the rustdoc should agree with
  (outcomes, resumable input, reset, snapshot, sandbox, cancel/timeout,
  threading contract). Rustdoc is the API-level companion.
- Root `Cargo.toml` already carries `[workspace.package]`
  version/edition/rust-version/license.
- Archive root: `/Users/fft/Developments/Archive/rum-history/` (MANIFEST.md
  entry format as before; 移除 commit placeholder exactly
  `（由 Claude 在提交时回填）`).

## Scope

- MAY modify: `crates/rum-host/src/**` (doc comments + the
  `#![warn(missing_docs)]` attribute ONLY), `crates/*/src/lib.rs` crate-level
  `//!` docs for `rum-core`, `rum-syntax`, `rum-runtime`, `rum-stdlib`
  (light: one honest paragraph each, if missing), root `Cargo.toml`
  (`[workspace.package]` keywords/categories), each crate's `Cargo.toml`
  (description + workspace inheritance fields).
- MAY do (housekeeping): `git rm ISSUE-sandbox-profile-fs-builtins.md` after
  archiving a verbatim copy; append its MANIFEST entry (性质: resolved issue
  record for the brief-095 capability sandbox; follow-up already tracked in
  `TODO.md` as the scoped-fs-roots item).
- MUST NOT touch: any executable code path, any test assertion, other docs,
  scripts, `.git`.

## Metadata requirements

1. `[workspace.package]` gains:
   - `keywords = ["symbolic", "term-rewriting", "sandbox", "scripting", "embedded"]`
   - `categories = ["mathematics", "compilers"]`
   (≤5 keywords, each ≤20 chars — crates.io rules.)
2. Every crate `Cargo.toml`: ensure `license.workspace = true`,
   `keywords.workspace = true`, `categories.workspace = true` (adjust
   categories per crate where clearly better, e.g. `rum-jupyter` →
   `["development-tools"]`), and a one-line `description` that matches the
   crate's role as stated in `README.md` §Workspace. No vendor names in any
   metadata field.
3. Do NOT add `repository`/`homepage` (no public remote exists yet — added at
   publication time).

## Rustdoc requirements (rum-host)

- Crate root `//!`: what the crate is, a minimal eval example (mirroring
  `examples/basic_embedding.rs`), pointers: guide `embedding` chapter for the
  narrative, `DISCLAIMER.md` for project boundary.
- Every public type, method, enum variant field where rustdoc expects it:
  a one-to-three-line doc stating contract, not implementation. Reuse the
  guide's wording where it fits (they must not contradict).
- Rust code blocks in docs are doctests — keep them compiling (use
  `no_run` where evaluation would be slow; avoid `ignore`).
- Add `#![warn(missing_docs)]` at the crate root; end state = zero warnings
  from `cargo doc -p rum-host`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo doc -p rum-host --no-deps 2>&1 | tail -5          # zero warnings
cargo doc --workspace --no-deps 2>&1 | grep -ci warning  # report count (other crates' warnings are report-only)
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo test -p rum-host --doc 2>&1 | tail -3              # doctests green
cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; [print(p["name"], "::", p["description"]) for p in json.load(sys.stdin)["packages"]]'
rg -in 'wolfram|mathematica|wstp' crates/*/Cargo.toml Cargo.toml || echo METADATA-CLEAN
ls ISSUE-sandbox-profile-fs-builtins.md 2>&1; ls /Users/fft/Developments/Archive/rum-history/ISSUE-sandbox-profile-fs-builtins.md
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Outside-repo writes ONLY under `/Users/fft/Developments/Archive/rum-history/`.
- Doc comments and manifests only; if a doc forces you to notice an API wart,
  REPORT it — do not change the API.
- Test totals must match the pre-change totals except for ADDED doctests.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — rum-host doc coverage summary (items documented, doctest
   count), per-crate metadata lines, housekeeping.
2. Verification — each command with verbatim output.
3. API warts noticed while documenting (report-only list; empty if none).
4. Stop-loss / open questions (empty if none).
