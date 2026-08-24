# brief-319 — diagnose the UpValue definition-install quadratic

## Goal

Installing many `UpValue` definitions on one symbol is superlinear — 38 s for
20 000 definitions. Find where the time actually goes, and propose a fix.

**Scope: NONE — diagnostic only, no source change.** Reading, grepping, the
env-gated runtime counters, and throwaway probes with the release `rum-cli` are
in scope; leave no probe files in the repository. Probes must feed stdin from
`/dev/null` — the CLI waits for input otherwise.

## Measured (release CLI, verified by the director — do not re-derive)

`g /: h<i>[g] := <i>;` repeated `n` times:

| n | time |
|---|---|
| 5 000 | 1.69 s |
| 10 000 | 7.03 s |
| 20 000 | **38.56 s** |

4.2–5.5× per doubling.

**The cost is in installing definitions, not in dispatching them.** At
n = 10 000, the definitions alone take 6.84 s and adding one call that hits an
UpValue takes 7.00 s.

## What the director already narrowed down

Four shapes, n = 5 000 each:

| shape | time |
|---|---|
| A: `g /: h<i>[g] := <i>` — distinct lhs, UpValue | **1.68 s** |
| B: `g /: h[g] := <i>` — identical lhs, UpValue | 0.03 s |
| C: `f[<i>] := <i>` — distinct lhs, DownValue | 0.03 s |
| D: `g /: h[g, <i>] := <i>` — distinct lhs, same head symbol | **1.87 s** |

So it is specific to **inserting distinct UpValue definitions**: the replace
path is fine, the DownValue path is fine, and the head symbol makes no
difference.

Candidates inspected but **not** confirmed — treat these as leads, not answers:

1. `note_up_value_stored`
   (`crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:114-122`)
   maintains the index incrementally **only when the new entry lands at the
   end**; any other position sets `up_value_index = None`, and the next
   `prepare_up_value_store` rebuilds it in full. If insertions are landing
   mid-vector, that is O(n²) — but it is not established that they do.
2. `replace_or_insert_definition_by_specificity`
   (`crates/rum-runtime/src/runtime/definitions/definition_tags.rs:352-375`)
   recomputes `definition_specificity` inside the `partition_point` comparator,
   and `Vec::insert` at a non-terminal position is O(n).
3. `prune_dispatch_metadata_if_unused`
   (`down_value_dispatch.rs:257`) drops the whole metadata when `down_values` is
   empty and no tagged stores remain — the probe symbol `g` has no DownValues,
   so check whether this fires on this path and forces a rebuild.

An earlier survey also flagged the specificity recomputation as a candidate
(`definition_tags.rs`, "O(n·s·log n)") without measuring it.

## What to produce

1. **Where the time goes**, established rather than argued. The env-gated
   counters are the sharpest instrument here — an index rebuild count, or an
   insertion-position histogram, tells you in one run what timing cannot. If a
   sampling profile is genuinely needed, say so and stop; the director will
   redispatch with the sandbox that allows process attach.
2. **Which of the three candidates is the dominant term**, with the evidence
   that distinguishes them. If it is none of them, say what it is instead.
3. **Whether the insertion position is terminal or not** for shape A, since
   candidate 1 hinges entirely on that.
4. **A proposed fix**, with the smallest change that moves the number, plus a
   drafted implementation brief in `TEMPLATE.md` shape.
5. **What must not change**: definition ordering by specificity is observable —
   it decides which rule wins — so any fix must preserve the resulting order and
   the dispatch results. Name the tests that pin that.

## Pitfalls (read twice)

- **Do not fix anything in this brief.** A diagnosis that names the wrong term
  costs one dispatch; a fix aimed at the wrong term costs a gate cycle and
  leaves a diff to unwind.
- **Counters beat timings.** A rebuild count or a position histogram is
  load-immune; a stopwatch is not.
- **Attribute exactly as far as the evidence reaches.** Three leads are listed
  above precisely because none is confirmed.
- **Findings age.** This code was modified today by `de4971c` (wide tagged-value
  construction) and by brief-309's dispatch work; confirm what is in the tree
  now.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, the release `rum-cli`, and `cargo` only if a counter needs a build.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Every claim carries `path:line` or a counter reading. A claim you could not
  confirm is reported as unconfirmed.

## Report format (your final message)

1. Method — what you measured and how.
2. Where the time goes, with the evidence.
3. The verdict on each of the three candidates.
4. The proposed fix and its drafted brief.
5. Open questions for the director.
