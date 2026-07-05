# Brief 129 — Hand-written language guide (`docs/guide/`)

Implementation brief. Sixth step of `docs/dev/release-transformation-plan.md`.
The guide is the hand-written half of the documentation-as-contract model;
the generated `docs/reference/builtins.md` (brief-128) is the other half.

## Goal

`docs/guide/` exists and teaches a new user (and a host developer) the `rum`
language and embedding story from zero, grounded exclusively in `rum`'s own
behavior: **every code example in the guide was actually run against the
current build, and the shown output is the real output.** The guide describes
`rum` on its own terms — no comparisons to, or terminology borrowed as
authority from, other vendors' products (nominative context lives in
`DISCLAIMER.md` only).

## Context pointers (source material — read before writing)

- `docs/runtime-evaluation-audit.md` — evaluator model, dispatch stages,
  holds, limits (current-truth; reworded in brief-127).
- `docs/lowering-canonical-forms.md` — syntax → canonical form contracts.
- `docs/host-embedding-and-hardening.md`, `docs/host-adapter-plan.md` — the
  host/embedding surface (interrupt/cancel, timeout, poison recovery,
  reset, snapshot/restore, capability sandbox).
- `docs/reference/builtins.md` — the generated per-builtin contract.
- `crates/rum-host/examples/basic_embedding.rs` (if present) and the
  `rum-host` public API for the embedding chapter.
- Run examples with: `cargo run -q -p rum-cli -- --eval '<expr>'` (and the
  REPL/script forms where relevant). Use `scripts/run-bounded.py` for
  anything that could run long.

## Scope

- MAY create: `docs/guide/README.md` plus the chapter files listed below.
- MAY modify: `README.md` — ONLY the single sentence "A language guide
  (`docs/guide/`) and a generated builtin reference (`docs/reference/`) are
  being added as part of the current documentation pass." becomes links to
  both.
- MUST NOT touch anything else: no crate code, no other docs, no scripts,
  no `Cargo.*`, no `.git`.

## Chapters (fixed TOC — keep each file focused and modest; 150-350 lines)

1. `README.md` — guide index, who it is for, how to read, pointer to the
   reference and DISCLAIMER.
2. `getting-started.md` — build/run, REPL, `--eval`, running a `.wl` script
   file, first definitions, getting help from the reference.
3. `syntax.md` — expressions and atoms (symbols, exact integers, rationals,
   strings), operators and common sugar, lists, associations, comments;
   state explicitly that decimal literals lower to exact rationals (link
   design-boundaries).
4. `evaluation.md` — the evaluation model: fixed-point evaluation, held
   arguments and hold attributes, attribute-driven behavior (Listable, Flat,
   Orderless, ...), evaluation order, messages and `Quiet`/`Check`, iteration
   and recursion limits with `TerminatedEvaluation` outcomes.
5. `patterns-and-rules.md` — blanks and named patterns, sequences,
   alternatives/conditions/pattern tests, defaults; `MatchQ`/`Cases`/
   `ReplaceAll`/`ReplaceRepeated`/`Replace`; ownvalues/downvalues/upvalues/
   subvalues and definition precedence; documented matcher limits (the
   match-depth guard; the known Flat+Orderless+Optional corners per
   `TODO.md` — state them as documented limits).
6. `scoping-and-contexts.md` — `Module`/`Block`/`With`, closures over module
   locals, contexts and `` ` `` names, `$Context`/`$ContextPath`, packages
   and `Get`/`Needs`, `DumpSave`/`Get` session persistence (textual format).
7. `resource-limits-and-sandbox.md` — the caller-set limits (recursion,
   iteration, parse depth, match depth), what trips them and the resulting
   `TerminatedEvaluation` tags; interrupt/cancel and wall-clock timeout from
   the host; the capability policy (`Pure`/`FileRead`/`FileWrite`/
   `Diagnostic`) and `HostConfig::sandboxed()`; the host-side memory
   watchdog pattern.
8. `embedding.md` — `rum-host` quick start: create a session, evaluate,
   consume structured outcomes/messages/failures, resumable input,
   `reset()`, snapshot/restore-from-bytes, sandboxed configuration; a
   compile-checked minimal example (mirror the existing `basic_embedding`
   example); pointer to rustdoc for the full API.
9. `design-boundaries.md` — deliberate limits stated plainly: exact
   rationals only (no floating-point type; decimals lower to rationals);
   "good-enough" scope for several functions (cite the reference's limits
   notes); textual `DumpSave`; the deep-`Expr`-via-Rust-API drop limitation;
   no implementation of other vendors' protocols or file formats.

## Style rules

- Plain technical prose; no marketing voice; second person sparingly.
- Every example is an input/output pair that was ACTUALLY RUN; outputs are
  pasted verbatim (including message lines). If behavior surprises you,
  document what rum actually does — do not write aspirational output. If it
  looks like a bug, note it in the report instead of documenting around it.
- Use only rum-native terminology; name builtins exactly as in the
  reference; link chapters to reference entries where helpful (relative
  links).
- No external-engine names anywhere in `docs/guide/` (the rg gate below).

## Verification (MANDATORY — run all, paste verbatim)

```
ls docs/guide/
wc -l docs/guide/*.md
rg -in 'wolfram|mathematica|wstp|mma[-_]hol' docs/guide/ || echo GUIDE-CLEAN
# Example-accuracy evidence: for each chapter, paste 2 representative
# examples together with the actual rum-cli invocation + verbatim output
# that produced them (a transcript block per chapter, 3-9 in total... one per chapter with 2 examples each).
cargo run -q -p rum-cli -- --eval 'Total[{1, 2, 1/2}]'   # sanity: the toolchain you used
git status --short
```

Also state in the report the TOTAL number of examples in the guide and
confirm every one was executed (list any that could not be, with why).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Zero code changes anywhere.
- Examples must be real transcripts of the current build — fabricating or
  hand-editing an output is a hard failure.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What was written — per chapter, 1-2 lines + example count.
2. Verification — the commands above with verbatim output, incl. per-chapter
   example transcripts.
3. Anything rum did that looked wrong while writing examples (candidate
   bugs / doc-worthy surprises) — list with the exact input/output.
4. Stop-loss / open questions (empty if none).
