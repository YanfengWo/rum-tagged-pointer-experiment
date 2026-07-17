# Brief 251 — rum-cli feature forwarding (`hash` trimmable end-to-end)

## Goal

`cargo build -p rum-cli --no-default-features` produces a CLI binary whose
dependency graph contains none of the digest crates
(`sha2`/`sha1`/`md-5`/`crc32fast`), while the DEFAULT build stays exactly
what it is today. rum-stdlib's `hash` feature (brief-160) is already
NDF-clean; what is missing is the forwarding chain. Because cargo unifies
features, EVERY crate between rum-cli and rum-stdlib must forward:
`rum-host` (→ `rum-stdlib/hash`), `rum-terminal` (→ `rum-host/hash`),
`rum-repl` (→ `rum-host/hash` + NDF `rum-terminal`), `rum-cli` (→ all
three deps). One plain-default straggler re-enables the feature for the
whole graph — that is the failure mode this brief exists to avoid.

## Context pointers

- Read `AGENTS.md` first; TODO.md §Feature-Gating Policy is the policy
  background (do NOT edit TODO.md).
- Current facts (verified 2026-07-17): `rum-stdlib` has
  `hash = ["dep:sha2", "dep:sha1", "dep:md-5", "dep:crc32fast"]` with
  default-on; `rum-host` has NO `[features]` block and plain deps;
  `rum-terminal` → `rum-host`; `rum-repl` → `rum-host` + `rum-terminal`;
  `rum-cli` → `rum-repl` + `rum-host` + `rum-terminal` + `rum-syntax`,
  with one existing feature `alloc-count` (keep it untouched).
  `rum-jupyter` also depends on `rum-host` but is NOT in rum-cli's graph
  — leave it entirely alone (its default build must keep hash on via
  rum-host's default).

## Scope

- Files you MAY modify (Cargo.toml ONLY, in these four crates):
  - `crates/rum-host/Cargo.toml` — `[features] default = ["hash"];
    hash = ["rum-stdlib/hash"]`; declare `rum-stdlib` with
    `default-features = false`.
  - `crates/rum-terminal/Cargo.toml` — same pattern over `rum-host`.
  - `crates/rum-repl/Cargo.toml` — same pattern over `rum-host` AND
    `rum-terminal`.
  - `crates/rum-cli/Cargo.toml` — same pattern over `rum-repl`,
    `rum-host`, `rum-terminal`; preserve `alloc-count` as-is.
- `Cargo.lock`: feature declarations should not change it; if cargo
  rewrites it anyway, report the diff — do not hand-edit.
- MUST NOT touch: any `.rs` file, `rum-stdlib`/`rum-jupyter`/`rum-syntax`
  manifests, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except
  appending to your own report, `.git`.
- No `cfg(feature)` in source anywhere — if the chain seems to need one,
  STOP and report (it means a crate uses a digest symbol
  unconditionally, which would be a real finding).

## Deliverable

- The four manifest changes.
- Green `cargo fmt --all --check` and `cargo test --workspace`
  (default features — behavior unchanged; the 27 frozen CLI tests and
  REPL expectations must pass untouched).
- NDF proofs, all pasted verbatim:

  ```
  cargo check -p rum-cli --no-default-features
  cargo tree -p rum-cli --no-default-features -e normal | rg -c 'sha2|sha1|md-5|crc32fast'   # expect 0 matches (rg exits 1)
  cargo tree -p rum-cli -e normal | rg -c 'sha2'                                             # default: >0 (control)
  cargo build --release -p rum-cli --no-default-features
  target/release/rum-cli --eval '1 + 1' </dev/null                                          # smoke: prints 2
  ```

  NOTE the release NDF build overwrites `target/release/rum-cli` with a
  hash-less binary — after the smoke probe, rebuild the DEFAULT release
  binary (`cargo build --release -p rum-cli`) so the tree's binary state
  is left as found, and show that rebuild in the report.

- A `cargo check -p rum-jupyter` proof that the untouched jupyter crate
  still builds with defaults intact.

## rum / Rust pitfalls (read twice)

1. Feature unification is the whole game: check EVERY path into
   `rum-host` within rum-cli's graph (`cargo tree -p rum-cli -i
   rum-host` helps); a single `default-features` omission silently
   defeats the trim.
2. `default = ["hash"]` on all four crates keeps every existing build
   command byte-equivalent — no test expectations may change; if one
   fails, your wiring changed defaults somewhere.
3. The digest crates are OPTIONAL deps of rum-stdlib — `rum-stdlib/hash`
   is the correct forward target (not `dep:` names).
4. No network: no new packages are introduced; if cargo attempts a
   download, STOP and report.
5. No timing gate (manifest-only change; zero source edits).
6. Stop-loss: if NDF check surfaces a compile error in any crate (a
   latent unconditional digest use), STOP and report it as a finding —
   do not patch source.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --all --check
cargo test --workspace
```

Plus every NDF proof command above, verbatim, and the default-restoring
release rebuild.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you
  must.
- Touch NOTHING outside this repo, no network. Only Scope edits, `cargo`,
  the release `rum-cli` for the smoke probe, `python3`.
- Testing Policy hard line (`AGENTS.md`). No new `unsafe`. Minimal diff.
- Paste real verbatim verification output.

## Report format (your final message)

1. What changed — per manifest, the exact feature/dep lines.
2. Verification — verbatim fmt/test totals + all NDF proofs + the
   restored default build.
3. Confirmation Cargo.lock is unchanged (or its diff, explained).
4. Stop-loss / open questions.
