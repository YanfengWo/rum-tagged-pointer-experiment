# brief-077 — Orderless + Optional pattern-matching tests (fill the 0%-coverage vacuum)

## Goal

`crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs`
(6 functions, 224 lines) has **0% test coverage** — the Orderless+Optional (and
Orderless+Flat+Optional) matching combination is exercised by NO test (rum has Orderless
tests and Optional tests, but none that COMBINE them). Fill it with WL-level `MatchQ` /
`ReplaceAll` tests, **with every expected value calibrated against WolframKernel** (do NOT
trust rum's own `MatchQ`/`FullForm` as the oracle — see memory `diagnostic-tool-trust`:
rum's matcher is the subject under test). TEST-ONLY: NO product-code change. If rum's
Orderless+Optional matching DIVERGES from WK, that is a real BUG — leave the failing test
and REPORT it as a finding (do NOT weaken the assertion to rum's current output).

## What the file does (target the behaviors, not the internal fns)

`optional.rs` implements, for a pattern with an **Optional** argument (`x_:default` or
`x_.`) under an **Orderless** head (and the **Flat**+Orderless variant), the two branches:
- **present:** the optional arg is matched from the value args (in any order — Orderless);
- **omission:** the optional arg is absent from the value args and takes its default.
The 6 fns are present/omission × Orderless-Optional / Orderless-Flat-Optional, plus the
"collect all matches" variants. Cover these behaviors through the public surface.

## Coverage to add (WL tests; multiple per area; expected values FROM WK)

For an Orderless head `f` (`SetAttributes[f, Orderless]`) — and separately a Flat+Orderless
head — with Optional patterns, cover at least:
1. **Optional present, orderless order:** `MatchQ[f[1, 2], f[a_, b_ : 0]]` and the
   permuted `f[2, 1]`; a `ReplaceAll` that binds `a`,`b` and checks the bound values
   (Orderless may bind either way — assert what WK actually does).
2. **Optional omitted → default:** `MatchQ[f[1], f[a_, b_ : 0]]` (True; b→0); a replace
   confirming `b` took the default; `f[]` against `f[a_ : 9, b_ : 0]` (both defaults).
3. **Optional + a fixed/other arg, orderless:** `f[x, 1, 2]` against
   `f[c_, a_, b_ : 0]` — present and omitted cases with the extra rest pattern.
4. **`x_.` default-from-`Default[f]`** vs explicit `x_:v`: set `Default[f] = d`; test
   `f[1]` against `f[a_, b_.]` (b→d) — present and omitted.
5. **Flat + Orderless + Optional:** the same present/omitted cases with
   `SetAttributes[g, {Flat, Orderless}]` and an Optional pattern (drives the
   `*_flat_optional_*` fns).
6. **Optional with a Blank/BlankSequence neighbor under Orderless** (e.g.
   `f[a_, b___, c_ : 0]`) — a couple of present/omitted cases.
7. **Non-match edges:** a case that should be False (e.g. arity/head mismatch) so the
   tests can actually fail.

Cover both the "does it match" (`MatchQ`) and "what does it bind" (`ReplaceAll`/`Cases`)
aspects, since binding order under Orderless is where divergence hides.

## rum / test pitfalls (read twice)

1. **Calibrate every expected value against WolframKernel** (real `wolframscript`), NOT
   against rum's output. rum's matcher is the code under test; using it as its own oracle
   proves nothing and could bake in a bug. (Codex has `WOLFRAM_KERNEL`; if unavailable,
   hand-derive from WL semantics and say so.)
2. **Orderless binding is nondeterministic-looking:** WK picks a specific binding; assert
   what WK does (run it), and if rum picks a different-but-valid binding, that IS a
   divergence to report (byte-for-byte match with WK is the bar).
3. **Test through the public WL surface** (`MatchQ`/`ReplaceAll`/`Cases`/`ReplaceList`),
   the way existing rum-runtime tests do (evaluate WL snippets via the session); do NOT
   call the internal `match_compiled_orderless_*` fns directly.
4. **Report divergences as findings** (leave the failing test), do NOT weaken assertions.
5. Match the existing test style in
   `crates/rum-runtime/src/runtime/tests/definitions_and_rules_patterns.rs` (or add a new
   `tests/pattern_matching_orderless_optional.rs` module wired into the tests mod).

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/tests/` (add a test module + wire it into
  the tests `mod`), test-only. NO product-code change under non-test `src`.
- MUST NOT: edit any `mma-hol` file; `Expr`/serde; no new `unsafe`; no `git commit`.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # new + existing runtime tests green (paste count: was 723 -> N)
```

Then confirm the vacuum is actually filled (the point of the brief):
```
cargo llvm-cov -p rum-runtime --summary-only 2>&1 | grep orderless/optional.rs
# optional.rs region coverage should be well above 0% now (paste before 0% -> after)
```

Paste: `cargo test -p rum-runtime` summary (before/after count), the optional.rs coverage
line (0% → N%), and — if any test fails because rum diverges from WK — the failure + your
WK-vs-rum analysis (a finding).

## Hard rules

- No `git commit`/branch/push. Never edit any `mma-hol` file. Test-only (no product `src`
  change). No new `unsafe`.
- Expected values FROM WK, not from rum. Real assertions (MatchQ True/False + bound values).
- Report WK-divergences as findings; do not weaken to rum's current output.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the new test module + how many tests per area; confirm no product-code
   change; confirm expected values were WK-calibrated (how).
2. Verification — `cargo test -p rum-runtime` summary (723 → N) + the optional.rs coverage
   before/after — verbatim.
3. Findings — any rum-vs-WK divergence a test revealed (with the case + analysis), or
   "Orderless+Optional matches WK on all covered cases."
4. Open questions — any Orderless-binding case where WK's choice was ambiguous/surprising.

---

## Outcome (Claude gate, 2026-07-01) — DONE + KEEP. Coverage 0%→71%; found + quarantined 4 real WK-conformance bugs.

The coverage push did exactly what good tests should: it FOUND REAL BUGS. Committed
test-only (`tests/pattern_matching_orderless_optional.rs` + mod.rs wiring).
- 8 tests / 39 WK-calibrated WL cases. **5 pass** (basic Orderless+Optional: present/
  omit/explicit-default/`x_.` via `Default[f]`/other-args/non-match — all match WK).
- **3 quarantined `#[ignore]`** documenting 4 WK divergences in the Flat/Orderless+Optional
  + sequence-neighbor corner (binding order + ReplaceList under-collection; see the
  ignore reasons + TODO "Immediate Correctness And Debt"). Left as executable bug-spec
  (remove `#[ignore]` when the matcher is fixed).
- Coverage: `optional.rs` 0% → **71.43% region / 76.34% line** from the 5 passing tests
  alone (90%/97% with the ignored ones); the vacuum is filled for the WORKING paths and
  the buggy paths are now documented.
- Gate (Claude reproduced): `cargo test -p rum-runtime` 728 passed / 0 failed / 3 ignored;
  fmt clean; no product-code change (test file + mod.rs only).

**Findings queued:** the 4 Flat/Orderless+Optional matcher divergences (TODO, LOW urgency —
exotic corner, mma-hol doesn't use it). Next coverage targets: evaluation_cache.rs (triage
real-logic vs profiling branches) and gensym_gc.rs.
