# brief-320 — discover UpValue candidates at tag positions only

## Goal

UpValue dispatch collects candidate symbols by walking an expression
recursively, so a call whose arguments contain a deeply nested symbol scans
every UpValue stored on that symbol — and then rejects all of them. Restrict
candidate discovery to the positions where a stored UpValue's tag can actually
live. Dispatch results, definition ordering and matching order must not change.

## What the diagnosis established (brief-319, verified by the director)

Installing `n` UpValue definitions on one symbol is quadratic: 38.6 s at
n = 20 000. The cost is not in the store — it is in dispatch, triggered by the
statement separator.

Each script line ending in `;` lowers to `CompoundExpression[…, Null]`, an
ordinary `HoldAll` builtin, and ordinary calls enter UpValue dispatch **before**
builtin dispatch (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:34`). The
wrapper's arguments contain the tag symbol nested inside the `TagSetDelayed`, so
every line re-materializes and re-scans every UpValue installed so far.

Counters on a 128-line probe:

```
rum-profile definition-dispatch kind=up_value symbol=g candidates=8128
  prefilter_skips=8128 fast_matches=0 matcher_matches=0
  prefilter_skip_rate_per_mille=1000
```

8128 is C(128,2), and **every candidate was rejected by the shape prefilter** —
the entire scan is wasted work. Removing only the trailing semicolons drops the
count to zero while still installing all 128 definitions.

Director's confirmation at n = 5000: **2.54 s with trailing semicolons, 0.03 s
without** — an 85× difference from punctuation alone.

The three candidates the earlier survey and the director suspected — index
rebuild on non-terminal insert, specificity recomputation in the
`partition_point` comparator, and the dispatch-metadata prune — were each
measured and **none is the quadratic term**. Do not spend time on them.

## The fix, and why it preserves semantics

A stored UpValue's tag must be either a direct symbol argument or the symbol
head of a direct call argument — enforced at installation by
`valid_upvalue_tag_position`
(`crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:326`)
and revalidated on value-table restoration
(`definitions/value_table_assignment.rs:187`).

Therefore a symbol found **deeper** than those positions cannot own an UpValue
that could match this call. Discovering it produces a candidate that the shape
prefilter is guaranteed to reject. Restricting discovery to the same positions
installation allows is not a heuristic — it is the dual of the storage rule, so
the set of candidates that could ever match is unchanged.

Concretely:

- Extract a shared helper for "the tag symbol at this argument position" from
  `valid_upvalue_tag_position`, so discovery and installation cannot drift
  apart. Sharing the rule is the point; two copies of it would be a latent bug.
- Have `collect_upvalue_candidate_symbols` (`:345`) inspect only each direct
  argument and its direct symbol head.
- Remove `collect_expr_symbols_unique` (`:413`) if nothing else uses it — check
  before deleting.

**Do not** special-case `CompoundExpression`, suppress UpValues by builtin name,
or change its dispatch class. A fix that names one builtin would leave every
other wrapper shape broken.

## Behaviour that must not change

- Which UpValue wins for a given call, including when several could match.
- Definition ordering by specificity — it decides which rule wins.
- The dispatch result for tags at legitimate positions: a direct symbol
  argument, and the head of a direct call argument. Both must still be found.

Name the existing tests that pin these, and add a case for a tag appearing
**only** at a deep position, asserting it is not dispatched — which is today's
behaviour too, since the prefilter rejects it; the change makes that a
structural property rather than a filtered one.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else — the UpValue store, specificity sorting,
`CompoundExpression`, `eval_dispatch`, public types, and the profiling
infrastructure.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Candidate discovery restricted to tag positions, with the rule shared with
  installation rather than duplicated.
- The counter evidence: the 128-line probe's `candidates=` must fall from 8128
  to zero while all 128 definitions still install and dispatch correctly.
- The process-level measurement rerun at n = 5000 / 10000 / 20000 — this is the
  symptom that started the line, so it belongs in the gate even though the
  in-kernel microbench cannot reproduce per-form wrappers.
- Tests as described above.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (5000, 10000, 20000):
    lines = ["g /: h%d[g] := %d;" % (i, i) for i in range(1, n + 1)]
    open("/tmp/b320-%d.rum" % n, "w").write(
        "\n".join(lines) + '\nPrint["uv=", h%d[g]];\n' % (n // 2))
PY
for n in 5000 10000 20000; do echo "--- $n ---"; /usr/bin/time -p ./target/release/rum-cli /tmp/b320-$n.rum < /dev/null; done
```

Each must print the correct `uv=`, and growth must be linear rather than
4.2–5.5× per doubling. Reference: 1.69 / 7.03 / 38.56 s before.

**This touches evaluation dispatch, so the microbench A/B is a hard gate.** Run
`scripts/core-language-microbench.sh` on base and candidate and paste the
`upvalue-candidates-tiny`, `upvalue-candidates-wide-small` and
`upvalue-candidates-wide-large` rows plus their same-tier neighbors. Any hot
label regression is a failure. State plainly whether the machine was quiet.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the candidate count does not fall to zero, or any
  ordering or dispatch test changes, STOP and report rather than adjusting a
  test.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and how discovery and
   installation now share the rule.
2. The counter rows before and after.
3. Verification — each command with verbatim output: test counts, the process
   timings, and the A/B rows with neighbors.
4. Stop-loss / open questions.
