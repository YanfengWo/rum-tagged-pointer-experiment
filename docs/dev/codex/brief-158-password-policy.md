# Brief 158 — RandomPassword policy options (require classes, no adjacent repeats, exclude substrings)

Implementation brief. Extends `RandomPassword` (from brief-157) with the
real-world policy constraints. Method: whole-password REJECTION SAMPLING —
generate a full unbiased password from the entropy source, accept iff it
satisfies every constraint, else resample; bounded attempt cap with a
fail-closed message when constraints cannot be met. Rejection over the
constrained set stays exactly uniform (no bias). `Capability::Random`,
entropy-backed, unaffected by the session PRNG (same as brief-157).

## Option surface (rum option-rules; strings, not magic class names)

`RandomPassword[n, opts…]` and `RandomPassword[alphabet_String, n, opts…]`
accept:

- `"Require" -> {group_String, …}` — each group is a set of characters
  (as a string); the result must contain AT LEAST ONE character from each
  group. Groups need not be disjoint. Each group must be a subset of the
  alphabet (pre-validate: a group with a char outside the alphabet →
  fail-closed `::password` message naming the offending group).
- `"NoAdjacentRepeats" -> True` (default `False`) — forbids two identical
  characters in adjacent positions. (Only identical-adjacent; sequential
  runs like `abc` are explicitly NOT in scope — documented.)
- `"Exclude" -> {substr_String, …}` — the result must not contain any of
  these substrings. Empty strings in the list → fail-closed.
- `"IgnoreCase" -> True` (default `False`) — makes `"Exclude"` matching
  case-insensitive (does not affect `"Require"`, which is exact-set
  membership — document).

No options → identical to brief-157 `RandomPassword` (backward compatible;
the no-constraint path must not pay the rejection loop).

## Feasibility pre-checks (fail-closed BEFORE sampling, cheap)

- number of `"Require"` groups > `n` when the groups are pairwise disjoint
  in the strictest sense — actually: if the count of required groups > n,
  it is unsatisfiable (each group needs a distinct position at minimum only
  if disjoint; to stay simple and correct, only reject when
  `Length[Require] > n` AND report it, since even overlapping groups can't
  place more than n required hits... keep it: reject when a REQUIRED GROUP
  IS EMPTY, when any group is not a subset of the alphabet, when an
  `"Exclude"` entry is empty, or when `n < 1`). Do NOT try to prove general
  infeasibility — that is what the attempt cap is for.
- Attempt cap: a fixed bound (suggest 10_000). On exhaustion →
  `RandomPassword::password` message ("could not satisfy the password
  policy within N attempts; loosen the constraints or increase length") +
  `$Failed`. Document the cap in the reference limits note.

## Uniformity note (document in the reference)

Whole-password rejection sampling draws each candidate uniformly from
`alphabet^n` and accepts iff it satisfies the policy, so the output is
uniform over the policy-satisfying set — no positional bias from "placing
required characters". State this; it is the reason the method was chosen
over constructive placement.

## Tests

- `"Require"`: single and multiple groups; result always contains ≥1 from
  each (assert over many draws with a fixed-bytes test EntropySource that
  is engineered to require a couple of resamples — pin the exact accepted
  output); group-not-subset-of-alphabet → fail-closed; empty group →
  fail-closed.
- `"NoAdjacentRepeats"`: assert no adjacent equal chars over many draws;
  a fixed-bytes source that would produce an adjacent repeat forces a
  resample to a pinned result.
- `"Exclude"`: forbidden substring never appears; `"IgnoreCase"` makes a
  differently-cased forbidden substring also excluded; pinned resample
  case; empty exclude entry → fail-closed.
- Combined policy (Require + NoAdjacentRepeats + Exclude) end-to-end with a
  crafted entropy stream → exact expected password.
- Cap exhaustion: an impossible-in-practice policy (e.g. alphabet `"ab"`,
  `"NoAdjacentRepeats" -> True`, `n = 20` is satisfiable — instead use
  alphabet `"a"`, `"NoAdjacentRepeats" -> True`, `n = 2`: impossible) →
  `::password` message + `$Failed` within the cap.
- Capability/entropy contracts unchanged (denied under sandboxed(),
  noentropy without a provider) still hold WITH options.
- No-options path == brief-157 behavior (a determinism/entropy pin
  unchanged).
- Vocabulary/catalog registration unchanged (still `RandomPassword`, one
  builtin, more arities); reference regen (count unchanged at 600 — options
  do not add a builtin; confirm the entry text updates and drift is green).

## Scope

- MAY modify: the stdlib random family files (RandomPassword impl +
  option parsing) + tests, rum-host policy tests if extended,
  `docs/reference/` via generator, the randomness guide section
  (`docs/guide/design-boundaries.md` or wherever brief-140/157 put it) —
  add the policy options with real transcripts + the uniformity note.
- MUST NOT touch: the PRNG core, EntropySource seam, `RandomString`,
  evaluator/dispatch, `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 600 (unchanged; options only)
printf '' | cargo run -q -p rum-cli -- --eval 'p = RandomPassword[16, "Require" -> {"ABCDEFGHIJKLMNOPQRSTUVWXYZ", "0123456789", "!@#%"}, "NoAdjacentRepeats" -> True, "Exclude" -> {"alice"}]; {StringLength[p], p}'
printf '' | cargo run -q -p rum-cli -- --eval 'RandomPassword["a", 2, "NoAdjacentRepeats" -> True]'   # fail-closed (impossible)
printf '' | cargo run -q -p rum-cli -- --eval 'RandomPassword[8, "Require" -> {"xyz"}]'                # fail-closed if xyz not subset of default alphabet? (xyz ARE in A-Za-z0-9 → ok; pick a real out-of-alphabet probe)
printf '' | cargo run -q -p rum-cli -- --eval 'RandomPassword["abc", 8, "Require" -> {"xyz"}]'         # fail-closed: xyz not subset of "abc"
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Whole-password rejection sampling only — no constructive placement (bias).
- No-options path must be zero-overhead relative to brief-157.
- Fail-closed on infeasible/malformed policy; cap-exhaustion is a clear
  message, not a hang.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. The option contract as landed + the feasibility-precheck and cap
   decisions.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
