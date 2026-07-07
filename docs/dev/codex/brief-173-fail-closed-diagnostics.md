# Brief 173 — Unit-2 fixes 4: fail-closed diagnostics

Implementation brief. Unit-2 fix batch 4 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-2 triage item 5; evidence
E5 in `docs/dev/usability-pass/structure-functional-findings.md`).
Four builtins that silently produce a wrong-ish or inert result on
malformed input get a message. The hard part is the SEAM between "real
error" (message) and "genuinely symbolic, stay inert" (unchanged) —
each item states its exact seam. No behavior changes on any input that
succeeds today; no message on any input that is legitimately symbolic.

## Changes

1. **`Inner` / `Dot` dimension-mismatch message**. In
   `crates/rum-stdlib/src/functional_ops/arrays/tensor.rs`,
   `compute_inner` returns `None` at several points; ONLY ONE is a real
   error: `left_contract_dim != contract_dim` (both operands are
   same-head, well-formed RECTANGULAR tensors whose contraction
   dimensions disagree). At that specific point, emit a message
   (`Inner::dim` / for the `Dot` caller `Dot::dot`, or the existing WL
   tags if you find them — report which) and still return the inert
   call. Every OTHER `None` path (not both Call heads, different heads,
   `tensor_shape` returns `None` for ragged/symbolic input) MUST stay
   inert with NO message — those are deliberately symbolic. The
   message must carry the two conflicting dimensions.
   - `compute_inner` is shared by `Inner` and `Dot`; thread the caller
     identity (or the symbol name) so the message names the right
     builtin. `Dot`'s pairwise fold means the message names the pair
     that failed.
2. **`MapAt` missing-target message** (WL parity). At
   `crates/rum-stdlib/src/functional_ops/targeted_map.rs`, when an
   EXPLICIT integer/position target cannot be resolved (out of range),
   emit a `MapAt::partw`-style message (WL uses `partw`; confirm and
   report) and keep returning the input unchanged. `{All, k}`-style
   specs and successful targets are unchanged. Do NOT message for a
   position that legitimately selects nothing under a level/`All`
   pattern where WL also stays silent — only explicit out-of-range
   integer parts.
3. **`SubsetMap` incompatible/missing path message**. Same file: when
   the path set has duplicates, overlaps, or an exact path that does
   not resolve, emit a targeted message and return the input (today it
   silently falls back — E5). Successful disjoint-path replacement is
   byte-identical. Document the message tag chosen.

Message hygiene: no new message on any input that evaluates to a
result today; no message on genuinely symbolic inert cases. Update
`docgen.rs` LIMITS notes where they describe these as silent.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# error cases: message + inert/unchanged return
printf '' | cargo run -q -p rum-cli -- --eval '{Inner[f, {a, b}, {{c, d}}, g], Dot[{a, b}, {c}], MapAt[f, {a, b}, 3], SubsetMap[Reverse, {a, b}, {{1}, {1}}]}'
# genuinely symbolic / successful cases: NO message, unchanged
printf '' | cargo run -q -p rum-cli -- --eval '{Dot[x, y], Inner[f, p, q, g], Dot[{a, b}, {c, d}], Inner[f, {a, b}, {c, d}, g], MapAt[f, {a, b}, 1], MapAt[f, {a, b}, {All}], SubsetMap[Reverse, {a, b, c}, {{1}, {3}}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 emits a
dimension/target/path message for EACH of the four and returns the
inert/unchanged value (paste both the messages and the values); probe
2 emits NO messages — `Dot[x,y]`/`Inner[f,p,q,g]` stay symbolic,
`Dot[{a,b},{c,d}]` = `a c + b d`, the MapAt/SubsetMap successes apply
(paste all).

## Scope

- MAY modify: `crates/rum-stdlib/src/functional_ops/arrays/**`,
  `crates/rum-stdlib/src/functional_ops/targeted_map.rs`,
  `crates/rum-stdlib/src/catalog/functional.rs` (descriptions),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: the operator-form matcher, other catalogs, the
  guide, `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior change is EXACTLY: a message on the specific malformed
  cases named above; the RETURN value on those cases is unchanged
  (inert call / unchanged input). Every other input — especially
  genuinely symbolic Inner/Dot and every currently-succeeding call —
  byte-identical, message-free.
- Minimal diff. Stop-loss: same failure twice → STOP and report. If
  the symbolic-vs-error seam for any item is not crisp in the code,
  STOP and report rather than risk messaging a legitimate symbolic
  expression.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; the exact seam and message tag for each of
   the four; confirmation that all other None/fallback paths stay
   silent.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
