# Brief 276 — Flat+Orderless+Optional respec + wrapped-singleton candidates

Implementation brief with a mandatory documentation part (A) and a
best-effort implementation part (B) that has its own internal
stop-loss. Closes the matcher OneIdentity/Optional + Flat respec line
(TODO.md §ACTIVE PLAN, user-approved plan items 1, 3, 4) opened
alongside brief-275 (already landed: OneIdentity promoted-singleton
matching + `Default[Plus]`/`Default[Times]`).

## Why this brief exists (read before touching code)

Two tests in
`crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`
are `#[ignore]`'d with expected output that is RECORDED THIRD-PARTY
ENGINE OUTPUT from rum's difftest era (a Wolfram Kernel build), down to
byte-exact enumeration order and duplicate rows. Under the current
Testing Policy (`AGENTS.md`) this is not just stale — it is not
legally re-derivable (no oracle comparisons, no committed third-party
engine output). This brief's Part A REMOVES that fossil and replaces
it with rum's own documented contract. Part B implements one genuine,
previously-undocumented gap the fossil was pointing at — a real
architectural hole, not a WK-order mismatch — with a firm size limit
and an explicit fallback.

**Ground truth (verified 2026-07-20 on release rum-cli @ 6ccaf2e, i.e.
AFTER brief-275 landed):**

```
SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1,2], g[a_, b_:0]]              → True
SetAttributes[g, {Flat, Orderless}]; g[1,2] /. g[a_, b_:0] :> {a, b}         → List[2, 1]
SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b_:0] :> {a,b}] → List[List[2, 1], List[1, 2], List[g[1, 2], 0]]
SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1], g[a_, b_:0]]               → True
SetAttributes[g, {Flat, Orderless}]; g[1] /. g[a_, b_:0] :> {a, b}          → List[1, 0]
SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1], g[a_, b_:0] :> {a,b}] → List[List[1, 0]]
SetAttributes[g, {Flat, Orderless}]; MatchQ[g[], g[a_:9, b_:0]]              → True
SetAttributes[g, {Flat, Orderless}]; g[] /. g[a_:9, b_:0] :> {a, b}         → List[9, 0]
```

**Grounding of the real gap (code-read 2026-07-20):** for a Flat head
WITHOUT `OneIdentity`, when the matcher groups exactly one value for a
plain `Blank` pattern-argument, `flat_match_value` (line ~420 of
`pattern_matching/sequence.rs`) returns the BARE element
(`[single] => single.clone()`) — it never also offers the
HEAD-WRAPPED form (`g[1]` as distinct from `1`) as a second candidate.
The same short-circuit exists in the orderless-flat sibling,
`match_compiled_orderless_flat_selected_values_owned`'s `[value]` arm
(`pattern_matching/sequence/orderless/selected.rs` line ~51-59). No
system builtin in rum is currently Flat-without-OneIdentity (`Plus`,
`Times`, `Union`, `Intersection` all carry `OneIdentity`; `And`/`Or`
are not registered `Flat` at all in rum) — this gap is reachable ONLY
by a user-declared `SetAttributes[myHead, {Flat, Orderless}]` without
`OneIdentity`. Confirmed zero hot-path exposure for any built-in.

## Part A — respec (MANDATORY, do this regardless of Part B's outcome)

1. In `pattern_matching_orderless_optional.rs`:
   - Rename the helper `assert_wk_cases` → `assert_rum_cases` and its
     doc/panic text from "WK-calibrated"/"diverged" framing to "rum's
     documented contract" framing (mechanical rename, `replace_all`
     style — update every call site in the file).
   - Remove the two `#[ignore = "KNOWN WK DIVERGENCE ..."]` tests
     (`flat_orderless_optional_matches_collects_and_omits_like_wk`,
     `flat_orderless_optional_with_sequence_neighbors_uses_wk_backtracking_order`)
     ENTIRELY — do not leave them ignored, delete them.
   - Add THREE new, non-ignored tests in their place, named for what
     they assert (not for WK conformance), asserting rum's OWN
     contract:
     - `flat_orderless_optional_matches_collects_and_omits` — the
       MatchQ/`/.`/`ReplaceList` cases from the Ground Truth block
       above for `g[1,2]`, `g[1]`, `g[]`, run BEFORE Part B (i.e.
       write this test FIRST against the pre-existing behavior, using
       the exact Ground Truth values as the expected output) — if Part
       B lands, UPDATE this test's `ReplaceList` expectations to
       include the new wrapped rows (ordering per Part B's contract
       below); if Part B does not land, this test stays exactly as
       written against the Ground Truth.
     - `flat_orderless_optional_with_sequence_neighbors` — run the
       source expressions from the deleted
       `..._sequence_neighbors_uses_wk_backtracking_order` test
       (`g[1,2] /. g[a_, b___, c_:0] :> {a, {b}, c}` and its
       `ReplaceList` form) against WHATEVER rum's actual current
       output is (probe it yourself, do not guess) and pin THAT as the
       expected value. This is the compound "their compound" gap from
       the debt note — it is explicitly NOT required to change in this
       brief; pin current truth, do not chase WK's order.
     - `flat_orderless_optional_nonmatch_edges_stay_false` may be
       skipped as a separate test if the existing
       `orderless_optional_nonmatch_edges_stay_false` already covers
       the Flat non-match edge (check before adding a duplicate).
2. Write `docs/dev/matcher-flat-optional-contract.md` (new file,
   following the shape of `docs/dev/structured-exceptions-contract.md`
   as a size/tone reference). Contents:
   - One paragraph: what this documents (rum's own Flat+Orderless+
     Optional candidate contract) and WHY it exists (the WK-derived
     spec was retired as a policy fossil; link this brief number and
     date).
   - The candidate contract as stated in Part B's Specification below
     (or, if Part B did not land, state that plainly: "the
     head-wrapped single-element candidate described below is NOT
     currently implemented; only the bare form is tried" — and still
     record the intended contract for a future attempt).
   - Explicitly record: rum does not target WK's enumeration ORDER;
     only rum's own internal determinism is guaranteed (documented
     WON'T-FIX on order conformance).
3. Do NOT touch `TODO.md` — it is out of Scope (see below) even though
   this line traces back to a `TODO.md` debt entry; Claude updates that
   file at the gate.

## Part B — wrapped-singleton candidates (best-effort, has its own stop-loss)

### Specification (the target contract, IF implemented)

For a `Flat`-attributed pattern head WITHOUT `OneIdentity`: when a
plain `Blank`-style pattern-argument (`MatcherArg::Single`) is matched
against a grouped selection of exactly ONE value from the flat
argument list, rum tries TWO candidate values for that slot, in this
order:

1. the BARE element (today's behavior — unchanged, preserves every
   existing regression pin as the first-tried candidate);
2. the element re-wrapped in the flat head,
   `Expr::call(flat_head, vec![bare_element])` (e.g. `g[1]`) — as an
   ADDITIONAL candidate, tried/collected immediately after the bare
   one, before any longer-consume-length grouping is tried.

This applies symmetrically to:
- the plain (non-orderless) Flat single-choice loop,
- the Orderless+Flat single-choice generation.

`MatchQ`/`/.` (first-match semantics): bare-first ordering means NO
currently-passing test changes its result — the wrapped form is only
ever reached when bare fails to complete a full match (including the
REST of the pattern).

`ReplaceList` (collect-all semantics): both the bare-success row and
the wrapped-success row (when both succeed) appear as separate output
rows, wrapped immediately following bare, before the next length is
tried.

### Implementation guidance (follow this shape; do not improvise a
### differently-scoped design)

- **Orderless+Flat path** (`sequence/orderless/`): thread a
  `one_identity: bool` parameter into `orderless_candidate_choices`
  (`candidates.rs`) and its private `orderless_partition_choices_by_lengths`
  helper, ALONGSIDE the existing `flat_head: ` — wait, `flat_head` is
  not currently passed to `orderless_candidate_choices` either; add it
  too, but ONLY reachable via the `allow_flat_single` branch (the
  non-flat call sites pass irrelevant/unused values — check whether an
  `Option<&Expr>` is cleaner than a plumbed-but-unused param, and
  prefer whichever keeps the non-flat call sites untouched in spirit).
  In the `MatcherArg::Single(_) if allow_flat_single` arm: after
  computing the length-1..=max_len partitions as today, for every
  choice whose `selected_values` has length exactly 1, when
  `!one_identity`, insert an ADDITIONAL `OrderlessCandidateChoice`
  immediately after it in the returned `Vec` — identical `rest_values`,
  but `selected_values = vec![Expr::call(flat_head.clone(),
  vec![original_value])]`. Because `match_compiled_orderless_flat_selected_values_owned`'s
  `[value]` arm already does a plain `match_compiled_owned(pattern,
  value.clone(), bindings)` with no awareness of "bare vs wrapped", NO
  CHANGE is needed in `selected.rs` — the pre-wrapped `Expr` flows
  through unmodified. This keeps the actual matching logic untouched;
  the entire feature is a candidate-GENERATION change.
  `one_identity` is computed once, by the caller of
  `match_compiled_orderless_args_with_flat_head_owned`
  (`pattern_matching/core.rs`, where it is ALREADY computed as a local
  in `match_compiled_call_owned`) — thread it down through
  `match_compiled_orderless_args_with_flat_head_owned` →
  `match_compiled_orderless_flat_arg_choices_owned` →
  `orderless_candidate_choices`. This is a MECHANICAL signature-ripple
  (grep every call site of these three functions before starting; the
  collect-side siblings need the same threading).
- **Plain Flat path** (`sequence.rs`): in
  `match_compiled_flat_single_choices_owned` and its collect sibling
  `collect_compiled_flat_single_choice_matches_owned`, do NOT thread a
  new parameter (these two already carry `flat_head: Expr`, and
  threading `one_identity` through their own recursive self-calls is
  more invasive than the orderless case's single generation point).
  Instead, compute `one_identity` LOCALLY inside each function via a
  direct attribute check on `flat_head` (mirroring the check
  `match_compiled_call_owned` already does — `flat_head` is always a
  literal system/user symbol `Expr` at these call sites per brief-275's
  grounding): `matches!(&flat_head, Expr::Atom(Atom::Symbol(s)) if
  self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity))`.
  At `consume == 1`: after the existing bare-value pattern match
  (match fn) / after collecting the bare-value's matches (collect fn),
  if `!one_identity`, ALSO try the wrapped value
  (`Expr::call(flat_head.clone(), vec![selected_values[0].clone()])`)
  through the identical continuation shape (match: only if bare's
  attempt — INCLUDING its rest-match — did not fully succeed; collect:
  unconditionally, appending any results after the bare results), then
  proceed to `consume + 1` as today. Keep this as a small, local
  addition at the `consume == 1` branch point — do not restructure the
  `consume` loop's general shape.
- Do NOT touch `flat_match_value` itself (it is correct and used by
  other call sites for consume > 1, which is unaffected by this
  brief).

### Stop-loss (read this before starting Part B; it is generous on
### purpose)

Run the ENTIRE existing (non-deleted, non-ignored) test suite in
`pattern_matching_orderless_optional.rs` PLUS every other
`pattern_matching_*`-named test file in
`crates/rum-runtime/src/runtime/tests/` after each incremental step.
If:
- the same implementation step fails twice for the same reason, OR
- ANY previously-passing test (in ANY file, not just this one) changes
  its output, OR
- the threading described above turns out to require touching more
  than the files listed in Scope,

then STOP Part B entirely: `git checkout --` (or manually revert) every
Part-B-only source change (keep Part A's test/doc changes), and in
Part A's third new test
(`flat_orderless_optional_matches_collects_and_omits`), pin the Ground
Truth values from this brief's header (i.e., no wrapped candidates) as
final. Report the stop-loss clearly — this is a fully acceptable
outcome, not a failure to explain away.

If Part B succeeds, update `matcher-flat-optional-contract.md` and the
`flat_orderless_optional_matches_collects_and_omits` test to the ACHIEVED
`ReplaceList` rows (verify by running the release CLI probe yourself —
do not hand-derive the expected wrapped-row order, read it from actual
output).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`;
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence.rs`
    (Part B only);
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs`,
    `entry.rs`, `selected.rs`, `collect.rs`, `match_choices.rs` (Part B
    only, signature-threading as described);
  - `crates/rum-runtime/src/runtime/pattern_matching/core.rs` (Part B
    only, MECHANICAL: pass the already-computed `one_identity` local
    into the orderless-flat call — do not otherwise change this file;
    it was the subject of brief-275 and must not regress that work);
  - `docs/dev/matcher-flat-optional-contract.md` (NEW).
- Files/areas you MUST NOT touch: `TODO.md`, `PROGRESS.md`,
  `AGENTS.md`, any other `docs/**`, anything under `docs/dev/codex/`
  except this file's own report, `.git`, `Cargo.lock`, any stdlib
  crate, `rum-host`. If you believe you must, STOP and report.
- Do NOT add unconditional cost to the default evaluation path — in
  particular, no OneIdentity-attributed head (Plus, Times, Union,
  Intersection, or any user OneIdentity head) may see ANY new work;
  the `!one_identity` gate must short-circuit before any allocation.

## Deliverable

- Part A: renamed/reframed test helper, two fossil tests replaced by
  three (or two, per the dedup check) truth-pinned tests, the new
  contract doc.
- Part B (if not stop-lossed): the two candidate-generation additions,
  updated `ReplaceList` expectations in the Part-A test reflecting
  achieved output, updated contract doc.
- The report per Report format below, which MUST state explicitly
  whether Part B landed or was stop-lossed, and why.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace        # known caveat: loopback-bind ZeroMQ tests may fail under the sandbox; if so, run and paste the two supplemental splits (--workspace --exclude rum-jupyter; -p rum-jupyter --lib -- --skip zeromq::tests::native) and say so
cargo test -p rum-runtime pattern_matching -- --include-ignored     # confirm zero #[ignore] remain in the touched file and nothing else regressed
```

If Part B lands (matcher source touched), also run the microbench A/B:
build base BEFORE any Part-B source edit (copy to
`/tmp/brief276_base_rum-cli`), build candidate after; one
`scripts/core-language-microbench.sh` run per binary; paste
`matchq-*`/`downvalue-pattern-*`/`replaceall-simple` rows and same-tier
neighbors, state direction mix. Given the confirmed zero built-in
exposure, a flat/no-movement result on these labels is the expected
(and required) outcome — any movement on them needs an explanation
before you call it noise.

Then paste the verbatim output of these probes on the CANDIDATE release
build (stdin from /dev/null):

```
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1,2], g[a_, b_:0]]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; g[1,2] /. g[a_, b_:0] :> {a, b}'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b_:0] :> {a, b}]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1], g[a_, b_:0]]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; g[1] /. g[a_, b_:0] :> {a, b}'
target/release/rum-cli --eval '1+1'   # regression: OneIdentity path untouched by any Part-B change
target/release/rum-cli --eval 'MatchQ[5, x_ + y_.]'   # regression: brief-275 behavior intact
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all
  changes in the working tree; Claude runs the authoritative gate and
  commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- Touch NOTHING outside this repo (except `/tmp/brief276_base_rum-cli`
  and microbench output dirs under `/tmp`), no network.
- **Testing Policy is a hard line** (`AGENTS.md`): no black-box
  comparisons against other vendors' engines, no oracle harnesses, no
  third-party engine outputs committed. This brief's whole point is
  REMOVING a violation of this policy — do not reintroduce one by
  consulting WK/Mathematica output to derive Part B's expected values;
  derive everything from rum's own actual behavior.
- **No new `unsafe`.**
- **Hot-path changes require the microbench A/B gate** if Part B
  touches matcher source.
- Minimal diff: no drive-by reformatting, renames, or comment changes
  outside the task.
- You MUST paste the real verbatim verification output. If a run did
  not reach green / produce the evidence, say so explicitly — never
  fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why. State
   clearly: Part A landed (always); Part B landed / was stop-lossed
   (which, and why).
2. Verification — each command run, with the exact verbatim output
   lines (test totals incl. the `--include-ignored` zero-ignored
   check, A/B rows if Part B landed, probe outputs).
3. The finding / verdict — the achieved contract vs the target
   contract, any deviation.
4. Stop-loss / open questions — if Part B was stop-lossed, the exact
   trigger condition that fired; otherwise empty.
