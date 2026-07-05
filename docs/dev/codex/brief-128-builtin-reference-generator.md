# Brief 128 — Builtin reference generator + drift test

Implementation brief. Fifth step of `docs/dev/release-transformation-plan.md`.
This creates the generated half of the documentation-as-contract model
(`docs/testing-strategy.md` §Documentation Is the Contract).

## Goal

`docs/reference/` contains a generated builtin reference covering EVERY
builtin installed by `install_core` (both the kernel-resident catalog in
`rum-runtime` and all `rum-stdlib` families): one alphabetical entry per
builtin with its attributes, sandbox capability, one-line summary, and an
optional limits note. A `#[test]` drift gate regenerates the reference
in-memory and fails if the committed files disagree, so any contract change
shows up as a documentation diff in review.

## Context pointers

- `BuiltinSpec` lives in `crates/rum-runtime/src/runtime/metadata.rs` — all
  fields `pub`: `name`, `summary`, `attributes: AttributeSet`, `dispatch`,
  `handler`, `capability: Capability`.
- Builtins come from TWO sources: `rum-runtime`'s `builtin_catalog/` modules
  and `rum-stdlib`'s `catalog/` families (14 arrays). The ONLY complete,
  future-proof enumeration is the runtime registry after `install_core` on a
  fresh `Session`. Use the registry, not the static arrays.
- If no public read-only API exists to iterate installed builtin specs, add a
  minimal one (e.g. an iterator over registered `BuiltinSpec`s) to the
  kernel API, marked `#[doc(hidden)]` — read-only, no behavior change.

## Scope

- MAY create: `crates/rum-stdlib/src/docgen.rs` (+ submodules; declared as
  `#[doc(hidden)] pub mod docgen;`), `crates/rum-stdlib/src/bin/generate-builtin-reference.rs`,
  `docs/reference/README.md`, `docs/reference/builtins.md` (generated).
- MAY modify: `crates/rum-stdlib/src/lib.rs` (module decl),
  `crates/rum-stdlib/src/tests.rs` (+ a new test module file) for the drift
  test, and — ONLY if registry iteration is missing — a minimal read-only
  addition in `crates/rum-runtime/src/runtime/metadata.rs` /
  `kernel_api` re-export.
- MUST NOT touch: builtin implementations, catalog spec contents, evaluator
  code, other docs, `Cargo.toml` beyond nothing (bins under `src/bin/` are
  auto-discovered), `.git`.

## Design requirements

1. **Deterministic output.** Entries sorted by builtin name; attributes
   rendered in one fixed canonical order; capability rendered as
   `Pure` / `FileRead` / `FileWrite`; stable header/footer text. Two runs on
   the same code produce byte-identical files.
2. **Entry shape** (keep it compact, one entry ≈ 4-6 lines):
   ```markdown
   ### Divisors

   Returns the sorted positive divisors of an exact integer.

   - Attributes: `Listable`, `Protected`
   - Capability: `Pure`
   ```
   plus, when a limits note exists:
   ```markdown
   - Limits: trial-division based; intended for moderate-size integers.
   ```
3. **Limits sidecar**: `docgen` holds a `LIMITS: &[(&str, &str)]` table.
   Seed it with entries for `Factor`, `FactorInteger`, `PrimeQ`, `Prime`,
   `PrimePi`, and `DumpSave` — but FIRST verify each claim by reading the
   implementation (e.g. is `Factor` structural-only? is `PrimeQ` trial
   division? is `DumpSave` a textual format?). Write only what the code
   supports; skip a seed you cannot verify and report it. The drift test
   must fail if a `LIMITS` key names a builtin that does not exist.
4. **README.md** (hand-written once, small): states the file is generated,
   the regen command
   (`cargo run -p rum-stdlib --bin generate-builtin-reference`), the builtin
   count, and that the drift test enforces sync.
5. **Drift test**: regenerates `builtins.md` content in-memory via the same
   `docgen` code and compares against the committed file (plus the LIMITS
   validity check). Failure message names the regen command. Keep it in the
   ordinary `cargo test -p rum-stdlib` suite (fast — one Session build).
6. The bin locates the repo root via `CARGO_MANIFEST_DIR/../..`; it prints
   the entry count and output path.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo run -p rum-stdlib --bin generate-builtin-reference
git status --short
head -40 docs/reference/builtins.md
rg -c '^### ' docs/reference/builtins.md          # entry count
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# paste the three entries for Divisors, Map, StringJoin verbatim
```

Then a drift-test negative check: temporarily append a byte to
`docs/reference/builtins.md`, run the drift test, paste the failure line,
restore the file, rerun to green.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- No behavior change to any builtin or evaluator path; the only allowed
  `rum-runtime` change is the minimal read-only iteration API if missing.
- No new `unsafe`.
- Deterministic generation is a gate: run the bin twice, `git diff` must be
  empty after the second run.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file; whether a registry-iteration API had to be added.
2. Verification — each command with verbatim output (incl. entry count and
   the negative drift check).
3. Limits seeds — which were verified and written, which skipped and why.
4. Stop-loss / open questions (empty if none).
