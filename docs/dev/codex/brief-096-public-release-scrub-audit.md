# brief-096 — public-release scrub + audit of `release-staging/`

## Goal

Audit and scrub the pre-assembled public-release tree at `release-staging/` (a gitignored
staging copy of the shippable subset) so it is (1) a self-contained, working project that
builds and tests green, and (2) free of sensitive / internal / positioning-inappropriate
content in comments, identifiers, test names, and docs. The public project is positioned as
an **independent sandboxed expression evaluator with a Mathematica-style syntax subset and its
own semantics — NOT a Wolfram-compatible / parity project.** Nothing in the public tree may
present it as replicating, benchmarking, or achieving parity with any Wolfram product.

## CRITICAL scope

- Operate **ONLY inside `release-staging/`**. Do NOT touch any file OUTSIDE it (the dev repo's
  tracked files, `crates/` at the repo root, `docs/`, `scripts/`, etc. are OFF LIMITS —
  `release-staging/` is a separate copy; the originals stay pristine).
- Do NOT `git commit`/branch/push (staging is not even a git repo).
- Do NOT change the public crate NAMES (`rum-core`, `rum-cli`, …) or the public API surface
  (pub items used across crates) unless a rename is trivially safe and you re-verify green.
  Prefer changing comments, INTERNAL identifiers, test names/content, and docs.

## Part A — make it a working standalone project

```
cd release-staging
cargo build --workspace          # must be green (already verified once)
cargo test --workspace           # make it green
```
Some tests were written against a Wolfram-kernel differential oracle or the `mma-hol` corpus
(dev-only assets that are NOT in this tree). For each failing / inappropriate test:
- If the test verifies **rum's own behavior** and merely has oracle/parity WORDING → keep the
  test + its value assertions, and neutralize the wording (see Part B).
- If the test **depends on an excluded asset** (a Wolfram kernel, `mma-hol` files, snapshot
  files, the difftest harness) or is **inherently about Wolfram parity/divergence** (e.g.
  `mma_hol_cold_lane`, the `#[ignore]`'d Flat/Orderless+Optional "WK divergence" specs) →
  REMOVE it (and any now-dead helper), keeping the suite green.
`cargo test --workspace` must end green with no ignored-for-WK-divergence tests remaining.

## Part B — scrub sensitive / positioning content

Scan the WHOLE `release-staging/` tree (code comments, identifiers, string literals, test
names, doc files) for these terms (case-insensitive) and remove/neutralize them:

`Wolfram`, `Mathematica`, `WolframKernel`, `wolframscript`, `MathKernel`, `Wolfram Language`,
`WL semantics`, `mma-hol`, `mma_hol`, `parity`, `oracle`, `difftest`, `diff-test`,
`byte-identical`, `WK` (as "Wolfram kernel"), `WK-calibrated`, `WK-conformant`, `drop-in`,
`compatible replacement`, `bug-for-bug`, `Gosper`(if it names WK behavior), brief/PROGRESS
back-references, and any comment describing "matches/diverges-from Wolfram".

Fix policy:
- **Comments / doc strings**: rewrite to describe the code's behavior in the project's OWN
  terms (e.g. "canonical form under this evaluator", "the evaluator's expression model"), or
  delete if they only exist to reference Wolfram/parity.
- **Identifiers** (module / fn / var / test names) containing the terms → rename to neutral
  project-domain names (keep compiling; internal only). Example: `mma_hol_*` test →
  remove or rename to a project-behavior test; `wk_*` → neutral.
- **String literals** surfaced to users (messages, `language_info`, banners): confirm they are
  rum-branded / neutral (the kernel identity is already `rum`/`text`/`text/x-rum`; the
  `mathematica`/`mma`/`.wl` strings in `crates/rum-jupyter/src/transport/json/parse.rs` are
  TEST FIXTURES — change them to rum-neutral values).
- Do NOT weaken security/behavior; this is a naming/wording scrub only.

## Part C — no-leak verification

Confirm NO blacklisted asset is present in `release-staging/`: no `rum-difftest`, no `.claude`,
no dev docs (`AGENTS.md`/`CLAUDE.md`/`PROGRESS.md`/`TODO.md`/`docs/dev`/`docs/*-plan`/`*-audit`/
support-matrix/`wolfram-builtin-inventory`), no `scripts/`, no `jupyter_home`, no `WLBuiltin`,
no `*.bundle`, no `ISSUE-*.md`. (These were excluded during assembly; verify none slipped in.)

## Verification (paste verbatim)

```
cd release-staging
cargo fmt --check
cargo build --workspace
cargo test --workspace          # green; note removed tests
# final scan MUST be empty (except any deliberate nominative mention in README/DISCLAIMER,
# which are placeholders — flag those, do not count as failures):
grep -rniE 'wolfram|mathematica|wolframkernel|wolframscript|mathkernel|mma[-_]?hol|parity|oracle|difftest|byte-identical|WK-|drop-in|bug-for-bug' release-staging \
  --include='*.rs' --include='*.toml' | grep -viE 'README|DISCLAIMER'
```

## Report format

1. Part A — tests made green: which tests were REMOVED (WK/mma-hol/parity-dependent) vs
   KEPT-and-reworded; the final `cargo test --workspace` count.
2. Part B — the scrub: files touched, comment rewrites, identifier renames, the jupyter
   fixture fix; confirm no public API/crate-name change (or list any + why it was safe).
3. Part C — no-leak confirmation.
4. Verification — fmt/build/test verbatim + the final grep output (should be empty over
   `*.rs`/`*.toml`).
5. Anything you were unsure whether to scrub vs keep (flag for the director).
