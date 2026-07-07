# Brief 182 — Unit-4 fixes 1: rewrite-family operator forms

Implementation brief. Unit-4 fix batch 1 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-4 triage item 1; evidence
E6 in `docs/dev/usability-pass/control-logic-rewrite-findings.md`).
This is the operator-form spillover recorded across units 2-3, now
addressable. Extends the EXISTING callable-wrapper mechanism exactly as
brief-175/179 did — same chain in
`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`, no new
dispatch path. These are KERNEL builtins (rum-runtime), so the arity
exemptions live in the kernel rewrite paths. Touches call planning →
microbench A/B gate applies.

## The symbols and their operator arities

Disambiguation rule (as in 175/179, stated in the matcher code): an
arity becomes an operator form ONLY if it cannot be a valid data-form
call. All five here are 1-arg operator forms; their data forms are all
≥2-arg (verified E6/E7), so 1 argument is unambiguous:

- `Cases[patt]` — data form `Cases[expr, patt, …]` (2-5 args).
- `DeleteCases[patt]` — data form `DeleteCases[expr, patt, …]` (2-5).
- `Replace[rules]` — data form `Replace[expr, rules(, levels)]` (2-3).
- `ReplaceAll[rules]` — data form `ReplaceAll[expr, rules]` (2).
- `ReplaceRepeated[rules]` — data form `ReplaceRepeated[expr, rules]`
  (2).

`ReplaceList` ALREADY has its operator form (it is the working
contrast case — mirror its wiring;
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs`).
`MatchQ[patt]` already landed in brief-179.

Semantics: `Foo[params][data]` ≡ `Foo[data, params]`. The
operator-arity direct call is inert and message-free (today these emit
`argtu`/`argx` — those messages disappearing for the 1-arg arity is the
intended change). Application rewrites to the ordinary call; validation
happens there. For `Cases`/`DeleteCases`, options/levelspecs that
follow in the DATA form still work — the operator form only fixes the
1-arg case (`Cases[patt, opts][expr]` is out of scope here unless
`ReplaceList`'s pattern already generalizes to it; if trivial include
it, else defer and say so).

## Mechanism

Extend the matcher chain (the brief-179 pattern): add a rewrite-operator
recognizer with the five kernel symbols (all 1-arg), the apply-rewrite,
and the argx arbitration. Exempt the 1-arg arity from each builtin's
kernel arity message (the brief-179 `MatchQ` change shape, applied to
`cases/collect.rs`, `cases/delete.rs`, `replace/once.rs`,
`replace/all.rs`, `replace/repeated.rs`).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Cases[_Integer][{1, a, 2}], DeleteCases[_Integer][{1, a, 2}], Replace[x_ :> f[x]][a], ReplaceAll[x_ :> f[x]][a], ReplaceRepeated[x_ :> f[x]][a], ReplaceList[x_ :> f[x]][a]}'
# inert message-free operator forms + ordinary calls byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{Cases[_Integer], Replace[x_ :> y], Cases[{1, a, 2}, _Integer], Replace[a, x_ :> f[x]], ReplaceAll[a + b, x_ :> f[x]], ReplaceRepeated[a, {a -> b, b -> c}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 →
`{{1, 2}, {a}, f[a], f[a], f[a], {f[a]}}`; probe 2 shows the two inert
operator forms with NO messages, then the ordinary calls byte-identical
(`{1, 2}`, `f[a]`, `f[a]+f[b]` or as today, `c`) — paste all.

## Microbench A/B (MANDATORY — the perf gate is active)

The matcher chain gains one recognizer arm. Run the alternating
protocol: build release from the CURRENT tree (candidate) and from
`HEAD` (base, `git stash`; restore after — `git stash list` empty at
the end); save both binaries; run
`scripts/core-language-microbench.sh --rum-cli <saved>` alternating
base/candidate ≥2 pairs each; report per-label best-of deltas and each
side's self-noise band; flag any label whose |delta| exceeds its noise
band; state whether the arm is consulted on symbol-head calls or only
compound-head calls (the brief-175 early exit should keep it
compound-head only).

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  (matcher arm), the arity paths of the five in
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/**`,
  `crates/rum-stdlib/src/catalog/rewrite.rs` (descriptions),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  kernel rewrite/wrapper tests under
  `crates/rum-runtime/src/runtime/tests/**`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: `Exit`/`Implies`/`Equivalent` (separate decisions),
  `ReplaceList`/`MatchQ` behavior, other families, the guide,
  `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push (stash restored at the end). No
  network, no new dependencies, no new `unsafe`.
- Behavior changes are EXACTLY: (a) the five 1-arg arities become inert
  message-free operator forms, (b) applying them rewrites to the
  ordinary call. Everything else byte-identical.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; confirm the five arity exemptions and
   whether the `Cases[patt, opts]` multi-arg operator form was included
   or deferred.
2. Verification — all commands verbatim.
3. Microbench A/B — worst rows, noise bands, consultation-path
   statement.
4. Stop-loss / open questions (empty if none).
