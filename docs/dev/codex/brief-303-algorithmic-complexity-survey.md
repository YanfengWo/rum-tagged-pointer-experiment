# brief-303 — repository-wide survey for reducible complexity classes

## Goal

Survey the whole repository independently and produce a reviewable plan naming
every place where an algorithm's **complexity class** can be lowered — O(n²) or
worse today, with a realistic path to something better. The deliverable is a
ranked candidate list backed by `path:line` evidence, plus drafted follow-up
implementation briefs grouped into dispatchable batches.

**Scope: NONE — no source change.** Do not edit any source file. Do not run the
microbenchmark scripts. Reading, grepping, and small throwaway probes with the
release `rum-cli` are in scope; leave no probe files in the repository.

## Why this brief exists, and what "independently" means

A previous survey by a different agent produced the two remaining P1 items in
`TODO.md` (quoted below). **That report is not to be trusted as ground truth.**
Two independent failure modes are being guarded against: an agent's blind spot,
and findings that have simply aged out. So:

- Do **not** start from the previous list, and do not treat its items as
  established. Derive your own candidates from the code.
- Then, separately, **re-probe the two carried-over items** and state whether
  each still holds *today*, with current `path:line` evidence. A previous batch
  may already have changed the code under them. If an item no longer reproduces,
  say so plainly — that is a valuable result, not a failure.

The two carried-over items, quoted from `TODO.md` for re-probing only:

1. *Shared replacement plan.* `Replace` still clones or scans its rules at each
   visited node and computes subtree depth even for level specifications that do
   not need it.
2. *Three bounded wide-input scans.* Repeated prefix rescans in `RumTokens`,
   wide TOML table keys, and `Reap`'s linear tag bucketing.

## One candidate already measured — verify and fold in

`split_script_into_forms` (`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:100`)
re-parses the whole accumulated text once per line while an expression stays
unterminated, which is Σ O(i) = O(N²) in the line count of a single multi-line
form. Measured on the release CLI:

| input | time |
|---|---|
| 10 000 elements across 10 000 lines (`data = {\n 1,\n 2,\n … }`) | **7.75 s** |
| the same 10 000 elements on one line | **0.01 s** |

Scaling is textbook quadratic: 500 → 0.02 s, 1000 → 0.07 s, 2000 → 0.28 s,
4000 → 1.14 s. Verify this yourself, then include it as a candidate with the
rest and propose the fix (an incremental delimiter/quote balance check that only
calls `parse_expr` when the form can actually be complete is the obvious
direction — confirm it against the real code, including how
`SyntaxError::is_incomplete()` is used at `:128`).

## Already done — do not re-report these

51 optimizations landed in the current unpushed batch. Do not propose anything
whose mechanism is already covered by one of these titles; if you believe one is
incomplete, that is a *different* claim and must carry evidence that the
remaining cost is still superlinear.

```
avoid eager iteration continuation clones      linearize custom same-test state
batch monomial factor products                 linearize custom set same-test state
batch order-preserving polynomial transforms   linearize delayed association aggregates
batch wide sequence edits                      linearize keysortby decoration
compare polynomial terms sparsely              linearize Module escape closure
cursor ordered matcher arguments               linearize resumable collection traversals
finalize polynomial term batches once          linearize resumable fold families
index collection grouping and aggregation      linearize resumable tensor callbacks
index repeated association reads               linearize reversible suffix matching
index wide arithmetic groups                   linearize string Except filtering
index wide association grouping                linearize targeted map updates
index wide codec formatting and dispatch paths linearize wide DownValue construction
index wide commonest counts                    linearize wide Drop filtering
index wide exact ReplaceAll rules              linearize wide literal string searches
index wide ground DownValues                   linearize wide tagged-value construction
index wide polynomial factor multisets         make resumable predicate loops iterative
index wide polynomial variable bases           prepare wide association lookups once
index wide schema association keys             resolve replace part conflicts in plan trie
index wide string match states                 share ReplaceAll rule tables
index wide subset map paths                    skip impossible ReplaceAll candidates
index wide UpValue symbols                     stop copying ReplaceAll argument suffixes
iterate custom equivalence tests                stop copying traversal suffixes
iterate custom sort comparisons                 stop rescanning unterminated HTML entities
iterate GroupBy output phases                   streamline common factors and division
linearize batched association key selection     use augmenting paths for exact containment
linearize bounded Nearest selection
```

Also out of bounds, by standing decision (`TODO.md` §Decision-gated or deferred):
custom `Sort`/`KeySort` comparator reordering (a semantics decision), full
polynomial factorization, and formatter binary-chain planning (P2, deferred
until after the P1 runtime/stdlib work). The evaluator's per-item
`resume_compound_expression_*` recursion is a known separate lane — you may
list it, but do not plan it here.

## What counts as a candidate

A candidate must be a **complexity-class** reduction, not a constant-factor
tidy-up. For each one, state:

1. **Location** — `path:line`, the function, and what it is called from.
2. **Current class and why** — the nested-scan/rebuild/rescan structure that
   makes it superlinear, in terms of a named `n` (say what `n` is: argument
   count, rule count, key count, line count, …).
3. **Target class** and the mechanism that reaches it.
4. **What input actually triggers it** — a concrete expression or script shape,
   and the `n` at which the cost becomes visible. This is the field that
   separates a real finding from a theoretical one.
5. **Reachability** — is this on an evaluation path a user hits normally, or
   only through an unusual builtin? Is `n` realistically large there?
6. **Risk** — order-sensitivity, observable side-effect order, suspension/resume
   shape, message behavior, allocation on small inputs.

Rank by (real impact × certainty), not by how easy the fix looks.

**Explicitly report the negative cases too**: places that look quadratic but are
bounded in practice (small fixed `n`, an early exit, a guard upstream). Naming
them saves the next survey from re-walking them, and a short "checked, bounded
because …" line is a real deliverable.

## Where to look

Cover the whole workspace, but weight the effort by where wide user data
actually flows: `rum-runtime` and `rum-stdlib` first, then `rum-syntax`, then
the host/CLI/jupyter crates. Structural smells worth grepping for:

- a loop whose body scans or rebuilds the whole collection (`contains`,
  `position`, `iter().find`, `retain` inside a loop over the same data);
- `clone()` / `to_vec()` / `collect()` of a container inside a per-element loop;
- re-parsing, re-formatting, or re-deriving something already computed for a
  prefix of the same input (the `split_script_into_forms` shape);
- recursion that passes the remaining slice **by value** at each step;
- sorting, or building a map/index, inside a loop that runs per element;
- pairwise comparison over all elements where a hash or sort would do;
- string building by repeated concatenation into a fresh buffer.

Do not rely on any single grep. Confirm each hit by reading the function and its
callers, and say what you read.

## Deliverable

1. **The candidate table**, ranked, with the six fields above per candidate.
2. **The re-probe verdict** on the two carried-over `TODO.md` items — still
   holds / no longer reproduces / partially holds, each with current evidence.
3. **The negative list** — looked at, not a candidate, one line of why each.
4. **A batching plan.** Group the accepted candidates into batches that can each
   be one implementation brief. A batch must be internally coherent (same
   subsystem or same mechanism), independently gateable, and small enough that
   its diff can be reviewed in one sitting. State the batch order and why.
5. **Drafted implementation briefs**, one per batch, in `TEMPLATE.md` shape.
   These are raw material for me to rewrite — write them fully, but expect them
   to be edited rather than dispatched as-is.

## Pitfalls this project has hit before (read twice)

- **Optimizing something that is not the bottleneck.** A correct complexity
  observation about a function that runs once per session is not a finding.
  Always answer field 4 and 5 before proposing work.
- **Over-generalized scope.** A correct root-cause observation attributed too
  widely is the single most common defect in these surveys. Attribute it
  exactly as far as the evidence reaches, and no further.
- **Masking a symptom.** Reducing an operation count to hide a slow per-op cost
  is not a fix; name the per-op cost instead.
- **Space-for-time without saying so.** If a proposal trades memory for time,
  say it explicitly and quantify.
- **Findings age.** If you cite a structure, confirm it exists in the tree as it
  stands now, not as an earlier report described it.
- **Small-input cost is a regression.** Every index/table proposal must state
  its threshold story: what keeps small inputs allocation-light. The existing
  code uses a `>= 16` / `>= 32` / `>= 64` threshold pattern — follow it.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands allowed: file reads,
  grep/find, `python3` for parsing, and the release `rum-cli` for throwaway
  probes. Do not run `cargo` builds beyond what an existing release binary
  needs, and do not run the microbench scripts.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- Every claim carries `path:line`. A claim you could not confirm is reported as
  unconfirmed, not asserted.
- If the survey is too large to finish well, prefer **depth over coverage**:
  deliver a smaller number of fully evidenced candidates plus an explicit list
  of the areas you did not reach. Do not pad the list with speculation.

## Report format (your final message)

1. Method — what you scanned, how, and what you deliberately skipped.
2. The candidate table (ranked).
3. The re-probe verdict on the two carried-over items.
4. The negative list.
5. The batching plan and the drafted briefs.
6. Open questions for the director.
