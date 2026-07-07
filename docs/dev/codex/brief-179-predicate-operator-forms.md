# Brief 179 — Unit-3 fixes 3: predicate operator forms

Implementation brief. Unit-3 fix batch 3 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-3 triage item 3; evidence
E1/E2 in
`docs/dev/usability-pass/predicates-associations-findings.md`).
Extends the EXISTING callable-wrapper mechanism exactly as brief-175
did for the structure symbols — same chain in
`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`, no new
dispatch path. Touches call planning → the microbench A/B gate applies.

## The symbols and their operator arities (disambiguation rule)

Same rule as brief-175, stated in the matcher code: an arity becomes an
operator form ONLY if it cannot be a valid data-form call of the same
builtin; never sniff argument types.

ONE-argument operator forms (data forms are all ≥2-arg, so 1 argument
is unambiguous):

- `ContainsAll[needles]`, `ContainsAny[needles]`,
  `ContainsExactly[needles]`, `ContainsNone[needles]`,
  `ContainsOnly[allowed]` — data form `Contains*[list, needles]`.
- `Between[range]` — data form `Between[value, range]`.
- `AllTrue[crit]`, `AnyTrue[crit]`, `NoneTrue[crit]` — data form
  `*True[list, crit]` (or `[list, crit, levelspec]`).
- `MatchQ[patt]` — data form `MatchQ[expr, patt]`. NOTE: `MatchQ` is a
  KERNEL builtin (rum-runtime), not stdlib; its arity-message exemption
  lives in the kernel `MatchQ` path. Add the matcher recognition arm
  and the exemption together.

Semantics: `Foo[params…][data]` ≡ `Foo[data, params…]`. The
operator-arity direct call is inert and message-free (today these emit
`argx`/`argtu` — those messages disappearing for exactly these arities
is the intended change). Application rewrites to the ordinary call;
validation happens there.

## EXCLUDED (do NOT add — record the reason in a code comment)

- `OrderedQ[p]`: 1-arg would collide with the data form `OrderedQ[list]`
  (brief-177's 1-arg canonical check). Disambiguation forbids it.
- `Lookup[key, default]`: 2-arg would collide with the data form
  `Lookup[assoc, key]`. The existing 1-arg `Lookup[key]` operator form
  stays; a 2-arg operator form would require first-argument
  type-sniffing. Excluded.

## Mechanism

Extend the matcher chain (the brief-175 `is_structure_operator_wrapper`
pattern): add a predicate-operator recognizer with the ten symbols and
their operator arities (all 1-arg here), the apply-rewrite, and the
argx arbitration. For `MatchQ`, exempt the 1-arg arity from the kernel
`MatchQ` arity message so its inert operator form is message-free.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{ContainsAll[{a}][{a, b}], ContainsAny[{z}][{a, b}], ContainsExactly[{a, b}][{b, a}], ContainsNone[{z}][{a, b}], ContainsOnly[{a, b}][{a}], Between[{1, 3}][2], AllTrue[EvenQ][{2, 4}], AnyTrue[OddQ][{2, 3}], NoneTrue[OddQ][{2, 4}], MatchQ[_Integer][3]}'
# inert message-free operator forms + ordinary calls byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{ContainsAll[{a}], Between[{1, 3}], MatchQ[_Integer], ContainsAll[{a, b}, {a}], Between[2, {1, 3}], MatchQ[3, _Integer], OrderedQ[{1, 2, 3}], Lookup[<|"a" -> 1|>, "a"]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 all
evaluate to the ordinary-call results (`True, False, True, True, True,
True, True, True, True, True`); probe 2 shows the three inert
operator forms with NO messages, then the ordinary calls
byte-identical (`True, True, True, True, 1`) — `OrderedQ[{1,2,3}]` and
`Lookup[…]` prove the excluded symbols still behave as data forms.

## Microbench A/B (MANDATORY — the perf gate is active)

The matcher chain gains one recognizer arm. Run the alternating
protocol: build release from the CURRENT tree (candidate) and from
`HEAD` (base, `git stash`; restore after — `git stash list` empty at
the end); save both binaries; run
`scripts/core-language-microbench.sh --rum-cli <saved>` alternating
base/candidate ≥2 pairs each; report per-label best-of deltas and each
side's self-noise band; flag any label whose |delta| exceeds its noise
band; state whether the arm is consulted on symbol-head calls or only
compound-head calls (brief-175's early exit should keep it
compound-head only).

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  (matcher arm), the kernel `MatchQ` arity path (wherever it lives in
  rum-runtime), the arity-message paths of the stdlib predicates in
  `crates/rum-stdlib/src/predicates/**`,
  `crates/rum-stdlib/src/catalog/predicates.rs` (descriptions),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  kernel wrapper/MatchQ tests under
  `crates/rum-runtime/src/runtime/tests/**`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: `OrderedQ`/`Lookup` behavior (excluded), other
  families, the guide, `TODO.md`, the tracker, findings files,
  `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push (stash restored at the end). No
  network, no new dependencies, no new `unsafe`.
- Behavior changes are EXACTLY: (a) the ten listed operator arities
  become inert message-free forms, (b) applying them rewrites to the
  ordinary call. Everything else byte-identical — especially the two
  excluded symbols.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; confirm the arity-exemption list matches
   the ten symbols and the two exclusions are commented.
2. Verification — all commands verbatim.
3. Microbench A/B — worst rows, noise bands, consultation-path
   statement.
4. Stop-loss / open questions (empty if none).
