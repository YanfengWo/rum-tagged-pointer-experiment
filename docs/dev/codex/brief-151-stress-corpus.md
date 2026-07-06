# Brief 151 — rum-native stress corpus (integration depth)

Implementation brief, TEST-ONLY (same rules as brief-150: src changes only
as `#[cfg(test)]` modules; discovered bugs get `#[ignore]` + `// BUG:` +
report, never a production fix here). Closes the integration-depth gap
left when the external acceptance corpus was removed: kernel behavior NEAR
the guards and UNDER load, exercised end to end.

## Corpus (each item: a fast default-suite variant + where noted a heavy
`#[ignore]`d variant with its invocation documented in the test's doc
comment)

1. **Guard-adjacent recursion**: user recursion succeeding just BELOW
   `recursion_limit` (assert exact result), tripping just ABOVE
   (`TerminatedEvaluation["RecursionLimit"]`, session usable after);
   a shape that trips the native stack guard (`StackLimit`) with a
   REDUCED `native_stack_limit` so the default suite stays fast; parser
   nesting at `MAX_PARSE_DEPTH` boundary (127 parses, 129 fails
   gracefully — find the exact boundary and pin it).
2. **Large rewriting workloads** (exact expected results, no timing
   assertions): a rule-based symbolic differentiator (define d[x]-style
   downvalue rules; differentiate a generated ~200-node expression;
   assert the exact simplified result); a Peano-arithmetic evaluator
   computing e.g. 12×13 through pure rewriting; a big list pipeline
   (`Fold`/`Map`/`Select`/`GatherBy` over `Range[50000]` within default
   limits — pick sizes that pass the default budgets or set limits
   explicitly); a string-pattern workload (`TokenizeWith` an INI grammar
   over a ~100KB generated config, assert token counts and spot spans).
   Heavy `#[ignore]` variants scale these up ~10x.
3. **Cancellation and timeout under load**: cancel from another thread
   mid-heavy-rewrite → `Cancelled` outcome within a generous deadline,
   session usable and correct afterward; `eval_with_timeout` on the same
   workload → `TimeConstraint`; a 10-cycle stability loop (heavy eval →
   cancel → verify usable) in an `#[ignore]`d test (the brief-066
   pattern).
4. **Snapshot/restore at scale**: build ~500 definitions (generated
   downvalues + attributes), `snapshot_to_bytes` → fresh session →
   `restore_from_bytes` → assert a sample of definitions evaluate
   identically and `definitions()` counts match.
5. **Repeated recoverable trips**: one session driven through every
   recoverable limit (recursion, stack, iteration-work,
   generated-element, replace-repeated, match-depth) three times each in
   sequence, asserting the session stays correct after all of it (the
   "battered session" test).

## Placement

`crates/rum-host/tests/stress.rs` for host-level items (3, 4, 5);
runtime/stdlib test trees for guard-adjacent and rewriting items (1, 2),
following neighbors. Heavy variants: `#[ignore]` + doc comment naming the
invocation (`cargo test -p rum-host --test stress -- --ignored`, run
under `scripts/guarded-test-run.sh` when invoked manually).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# default-suite wall time must stay sane — paste the slowest new suite line
scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-host --test stress -- --ignored --nocapture 2>&1 | tail -15
# ^ run the heavy variants ONCE, bounded, paste their pass lines
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`, TEST-ONLY.
- Exact-result assertions everywhere — no timing/perf assertions (this is
  a correctness-under-load corpus, not a benchmark; the microbench owns
  timing).
- Default suite stays fast: anything over ~5s goes `#[ignore]`.
- Stop-loss: same failure twice on one item → mark, move on, report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Corpus disposition — each item: landed (test names) / bug-found /
   adjusted-with-reason.
2. Discovered bugs with exact inputs (most important section).
3. Verification — all commands verbatim incl. the ignored-run tail.
4. Stop-loss / open questions.
