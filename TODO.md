# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log).

## ACTIVE PLAN — publication-prep line (opened 2026-07-21)

User ruled divergence is now SUFFICIENT and reversed the 2026-07-06
"publication deferred indefinitely" directive. Moving to pre-publication
prep. Two shaping rulings (2026-07-21):

- **Release form = source-available GitHub only** (no crates.io). ⇒ no
  per-crate `publish` flags, no crates.io metadata/name/semver burden; the
  only metadata add is a `repository` field once the URL exists.
- **Public tree is fully transparent** — `CLAUDE.md`, `docs/dev/` (briefs +
  design docs), `PROGRESS.md`, the Codex dispatch tooling all stay public.
  So "cleanup" is an accuracy pass, not a removal pass.

Four buckets, ordered by value + dependency (security first — the audit may
surface fixes that the SECURITY.md threat model must reflect):

1. **Security audit — CLOSED 2026-07-21** (highest value; the product is "run
   untrusted scripts safely"). Endstate: no sandbox bypass, three hard lines
   hold, both audit findings fixed, secrets + dependency review done. The one
   carry-forward is the SECURITY.md write-up, which is a bucket-2 (docs) task.
   - [x] **brief-289 — capability-classification & limit-coverage consistency
     audit (diagnostic). DONE 2026-07-21** (commit 3169cb4; findings
     `docs/dev/publication-audit/capability-consistency-findings.md`).
     Sections 1-2 by Codex (642-row classification table; gate-coverage all
     covered), sections 3-4 + verdict director-completed (Codex hit the OpenAI
     cybersecurity filter at the section-3 resource probes; live probing is
     director-run — see `codex-cybersecurity-filter.md`). Codex corrected the
     brief's stale checkpoint pointer: real enforcement is at
     `eval_dispatch/mod.rs:135-138`, not `builtins_context_loader.rs:68`.
     **Verdict: no sandbox bypass; the three hard lines hold** (zero kernel
     `unsafe`; zero kernel `std::fs`; no `std::net`). Two findings:
     - [x] **Finding A (low, doc) — DONE 2026-07-21.** Added a transitive-
       `FileRead` `Limits` note to `BeginPackage`/`DeclarePackage` in
       `docgen.rs` (the `LIMITS` table) and regenerated
       `docs/reference/builtins.md` (drift test `builtin_reference_is_in_sync`
       green). Capability label stays `Pure` per the user ruling; the reference
       now documents that the dependency/autoload form transitively requires
       `FileRead` and fails closed when denied.
     - [x] **brief-290 — Finding B (medium, hardening): `Power` now bounded by
       `integer_work_limit`. DONE 2026-07-21.** Pre-compute size estimate in
       `eval_power` (`|exponent| * (num_bits + den_bits)` vs
       `integer_work_limit * 64` bits), reusing `IntegerWorkLimitExceeded` so
       the rendering is byte-identical to an oversized `Factorial`; trivial
       bases 0/1/-1 skipped; `(1/2)^big` and `2^-big` guarded. `2^(10^9)` now
       returns `TerminatedEvaluation["IntegerWorkLimit"]`. Power-only scope
       (bit shifts etc. deliberately left). Gate: scope clean, workspace green
       (0 failed), all Power behavior tests pass. **Director optimized the
       guard to ZERO heap allocation at the gate** (Codex's BigInt-based version
       measured +4-6% on Power; the zero-alloc version — `.bits()` +
       `exponent.magnitude().to_u128()` + u128 saturating compare — A/B'd
       0.1447s vs 0.1503s baseline on `Table[k^3,{k,1,90000}]`, i.e. within
       noise, no regression).
     - [x] SECURITY.md states the operational requirement the audit surfaced
       (DONE 2026-07-21, in bucket 2): untrusted evaluation requires the host to
       set the two opt-in bounds (request memory ceiling + wall-clock timeout);
       the always-on structural limits bound computation SHAPE, not the cost of
       a single expensive value.
   - [x] Dependency/supply-chain — DONE 2026-07-21 (manual review; tooled scan
     is a pre-publication local step). `cargo audit`/`cargo deny` are not
     installed here and need the network, so recorded in the findings file
     §5 as `cargo install cargo-audit cargo-deny; cargo audit; cargo deny
     check` for the maintainer. Manual `cargo tree` review: all external crates
     mainstream/maintained; security-sensitive kernel deps are just `num` +
     RustCrypto digests (behind `hash`); the heavy `tokio`/`zeromq` stack is
     `rum-jupyter`-only (kernel + `rum-host` embedding path is free of it).
     Non-security notes for `cargo deny`: `thiserror` v1&v2, `getrandom`
     v0.3&v0.4 version skews.
   - [x] **Secrets/sensitive-info scan — DONE 2026-07-21 (director), CLEAN.**
     Current tree (git grep ~15 pattern classes: private keys, AWS/GitHub/
     OpenAI/Google/Slack prefixes, assignment-style secrets, auth headers,
     sensitive file types, tracked `.json` [zero], emails/PII [none]) AND all
     1495 commits of preserved history (`git log --all -G` / `--diff-filter=A`:
     no private keys / cloud prefixes / jupyter keys / `.env`-style assignments,
     no sensitive-extension file ever added). Two non-secret notes → cleanup
     bucket (both handled 2026-07-21): local absolute home paths in 4 docs were
     genericized (relative path in `CLAUDE.md`, `~`-relative in the 3 findings
     docs); `scripts/rum-jupyter-browser-smoke.py`'s `rum-browser-smoke` is a
     benign fixed test token, left as-is. A tooled re-verify (gitleaks/trufflehog,
     `--log` over history) at the mechanism-handoff gate as defense-in-depth.
2. **Doc completion** (in progress):
   - [x] `SECURITY.md` — DONE 2026-07-21. Sandbox threat model from the audit
     (capabilities, always-on limits table, the untrusted-eval requirement to
     set the opt-in memory ceiling + wall-clock timeout, `Protected` is not a
     boundary, crash-only isolation) + private disclosure to
     `liusq@tsinghua.edu.cn`.
   - [x] `CONTRIBUTING.md` — DONE 2026-07-21. Encodes the permanent testing
     policy (no external-engine differential testing), build/test commands,
     safe-Rust + hot-path-A/B + file-size rules, reference-drift regeneration,
     MIT contribution terms.
   - [x] `CODE_OF_CONDUCT.md` — DONE 2026-07-21 (Contributor Covenant 2.1,
     contact `liusq@tsinghua.edu.cn`).
   - [x] `CHANGELOG.md` — DONE 2026-07-21 (0.1.0 initial entry; Keep a
     Changelog format). README gained a "Contributing, security, and license"
     section linking all of these.
   - [x] rustdoc completeness — DONE 2026-07-21. `cargo doc --workspace
     --no-deps` is warning-clean; the embedding API `rum-host` already carries
     `#![warn(missing_docs)]` plus a 40-line crate doc, so the public surface is
     fully documented.
   - [x] Crate-level docs — DONE 2026-07-21. Added `//!` crate docs to the four
     front-end crates that lacked them (`rum-terminal`/`rum-repl`/`rum-cli`/
     `rum-jupyter`); the kernel crates already had them. Per-crate README files
     were deemed unnecessary under source-available-only publication (no
     crates.io readme surface); crate-level `//!` docs are the valuable form.

   **Bucket 2 (doc completion) CLOSED 2026-07-21.**
3. **Cleanup / accuracy (light):**
   - [x] Docs/scripts navigability — DONE 2026-07-21 (user-requested). Added
     `docs/README.md` (documentation index categorizing the 10 top-level docs +
     the guide/reference/dev subfolders), `docs/dev/README.md` (explains dev/ is
     the transparent internal working record — codex briefs, design docs,
     usability ledgers, audit findings — not needed to use rum), and
     `scripts/README.md` (categorizes the 12 scripts: perf/profiling, run-safety,
     jupyter, maintainer workflow). Slimmed the top-level README's Development
     section to link out to CONTRIBUTING/scripts/docs instead of inlining detail,
     and added a Documentation section. Folder layout kept as-is (no file moves —
     they would churn history and break internal cross-links like AGENTS.md's
     reading path and the guide's contract-sources list); order is imposed by the
     index READMEs. New links validated (zero dead).
   - [x] Local-path leaks — DONE 2026-07-21. Genericized the home-dir absolute
     paths in 4 docs: `CLAUDE.md`'s archive reference is now relative
     (`../Archive/rum-history/`), and the 3 findings/inventory docs' pasted
     cargo/panic transcripts now use `~`-relative paths. The tracked tree has
     zero `/Users/<name>` home paths.
   - [x] `.claude/hooks/guard-dispatch.py` (the one tracked `.claude` file)
     confirmed publish-safe (no absolute paths, no secrets).
   - [x] README/DISCLAIMER front-door read-through — DONE 2026-07-21 (README
     links validated zero-dead; DISCLAIMER legal posture reviewed, accurate).

   **Bucket 3 (cleanup / accuracy) CLOSED 2026-07-21.**
4. **Mechanism / handoff (light):** waiting on the GitHub owner/repo name from
   the user, then:
   - [ ] add `repository = "https://github.com/<owner>/<repo>"` to `Cargo.toml`;
   - [ ] replace the placeholder Jupyter `help_links` URL
     `https://example.invalid/rum` in `crates/rum-jupyter/src/adapter/mod.rs`
     with the real repository URL (surfaced during the `.rum`-extension review);
   - [ ] tag `v0.1.0` (annotated);
   - [ ] cold-backup bundle (per `cold-backup-bundle-procedure`, at the final
     publishable state);
   - [ ] hand the user the exact `git remote add origin <url>` /
     `git push -u origin main` / `git push origin v0.1.0` steps. GitHub
     empty-repo gotcha already reviewed with the user: create the repo with NO
     README/.gitignore/license so the 1495-commit history pushes fast-forward.

   **`.rum` extension decision (DONE 2026-07-21, commit 3e71ee2):** `rum` adopts
   its own `.rum` script suffix. The runtime is extension-agnostic (runs any
   suffix; `Get`/`Needs` append nothing), and the Jupyter kernel already
   advertised `.rum` + `text/x-rum`; the guide examples and DISCLAIMER were
   updated to match (they had lagged at `.wl`). `.wl` still runs; `.rum` is the
   preferred label to avoid implying Wolfram-Language compatibility.

Memory: `publication-prep-line.md`.

## COMPLETE — debt clearing + REPL/trace line (2026-07-20 → 07-21, briefs 280-288)

Two user-directed sweeps, both closed.

**Debt clearing (briefs 280-281).** The workspace now has ZERO files over
AGENTS.md's 35k architecture-debt line.
- `builtin_catalog/stdlib_attributes.rs` 30,747 bytes → parent 8.8k plus
  four first-byte chunks (6.8/6.6/4.6/6.3k). The 2026-07-18 "no family
  boundary" ruling WAS the split axis: a table whose only structure is
  its ASCII name order gets split along that order. Lookup stays
  const-time — a `const fn` first-byte dispatch then ONE
  `binary_search_by`, no `OnceLock` on the matcher-hot path. Compile-time
  strength preserved, not traded down (per-chunk sortedness + per-chunk
  disjoint byte-range membership ≡ the old single global assertion).
  That entry had also mispredicted the cost: it claimed a split would
  force "search multiple segments, or build+search a Vec" — neither
  happened.
- `rum-host/src/tests.rs` 68,639 bytes (2× the line, undocumented) → six
  files by responsibility, mirroring the embedding line's own increments.
  Pure move, proven by before/after set-equality of all 77 test names and
  all 18 helper names. WATCH: `capability_profiles.rs` landed at 19,614
  bytes with almost no headroom; its next boundary is already visible
  (dispatch-blocking vs named-profile pins vs capability-policy
  composition).

**REPL/trace (briefs 282-288).** See the Standing Engineering Debt
entries for detail. Headline: REPL rendering frozen with full-output
snapshots (`contains` 48 → 0); `:trace`; `TraceEvent` switched from
`String` to `Expr`, deleting a render→parse round trip and cutting
tracing overhead from 5.5-5.8× to 2.4×; `TraceDetail`, the lossless
symbol that names WHICH overlapping definition fired; and
`:trace --rules`. Reference 641 → 642, workspace 1662 → 1675/0.

Re-probed at close and deliberately NOT actioned: symbolic `Plus`/`Times`
like-term detection is confirmed superlinear on distinct terms (collection
cost 1.13 → 2.20 → 4.83 → 13.37 ms for n = 100/200/400/800, i.e. doubling
ratios 1.94× → 2.20× → 2.77×, accelerating toward quadratic), visible from
roughly n ≥ 400. Like terms stay linear via a fast path. **The entry's own
revisit trigger — profiler evidence from a REAL symbolic-algebra workload —
is still not met**; this was a synthetic worst case. Left as accepted
tradeoff, now with numbers and an inflection point on record.

## COMPLETE — matcher OneIdentity/Optional + Flat respec line (user ruling 2026-07-20, closed same day, briefs 275-279)

Re-scoped from the Standing Engineering Debt matcher entry after
re-probing (2026-07-20). The old target — reproduce WK's byte-exact
Flat+Orderless+Optional enumeration (the two `#[ignore]` tests) — is a
policy fossil: the expected strings are recorded third-party engine
output (difftest era), include WK's own duplicate rows, and cannot be
legally regenerated under the Testing Policy. The real content
decomposes into a common-path semantic gap (never previously recorded)
and a smaller-than-recorded exotic half (the wrapped-chunk enumeration
machinery already exists — `ReplaceList[a+b+c, x_+y__]` produces
`Plus[a,b]`-style candidates today).

Grounding (probed + code-read 2026-07-20):

- **OneIdentity promoted-singleton matching is missing** beyond the
  single-arg case: `core.rs` `match_compiled_call_owned`'s non-Call arm
  handles only `[single]` patterns (hard False for `h[x_, y_.]` vs an
  atom), and the Call arm has no head-mismatch fallback (WL: ANY
  expression matches `x_ + y_.` with y→0). This kills the WL
  coefficient idioms (`x_ + y_.`, `c_. x_^n_.`).
- **`Default[Plus]`/`Default[Times]` are unregistered** (dot-optionals
  resolve via downvalues on the `Default` symbol —
  `resolve_optional_implicit_default`; registration is data, no new
  mechanism).
- Flat singleton wrapping (`g[1]` candidates) and its Optional compound
  are the remaining exotic half; brief-080's failure mode (order changes
  regressing neighbors) demands the respec land first.

Queue (user-approved plan items 1-4, 2026-07-20):

- [x] brief-275 — items 2: OneIdentity promoted-singleton matching
  (atom arm generalized past `[single]`; Call-arm head-mismatch
  fallback; both routed through the existing orderless/flat arg
  dispatch with `vec![value]`) + register `Default[Plus] = 0`,
  `Default[Times] = 1` at install. Deliberate WL-conformant behavior
  change: `MatchQ[anything, x_ + y_.]` becomes True. Matcher = hot
  path ⇒ microbench A/B. DONE 2026-07-20 (revision 1: round 1
  eval-seeded `Default` downvalues leaked into the host user-
  definitions contract — new symmetric `installed_system_down_values`
  kernel classification, content-equal conservative predicate, OR'd
  into all four filter sites; host test unchanged and green. Gate:
  workspace 1651/0; A/B 14 faster/18 slower/6 flat — named targets
  (`matchq-*`, `downvalue-pattern-hit`, `replaceall-simple`) all flat-
  to-favorable; the three worst movements are unrelated loop-control
  labels that don't touch the matcher or Default lookup at all —
  noise-floor by the AGENTS.md unaffected-control rule, not a
  regression. Nested `c_. * b_^e_.` stays unmatched at the Power
  boundary — stated stop-loss, not chased.)
- [x] brief-276 Part A DONE — items 1+3: the two `#[ignore]` fossils
  respecced to rum's OWN documented contract (un-ignored, re-pinned to
  reproduced current truth), `assert_wk_cases` → `assert_rum_cases`
  naming hygiene, WK-order test names de-WK-ified, contract doc
  `docs/dev/matcher-flat-optional-contract.md` (enumeration-order
  conformance recorded WON'T-FIX). Workspace 1653/0.
- [x] brief-277/278/279 — item 4 DONE (2026-07-20): non-`OneIdentity`
  `Flat` singleton wrapped candidates. Three dispatches, the first two
  stop-lossed CLEANLY on my Scope omissions (277: the `one_identity`
  threading I rooted in `core.rs` reaches `sequence/entry.rs`,
  `orderless/optional.rs`, `replace/list/matching.rs`; 278: the new
  contract legitimately adds rows to a `ReplaceList` assertion in
  `definitions_and_rules_part3.rs`). Codex stopped before/rolled back
  each time with a clean tree — correct calls, my briefs were wrong.
  Final design threads nothing across a file boundary: both
  `allow_flat_single` call sites already hold `flat_head`, so
  `one_identity` is derived locally; the plain-`Flat` loop threads it
  but all six call sites are inside `sequence.rs`. `core.rs` untouched,
  so brief-275's work is safe by construction.
  **Two findings beyond the stated contract:**
  (a) the line's justification was re-grounded independently of the
  retired WK fossil — `MatchQ[g[1,2], g[a_g, b_]]` was False though
  `g[g[1], 2]` flattens to `g[1,2]`; that and `MatchQ[g[1], g[a_g]]`
  became the acceptance criteria, both now True;
  (b) the `optional.rs` repair (flat collect-all omission arm was
  first-match-shaped, keeping at most ONE omission match) is NOT
  `OneIdentity`-gated, so it fixes **built-in `Plus`/`Times`** too —
  they recovered dropped rows, the tell being that `Orderless` used to
  collect `{a,{b},0}` but not its mirror `{b,{a},0}`. My "zero built-in
  exposure" grounding held only for the wrapped-candidate half. Pinned
  at the gate by a new test
  (`flat_optional_omission_collects_every_match_for_builtin_heads`) and
  documented in the contract doc.
  Gate: workspace 1654/0; my own A/B (same-path base/candidate builds)
  = target matcher labels |delta| median 0.97%, max +1.29%, while
  unaffected controls moved median 2.29% / max 15.34% — controls moved
  MORE than targets, so noise floor by the AGENTS.md rule. 2^n
  candidate growth accepted per user ruling (no pruning; WL does none).

## COMPLETE — row-workload pure line (DB-prep; user ruling 2026-07-20, closed same day, briefs 272-274)

Pure-kernel preparation for future downstream database applications (SQL
or NoSQL): make the "table = List of Associations" workload complete and
honest. Scope ruling: pure builtins only — no drivers, no handles, no
Network capability, no symbolic SQL AST (those wait for a real downstream
pull; capability u8 bitmask is full and widening it is its own decision).
Endstate: reference 641, workspace 1644/0, guide syntax.md documents
named-slot fill semantics. Debt CLEARED (user ruling, same day): both
`stdlib_attributes.rs` binary-searched tables now carry compile-time
strict-ASCII-sortedness assertions (negative-calibrated — the
brief-273 mis-sorted insert now fails compilation).

Audit (probes run 2026-07-20, raw outputs in session scratchpad;
green surface: GroupBy incl. reducer form, Merge, `[[All,"key"]]` column
extraction, Lookup column+default, KeyTake over rows, SelectFirst,
TakeLargestBy, DeleteDuplicatesBy, CountsBy, GatherBy, PositionIndex,
Join concat/assoc-merge, AssociationThread, Missing propagation):

- **F1** `JoinAcross` absent (inert) — the one missing relational
  primitive. → brief-272.
- **F2** Named slots parse (`#dept` → `Slot["dept"]`) but do NOT fill
  from an Association argument at apply time; `Select[rows, #dept=="x"&]`
  silently returns `{}` (predicate never True). Kernel Function/Slot
  seam. → brief-274 (last; hot path, microbench A/B gate).
- **F3** `SortBy[rows, {f1, f2}]` multi-key tie-break form not
  implemented — the list is applied as a function, producing inert keys
  compared canonically; often coincidentally "right" (canonical order of
  the whole row = key order), discriminating witness got id-order instead
  of dept-then-id. Silent degradation. → brief-273.
- **F4** `MaximalBy`/`MinimalBy` absent (family sibling TakeLargestBy
  exists). → brief-273.
- **F5** `Query` head absent — WL Dataset query subsystem, out of folk
  scope; NOT pursued this line (future user ruling if ever).
- Parked observation: `#1 &[x]` without parens fails to parse ("trailing
  input"); `(#1 &)[x]` works. Parser precedence wrinkle, not row-critical.

Brief queue (stdlib first, kernel last):

- [x] brief-272 — `JoinAcross` (associations family): Inner/Left/Right/
  Outer, key spec = key | Key[k] | rule | list thereof; hash-join;
  right-wins collisions (matches `Join` on associations); unmatched rows
  carry only their own columns. Reference 638→639. DONE 2026-07-20
  (gate: workspace green, 14 probes reproduced).
- [x] brief-273 — `SortBy` multi-key tie-break form + `MaximalBy`/
  `MinimalBy` (structure_ordering family). Reference 639→641. DONE
  2026-07-20 (gate: workspace green, 15 probes; gate fix = one
  ASCII-sort violation in the attribute mirror insert — the table has
  no sortedness assertion, candidate for a debt note).
- [x] brief-274 — kernel named-slot fill: `Slot[string]` fills from
  `Association` arguments in the existing in-order rule scan (Lookup
  semantics; absent key with an association present →
  `Missing["KeyAbsent", key]`; no association at all → unchanged
  silent-inert, since `substitute_slots` is session-less and a loud
  message would require hot-path signature surgery — revisit only if
  it bites). Hot path (Function application) ⇒ microbench A/B gate.
  DONE 2026-07-20 (gate: workspace 1644/0; A/B 29 faster/6 slower/3
  flat, worst +2.2% module-local = noise floor, string-slot arm only
  by diff; 11 probes incl. cross-brief integration).

## COMPLETE — lazy sequences + sealed scoping line (user ruling 2026-07-18; both halves closed 2026-07-20)

Two parallel WL15-survey outcomes, both design-first, both taken to full
ambition. Sequencing (user 2026-07-18): **(c) first** (new cold symbols,
no core-path change, low risk), then **(d)** (touches symbol resolution,
higher risk). Each batch gates before the next.

**(c) LAZY SEQUENCES — COMPLETE (2026-07-18, briefs 262-267).** All four
batches closed (1 spine, 2a transformers + 2b LazyTable, 3a Part seam +
NestList/FoldList, 3b Subsets/Tuples, 3c Permutations pair, 4 guide).
Twelve lazy heads + the kernel `Part` re-dispatch seam + a folk eager
`Permutations`; reference 621→634; guide `evaluation.md` §Lazy Sequences.
Core asset = the per-element `PullStepper` (source/transformer arms,
ceiling/cancel charged at the source, everything fails closed, never
hangs). **(d) SEALED SCOPING — COMPLETE (2026-07-20, briefs 268-271;
batches below).**

Design docs: [docs/dev/lazy-sequences-design.md](./docs/dev/lazy-sequences-design.md),
[docs/dev/sealed-scoping-design.md](./docs/dev/sealed-scoping-design.md).

**(c) Lazy PURE sequences** — the pure half only; the stateful half
(`IncrementalObject`/`NextValue` cursors, external-source streaming) is
CLOSED WON'T-DO in the kernel (downstream package only if ever pulled).
Rulings: opaque `LazyList` head (not list-transparent); broad v1 set.
Design: `LazyList[<descriptor>]` reserved head (`{HoldAllComplete,
Protected}`, inert value) over a pull-based descriptor tree + Rust pull
interpreter; realizers force to List/element, `Lazy*` transformers stay
lazy; all `Capability::Pure`; realize loop under the iteration ceiling +
cancel (fail-closed on infinite); snapshot round-trips for free
(re-derive, no memo in v1).

- [x] Batch 1 — mechanism (spine): `LazyList` + descriptor tree + pull
  interpreter + `LazyRange` + realizers (`Normal`/`Take`/`First`) +
  opaque semantics + ceiling/cancel + snapshot round-trip. **DONE
  (brief-262 rev 1, baa4ff6):** interpreter reuses real
  `iteration_work_limit` + `interrupt_handle` (fail-closed infinite
  `Normal` in 0.034s; cancel-observing); exact-rational streams; `Take`
  `UpTo` reuse; opaque `_LazyList`/inert `HoldAllComplete` proven;
  snapshot re-derives. Reference 621→623; workspace green; live probes
  reproduced at gate; A/B not runnable (no Take/First/Normal-on-list
  label — single `is_lazy_list` head-check branch). `Rest`/`Part`
  deferred — `Part` is a kernel builtin needing a kernel→stdlib
  re-dispatch seam, `Rest` pairs with `LazyDrop`.
- [x] Batch 2a — transformers `LazyMap`/`LazySelect`/`LazyTake`/
  `LazyDrop` + `Rest` realizer (= `LazyDrop[·,1]`). **DONE (brief-263,
  596bc71):** interpreter refactored to a per-element `PullStepper`
  (Range source + Map/Select/Take/Drop arms) so filtering composes under
  a budget; ceiling/cancel accounting in the source (never-matching
  `LazySelect` fails closed in 0.11s); nested `f[x]`/`pred[x]` errors
  propagate. `LazyTake` = first finite lazy stream (Normal-on-exhaustion
  now exercised). Reference 623→627; workspace green; probes reproduced.
- [x] Batch 2b — `LazyTable[body, {i, …}]`. **DONE (brief-264, 827d6b2):**
  pure desugaring onto 2a primitives —
  `LazyList[LazyTake[LazyMap[Function[i, body], LazyRange[imin, step]],
  count]]`; no new stepper arm, no kernel iteration seam; `HoldAll` body
  bound lexically via `Function` (once per realized element; held-body
  ordering `{0,{1,2,3},3}` proven); empty range → `List[]`; step 0 /
  non-rational / bad spec fail closed. Reference 627→628; workspace
  green. **BATCH 2 CLOSED.**
- [x] Batch 3a — streaming `LazyNestList`/`LazyFoldList` + the kernel
  `Part` re-dispatch seam (`lz[[i]]`). **DONE (brief-265, 0158cbb):**
  `eval_part` on `LazyList` + single positive integer emits
  `self.evaluate(First[LazyDrop[lz, i-1]])` (no reverse crate dep; other
  specs fall through); `LazyNestList` source + `LazyFoldList` transformer
  steppers; nested `f` errors propagate. Reference 628→630; real
  base-vs-candidate A/B = no `part-list` regression (one head-check;
  +3.1% within a +2.4% untouched-control noise band). Known minor:
  out-of-range `lz[[i]]` surfaces `First::nofirst` (fail-closed).
- [x] Batch 3b — combinatorial enumeration sources `LazySubsets`/
  `LazyTuples`. **DONE (brief-266, 482c20b):** size-then-lex
  (next-combination) + mixed-radix (last-fastest) cursors, order-identical
  to eager `Subsets`/`Tuples` (`Normal[Lazy*] === *`); transient cursor,
  descriptor holds only input; `stepper_from_descriptor` re-validates so
  crafted descriptors don't panic; 2^40 space fails closed in ~0.03s;
  bounded `Take` cheap. Reference 630→632.
- [x] Batch 3c — `Permutations` (eager) + `LazyPermutations`. **DONE
  (brief-267 rev 1, 1b4f7a0):** distinct multiset perms in canonical lex
  order; eager + lazy share ONE `next_permutation` helper +
  `compare_expr_canonical`, so `Normal[LazyPermutations[l]] ===
  Permutations[l]` by construction. Eager enforces generated-element +
  memory guards (Range[50]→GeneratedElementLimit; MemoryConstrained→fail);
  lazy charges the source-pull ceiling. Reference 632→634.
  `Permutations[l, {n}]` deferred. **BATCH 3 (broad set) CLOSED.**
- [x] Batch 4 — guide section + reference regen + close-out. **DONE
  (99e9a75):** `evaluation.md` §Lazy Sequences (director-written,
  measured transcripts); reference at 634 (regenerated per batch).
  **(c) LAZY SEQUENCES LINE COMPLETE.**

**(d) Sealed scoping** — SPF scoping half only (loader half = `rum-fs`
downstream, never kernel). Ruling: **private = REAL encapsulation**
(kernel-enforced, not a naming convention); hidden-by-default
introspection; full three-tier; wrapper declaration form
`DefinePackage["`Pkg`", Public[{...}], PackageScope[{...}], body...]`.
Neither line adds a capability. **Grounding landed (brief-268,
e5c9e1c):** design §1 substrate CORRECTED — Module gensyms are
name-mangled (`x$NN`, `identity: None`, source-reachable), so the seal
is a NEW bit-62 identity allocator + `substitute_lexical_bindings`
rewrite + identity kept OUT of `symbol_generations` (resolver
byte-untouched; the miss branch IS the boundary), plus a
`mark_symbol_removed` exclusion (it inherits removed identities into
the generation map). **Snapshot ruling (user 2026-07-19):
snapshot-by-re-registration** — the held `DefinePackage` form is the
unit's snapshot representation (restore re-runs registration, fresh
identities, observably faithful); private post-registration mutations
don't persist in v1 (re-derive); leaked sealed references (opaque-value
pattern) get detect-fail-loudly as the floor and **re-link in the
design as a first-class part** (marker + restore-mode rewrite + stale-
epoch refusal). Rationale calibrated by the mma-hol strict/stable
thought experiment: one mode = boundary stronger than WL-strict +
persistence equal to WL-stable; **dual dividend goes into README.md at
batch-3 close-out (divergence-with-value posture)**.

- [x] Batch 1 (brief-269) — mechanism spine. **DONE (f2e1448):** full
  chain landed exactly per design (bit-62 seal, resolver byte-untouched,
  suspension-safe install mirroring Module, four-site snapshot mirror,
  `mark_symbol_removed` guard); 12 reachability tests; reference
  634→637; workspace 1604/0 (director ran the 9 sandbox-blocked jupyter
  loopback tests out of sandbox); live probes reproduced at gate. A/B:
  3 interleaved pairs + checksum-verified clean rebuilds — NO semantic
  regression (scoping/dispatch tier flat ±5%); a bidirectional
  layout-class movement on seven microsecond list-tier labels
  (+21..63% / five labels faster) recorded as binary layout lottery
  (no diff instruction on those paths; brief-265 precedent), re-observe
  at the next A/B. **Known v1 edge for batch 2 opener:**
  `replacement_index` captured pre-install can go stale if a body
  registers/unregisters other packages — re-locate by context at
  finish/fail + nested-registration test.
- [x] Batch 2 (brief-270) — hidden-by-default introspection. **DONE
  (e9fb37c):** one predicate (bit-62) + one gate (policy allows
  `Diagnostic`) across Names/Contexts/Definition/Information/message
  catalogs/completion/Save-DumpSave call sites (persist helper + host
  snapshot byte-untouched, pinned); NameQ + SymbolName/Context unchanged
  by ruling; host APIs classified trusted. Batch-1 `replacement_index`
  hardening landed (re-locate by context + nested-registration test).
  17 dual-variant tests; workspace 1621/0; A/B clean (bidirectional
  layout scatter only; same-binary +17% cross-session drift observed on
  list-literal-small, confirming the batch-1 layout-lottery reading).
- [x] Batch 3 — DONE. **3a (brief-271 rev 1, 4489c28):** unit-first
  snapshot emission + owned-key skip + sealed-keyed never emitted +
  leaked-reference detection with orphan refusal (= the stale-epoch
  rule, enforced at snapshot time) + `RestoreSealedReferences` re-link
  marker (restore-mode-only, exception-safe flag; reference 638) +
  fallible `snapshot_to_bytes` + Save/DumpSave sealedref refusal +
  Diagnostic-only Information "SealedUnit" row; sealed-free snapshots
  byte-identical. Gate correction: Codex's undeclared process-global
  identity allocator reverted to the session-scoped counter (identities
  are session-scoped; numeric reuse across sessions permitted — the
  brief's cross-restore distinctness assert was the over-pin that
  provoked it). Workspace 1630/0; marker-refusal CLI probe; 37 sealed
  tests + host suites; A/B: 13/14 separated labels FASTER — batch 1's
  layout-lottery labels snapped back to pre-269 levels (bidirectional
  closure of that reading). **3b (director):** guide
  scoping-and-contexts.md §Sealed Packages (three measured
  transcripts) + Host Snapshots addendum; README "Sealed Packages —
  real encapsulation, one mode" divergence section (user ruling
  2026-07-19); design doc final state. **(d) SEALED SCOPING LINE
  COMPLETE (2026-07-20). The lazy + sealed-scoping ACTIVE PLAN is
  fully closed — next direction is a fresh user decision.**

## COMPLETE — cleanup + message-registry line (user ruling 2026-07-17)

Queue, in order; each item gates before the next starts:

1. [x] **Builtin message-tag registry (+ docgen pairing) — COMPLETE
   (briefs 243, 245-249, 252-253; 2026-07-17, one day).** Inventory
   (114 sites, 144 tags, 966 closed pairs) → mechanism
   (`BuiltinSpec.messages` + four-surface merge, user wins, shapes
   frozen) → fills (kernel 304 + stdlib 433/214) → docgen pairing
   (reference renders Messages blocks from the same source) → carriers
   (General 12 + UpTo 2, user ruling; 965 registered pairs, reference
   615 entries) → enforcement (debug assertion at push_message,
   declared-or-exempt rule; zero gaps at landing). Excluded by proof:
   open families + one unreachable ledger row (SetDelayed::argform).
   Follow-up candidates recorded, NOT queued: Take/Drop `UpTo` support
   (WL-conformance gap found by brief-252's stop-loss; DONE in brief-254);
   catalog file splits (DONE 2026-07-18, briefs 260-261: kernel
   namespace.rs → variables/contexts/symbols; stdlib strings/structure/
   associations split by family; all now under threshold, reference
   byte-identical. `rules.rs` 17KB is warning-only, left.
   `stdlib_attributes.rs` was ACCEPTED DEBT then (user 2026-07-18) and
   its debt was CLEARED 2026-07-20 by brief-280 — see Standing
   Engineering Debt. **Catalog split work COMPLETE.**）.
2. [x] **`vm_rhs_template/` — DELETED (brief-244, ruled 2026-07-17).**
   Evidence: the ON path crashed the guard-adjacent recursion test
   (native stack overflow; OFF green, single-variable discriminator);
   flip-ON preconditions from 2026-07-01 never done; only recorded win
   was a closed-era workload. Gate: workspace 1512/0, zero references,
   A/B no regression (two flagged control labels cleared by a 3-pair
   interleaved re-run; downvalue tier flat-to-better).
3. [x] **rum-jupyter `signature.rs` sha256 debt — DONE (brief-250).**
   Hmac<Sha256> replaces the hand-rolled stack (165→42 lines); frozen
   vector test proves equivalence; deps 070a23d. ZeroMQ ordering flake
   did NOT recur (watch stays at two sightings). Side result: first
   workspace-write sandbox dispatch — works; loopback-bind denial makes
   the 9 native ZeroMQ tests sandbox-blocked (deterministic, not the
   flake); the director's out-of-sandbox gate covers them.
4. [x] **rum-cli feature forwarding — DONE (brief-251).** Four-crate
   chain (rum-host/rum-terminal/rum-repl/rum-cli forward `hash` with
   default-features=false deps); defaults byte-equivalent; NDF
   `cargo tree` shows zero digest crates; Cargo.lock unchanged;
   DIGEST_FORMAT cfg follow-up applied.
5. [x] **THE wl15/wfr DESIGN PASS — COMPLETE (2026-07-18, briefs
   254-259).** Three-batch plan approved 2026-07-17; all landed.
   With this item the whole cleanup + message-registry line is CLOSED
   (and the catalog-split follow-up from item 1 is done, briefs 260-261).
   - [x] Batch 1 — Take/Drop/TakeDrop `UpTo[n]` specs DONE (brief-254,
     5a2ef4a): clamp semantics, kernel Part-span endpoint parity;
     gate correction narrowed `UpTo::innf` to definitely-invalid
     contents (negative/rational), symbolic stays silent-inert.
     Workspace 1523/0.
   - [x] Batch 2 — the hold half (design doc `docs/dev/wl15-wfr-hold-design.md`;
     rulings A1/B1/B2/C1 recorded 2026-07-17):
     - [x] 2a mechanism — `SubValuesHoldAll` attribute (brief-255,
       8186536): bit 1<<15, compound-head args prepare as HoldAll via
       the existing base-symbol walk; symbol-head calls untouched.
       Workspace 1528/0; A/B no regression (director 3-pair interleaved).
     - [x] 2b-I mutation operator forms (brief-256, acaf1f3): nine
       compound-assignment heads adopt the attribute + one wrapper
       family; unary captures quiet-inert. Workspace 1537/0; A/B clean.
       Two revisions (stdlib file ownership; kernel handler location).
     - [x] 2b-II schema operator forms (brief-257, 476cf19):
       ConformSchema/SchemaErrors → special_with_hold; capture
       evaluates nothing, application byte-parity with direct call
       (validation-time resolution is pre-existing on both forms).
       Workspace 1543/0; A/B clean. One revision (parity-contract probe).
     - [x] 2c `ReplaceAllUnheld` — separate builtin (brief-258,
       48e909f): evaluates target first, then a structural descent mask
       skips held arg positions (original pre-rewrite head attributes;
       compound-head SubValuesHoldAll honored). ReplaceAll byte-
       unchanged. Workspace 1547/0; A/B replaceall-simple -2.53%
       (clean). Director completed the one vocabulary-mirror line Codex
       stop-lossed on. **BATCH 2 CLOSED.**
   - [x] Batch 3 — structured exceptions (E1-E6 green-lit 2026-07-18,
     all per recommendation). **3a core (brief-259, 42a2476):** new
     `EvalError::ExceptionThrown` channel (separate from Throw/Catch by
     construction — Catch never mentions it); five heads (Exception
     inert + ThrowException/CatchExceptions/RegisterExceptionType/
     ExceptionTypes, stdlib-registered via ControlDataBuiltin); string
     type hierarchy in a session-field registry (snapshot round-trip);
     ancestry dispatch (handler order × nearest ancestor,
     suspension-safe); E4 uncaught → in-band Failure["UncaughtException"];
     E5 registration-required fail-closed; E6 WithCleanup runs / Enclose
     passes through. Workspace 1554/0; reference 621. Two brief revisions
     (both correct Codex stop-losses on my brief omissions).
     **3b guide+close-out:** evaluation.md §Structured Exceptions added
     (director-written, examples measured). **BATCH 3 CLOSED.**
   **THE wl15/wfr DESIGN PASS IS COMPLETE** (batches 1+2+3;
   briefs 254-259).

**WON'T-DO ruling (user 2026-07-17): the parked pools are CLOSED, not
parked.** Stdlib parked candidates (`ByteArray` depth audit, compression
codecs, unicode normalization, ipaddress folk, fuzzing, codec follow-ons)
and the embedding line's parked tail (background-eval helper, per-request
rollback API, lighter checkpoint) are all WON'T-DO; re-open only on an
explicit new user ruling. Standing Engineering Debt entries below remain
as records of known limitations, not scheduled work.

## Feature-Gating Policy (ADOPTED 2026-07-07)

External-dependency stdlib families must not be forced on users who do not
want them (ARCHITECTURE.md §No unwanted dependencies). Rule: pure families
always-on; small pure-Rust dep → default-on Cargo feature; heavy/C-lib dep
→ non-default feature; OS-touching/very-heavy → downstream crate via the
`with_builtin_installer` hook.

- [x] brief-160 DONE (3d60497): feature skeleton in `rum-stdlib`; digest
  family behind default-on `hash` (`sha2`/`sha1`/`md-5`/`crc32fast`
  optional). Hex/base64/ByteArray stay always-on. Reference/drift defined
  at default features; `--no-default-features` builds and tests green with
  none of the digest crates in the dependency graph.
- Measured size context (2026-07-07, local A/B, `rum-cli` defaults):
  plain release 8.53 MiB → +strip 7.24 MiB → +lto+codegen-units=1
  6.19 MiB (env-var profile overrides; repo untouched). For comparison:
  libpython3.14 core 5.26 MiB + 60 MB stdlib dir; Lua ~0.3 MiB (no
  batteries). The candidate follow-up landed 2026-07-17 (brief-251):
  rum-cli forwards `hash` end-to-end and is digest-trimmable via
  `--no-default-features`.
- Compression = downstream `rum-compress` crate, AFTER `rum-fs` (C-library
  deps; never in `rum-stdlib`). Heavy encryption beyond hashing = non-
  default feature or downstream.

## Usability Pass — COMPLETE (2026-07-07 → 2026-07-14, briefs 161-230)

Was the mainline; all 10 sweep units are closed — **605 builtins swept**
(reference 613 after the FileNames restore). Canonical per-unit ledgers,
axes, rules, and close-outs: [docs/dev/usability-pass.md](./docs/dev/usability-pass.md);
memory `usability-pass-line.md`. User directives at open (still in force
for any future builtin work): HIGH coverage (all installed builtins, the
WL-inherited surface included) and compatible enhancement (WL names keep
WL-conformant defaults; improvements land as new options or rum-native
siblings). Method: audit briefs produce all-rows findings tables per
unit; gaps triage into small fix briefs (the 158/159 shape).

**HOST-EMBEDDING LINE — concern (b) CLOSED at b4 (2026-07-15); concern (a)
is the NEXT line, deferred to a fresh session
([docs/dev/host-embedding-line.md](./docs/dev/host-embedding-line.md)).**
Two parallel concerns: (a) the general embeddable-language API judged on
general merits; (b) dun's `pure-sandbox` + protocol conformance.
Sequencing (user 2026-07-14): (b) first, (a) only after (b), scenario-design
first.

- [x] b1 (brief-231, 74afadf): structured termination discriminator.
- [x] b2 (brief-232, ee8b788): request-level memory ceiling.
- [x] b3 (brief-233, b245590): stdout-ownership hygiene hard line.
- [x] b4 (brief-234, a3e5a76): crash-only + per-request isolation contract.
- [CANCELLED] b5 (protocol host prototype): user boundary ruling
  2026-07-15 — building a dun plugin is downstream work, not rum's job;
  the sandbox is hardened to downstream-usable strength at b4. Part 1
  (brief-235 codec crate) was reverted (revert `93a93d2`). b1–b4 reviewed
  boundary-clean (generic sandbox primitives, zero dun/editor naming).

**fs/os/net downstream capabilities: DEFERRED until after publication
(user 2026-07-15) — not the next target.** ACTIVE: concern (a) — the
EMBEDDING API LINE. Scenario-design pass landed 2026-07-16
([docs/dev/embedding-api-line.md](./docs/dev/embedding-api-line.md)):
three user-supplied downstream scenario families (Python/AI-infra, Rust/
wasm tooling, tauri typst IDE = dun's ancestor) + in-tree consumer survey
(findings F1-F8). USER RULINGS 2026-07-16: dogfood = stance A (port
rum-cli/rum-repl onto rum-host; cli first with microbench A/B hard
gate); queue signed off as a1 → a2 → a3 → a4 → a5.

- [x] a1 (brief-236): host failure-classification honesty —
  `HostFailure.exit_status` carries `Exit[code]`;
  `HostFailureCode::requires_reset()` contract method (poison truth
  verified: only the guard's caught-panic path poisons); rum-jupyter
  adopts the b1 termination field (string-match killed; a literal
  `TerminatedEvaluation["Cancelled"]` result value no longer
  misclassifies as cancelled). Workspace 1488/0.
- [x] a2a (brief-237): trusted-host contract surface —
  `HostSession::set_print_sink`/`clear_print_sink` (exclusive streaming,
  not tee; does not survive reset), `eval_script`, `eval_file`
  (FileIo-provider path; provider-less = `Other` failure),
  `queue_input_response` (the intended batch-input channel). Zero
  kernel changes. Status-quo pinned: script suspension surfaces as
  `InputRequired` but is NOT resumable (`resume_with_input` fails
  `Other`; remaining forms not evaluated) — documented, future seam if
  a consumer needs it. Workspace 1494/0.
- [x] a2b (brief-238 + revision): `rum-cli` non-interactive paths
  ported onto `HostSession`, byte-preserving (27 CLI tests frozen,
  zero expectation changes). Fixed the two contract gaps the port
  exposed: `HostValue.text` → CanonicalHoldFormTransparent (canonical
  style, one top-level `HoldForm` peeled — `HoldForm[1+1]` renders
  `Plus[1, 1]`; jupyter text/plain inherits the fix);
  `HostFailure.syntax_rendered` carries the full diagnostic.
  `--parse`/`--format` now go straight to `rum-syntax` (no session).
  REPL branch stays raw until a2c (marked). A/B verdict: NO regression
  — Codex's sequential-run STOP was overturned by the director's
  4-pair interleaved re-test (5 suspect labels cleared, median Δ 0.00%,
  `with-local`'s apparent +14% shown to be label bistability: the same
  candidate binary observed at both levels). Workspace 1495/0.
- [x] a2c (brief-239): `rum-repl` ported onto `HostSession`, output
  bytes frozen (REPL + CLI expectations unchanged). User rulings
  applied: `:lower` → contract `lower_dump` (diagnostic-grade, format
  not a stability promise); `:messages` → contract `message_catalog`
  (reuses `HostMessageTemplate`; per-symbol view stays on inspect).
  `:parse`/`:format` go straight to rum-syntax. rum-terminal is now
  the contract-type render-twin crate (7 dead runtime-typed renderers
  deleted, grep-verified consumer-free). **rum-cli and rum-repl no
  longer depend on rum-runtime/rum-stdlib at all — the in-tree hosts
  are pure rum-host consumers; the a2 dogfood arc is CLOSED.**
  Workspace 1498/0.
- [x] a3 (brief-240): typed value bridge, out — `HostTyped` gains
  `Bool` (True/False convenience mapping), ordered `Association`, and
  immediate `Rule` variants; delayed entries (`:>`, `RuleDelayed`)
  honestly stay `Other` (a data view must not present an unevaluated
  rhs as a value); depth-cap discipline uniform. First real consumer:
  jupyter execute results gain a conservative `application/json` MIME
  (user-approved wire addition — only fully-representable trees:
  Integer/Str/Bool/List/string-keyed deduplicated Association;
  existing three MIME entries byte-unchanged). Workspace 1506/0.
- [x] a4 (brief-241): typed value bridge, in — `expr_from_typed`
  construction (depth-guarded at MAX_TYPED_DEPTH; `Other` rejected as
  output-only; BigInteger/Rational validated; rationals normalize) +
  `HostSession::call(head, args)`, the injection-safe invocation path
  (head + `Symbol` args resolve via the session's real
  `resolve_symbol_name` — short names find System builtins exactly as
  source would; injection pinned by a hostile-string test). One
  kernel seam: `eval_expr_request` = eval_request's post-lower tail
  extracted (behavior frozen, verified). A/B: no regression — Codex's
  interleaved run flagged 3 labels on razor-thin per-label spreads
  (1-11µs); the director's independent interleaved re-run measured
  those labels' true spreads at 5-15% with deltas +0.6~0.9%, median
  0.00%, bidirectional scatter. Workspace 1519/0.
- [x] a5 (brief-242): named profiles, minimal set per user ruling —
  `hermetic()` (determinism contract; shares sandboxed()'s
  implementation today, promises may diverge) + `scripting()` (the
  named trusted-local preset = default()'s semantics; rum-cli adopts
  it). Each of the three named profiles pinned by an
  all-8-capability probe test (a ninth Capability variant now fails
  compilation until every profile takes a stance) + hermetic
  determinism smoke (RandomInteger streams replay across sessions).
  Workspace 1523/0.

**EMBEDDING API LINE (concern a) — COMPLETE (2026-07-16, briefs
236-242).** All five queue items landed and gated in one day. The
in-tree hosts are pure rum-host consumers (cli/repl dropped
rum-runtime/rum-stdlib deps entirely); the typed bridge is
bidirectional and injection-safe; the contract carries exit status,
syntax diagnostics, reset classification, streaming prints,
script/file eval, queued input, lower_dump, message_catalog, and three
named capability-pinned profiles. Canonical doc:
docs/dev/embedding-api-line.md. Next direction DECIDED 2026-07-17: the
cleanup + message-registry line, then the lazy + sealed-scoping line
(both COMPLETE — see the top of this file; fs/os/net remain deferred
until after publication per the 2026-07-15 ruling; the line doc's parked
queue tail is closed WON'T-DO by the same 2026-07-17 ruling).

- [x] brief-161 DONE: strings audit — 101 rows / 99 names, findings in
  `docs/dev/usability-pass/strings-findings.md`; triage + batch queue in
  the tracker (crypto batch BLOCKED on a user decision: hand-rolled
  SHA-256 in `strings/encoding.rs` + toy `Encrypt` vs the
  no-kernel-crypto policy).
- [x] brief-162: correct stop-loss (Listable would change nested/mixed
  inert forms; pinned current behavior in a test). Resolution in the
  tracker: align upward.
- [x] brief-162b DONE (c474840): attribute hygiene — 18 `Protected`
  pattern symbols; `Listable` case conversion with kernel threading
  owning list mapping (net code reduction). Workspace 1342/0.
- [x] brief-163 DONE (b5e8920): crypto surface plan A — toy
  `Encrypt`/`Decrypt`/`EncryptedObject` removed, hand-rolled SHA-256
  deleted, `Hash` onto RustCrypto behind `hash` with
  SHA1/SHA512/MD5/CRC32; reference 602; workspace 1343/0; NDF green.
- [x] brief-164 DONE (e221abc): operator forms for the eight
  WL-documented string builtins via the existing callable-wrapper
  chain; microbench A/B twice — no reproducible delta (noise floor
  13-53% on this machine's sub-ms labels; structural cost = one
  fast-fail branch).
- [x] brief-165 DONE (f762e2d): Overlaps parity + options registration;
  workspace 1345/0. (Watch item: one transient rum-jupyter test failure
  in Codex's first workspace pass, green on rerun — if it recurs, brief
  it together with the signature.rs sha256 debt.)
- [x] brief-166 DONE (559d4ab): text-shaping options + WL forms;
  workspace 1345/0. (Hygiene note from its assess-on-touch:
  `text_shape.rs` in warning range; split boundaries recorded in the
  report — option parsing / format helpers / catalog subfamilies.)
- [x] brief-167 DONE (46f6e96): url/argv/encoding ergonomics; workspace
  1345/0. Gate lesson: reference LIMITS notes live in `docgen.rs` —
  include it in MAY when a brief changes documented limits.
- [x] brief-168 DONE (944a6de): strict modes; workspace 1345/0.
  **UNIT 1 (strings) CLOSED** — deferred/debt ledger in the tracker.
- [x] brief-169 DONE (992e904): unit-2 audit, 89/89; triage + five-batch
  queue in the tracker (operator forms frozen pending perf thaw).
- [x] brief-170 DONE (226bceb): Heads registration for the map/apply
  eight; workspace 1345/0. PERF MORATORIUM LIFTED same day — unit-2
  operator forms thawed as brief-175 (after 171), microbench A/B gate
  restored.
- [x] brief-171+171b DONE (2d018ca): spec flexibility six-pack;
  171's inverted Partition anchoring caught at gate (output-semantics
  review, not just green tests) and corrected in 171b; workspace
  1345/0.
- [x] brief-175 DONE (d1f0b4a): 14 operator forms + wrapper non-Call
  early exit; A/B twice (3+3 Codex, 2+2 independent) — zero regressions,
  early exit IMPROVES symbol-head labels (downvalue-exact-hit -7.1%);
  workspace 1348/0.
- [x] brief-172 DONE (5528ea2): collection comparison controls (seven
  builtins; GatherBy silent-singleton bug fixed); workspace 1351/0.
- [x] brief-173 DONE (5044e0b): fail-closed diagnostics for
  Inner/Dot/MapAt/SubsetMap (crisp error-vs-symbolic seam); 1353/0.
- [x] brief-174 DONE (2249bfd): SelectPositions/Duplicates/BinarySearch;
  workspace 1356/0. **UNIT 2 (structure+functional) CLOSED** —
  ledger in the tracker.
- [x] brief-176 DONE (d87bc97): unit-3 audit, 83/83; healthier family
  (no silent bugs); four-batch triage in the tracker.
- [x] brief-177 DONE (55629d0): comparison controls; workspace 1358/0.
- [x] brief-178 DONE (02750b1): array predicate test/rank forms;
  workspace 1358/0.
- [x] brief-179 DONE (63c1363): ten predicate operator forms (A/B
  clean); OrderedQ[p]/Lookup[key,default] excluded (data-form
  collision); workspace 1360/0.
- [x] brief-180 DONE (828d774): KeyGroupBy/AssociationKeyDeflatten+
  Flatten/MergeNested; workspace 1366/0. **UNIT 3 CLOSED** (271
  builtins swept across units 1-3).
- [x] brief-181 DONE (191a9e6): unit-4 audit, 63/63; triage in the
  tracker. Standout: `Exit` is Capability::Pure but requests host
  termination — USER DECISION pending on the capability relabel.
- [x] brief-182 DONE (5733f2f): rewrite operator forms (five; A/B
  clean); workspace 1367/0.
- [x] brief-183 DONE (1af0a9d): relabel `Exit` from `Capability::Pure`
  to `Capability::Process` — sandboxed() drops it, trusted CLI keeps it
  (Exit[7] still exits 7); rum-host asserts both directions. Workspace
  1368/0. Memory: [[exit-capability-fix]].
- [x] brief-184 DONE (29f5fd3): rewrite options — `ReplaceRepeated`
  `MaxIterations -> Automatic` (per-call runaway budget, clamped to the
  session ceiling; malformed fails closed; safety) + `Replace`
  `Heads -> False` (levelspec head traversal, Cases/DeleteCases parity;
  flexibility). No A/B gate; defaults byte-identical. Reuses the shared
  `parse_heads_option_with_default` helper. Workspace 1371/0.
- [x] brief-185 DONE (f18c17b): `Sum`/`Product` malformed-iterator
  `argform` parity with `Do`/`Table` (mirror the shared `parse_do_spec`
  `None` verdict; inert return byte-identical, only a message added).
  Cross-crate ripple: a rum-stdlib test asserting the old silence was
  updated (necessary, not scope creep). Workspace 1373/0.
- [x] **UNIT 4 CLOSED (2026-07-07):** 181 audit + 182-185. Close-out
  decisions in the tracker: `Equivalent` Flat/OneIdentity REJECTED
  (conservative — WL surface unconfirmed, no oracle per policy, folk
  scope gains nothing, touches planning); `Implies` short-circuit stays
  REJECTED. Separate future candidate: boolean-head attribute hygiene
  (missing `Protected`; `And`/`Or` `{HoldAll}` vs WL — likely
  intentional, own design pass). Units 1-4 = 334 builtins swept.
- [x] brief-186 DONE (unit 5 AUDIT): math families 62/62 →
  `docs/dev/usability-pass/math-findings.md`. Folk-scope held (Factor/
  Together/Expand/rationals-only = OK-intended). Triage + audit
  corrections in the tracker. Key correction: negative/zero args are NOT
  a blanket fail-close — WL DEFINES some (PartitionsP/Q[-1]->0), so the
  fix returns WL's value where defined, messages only where WL errors,
  defers where uncertain.
  USER decision 2026-07-07 (singularities): 1/0, 0^0, (-1)!, zero
  div/mod/mult stay INERT (folk-conservative) — NO domain messages, NO
  special-value arithmetic; only clean WL values fixed. Reshaped the
  fix batches accordingly (arithmetic-singularity fail-closed batch
  DROPPED; FromDigits list range-check = WL-conformant, not a gap).
- [x] briefs 187-192 DONE: 187 argument/arity validation (ceb58b5), 188
  clean WL values Mod[m,0]/PartitionsP-Q[-1] (cb2c173), 189
  Coefficient/CoefficientList Laurent-leak guard (bb41439), 190
  Commonest[list,n]+Quantile[list,{q..}] (e67cfdc), 191 Total level forms
  (95426db), 192 PowerMod modular inverse (1d5e885). Full detail +
  close-out decisions in the tracker (unit-5 section) and
  [[usability-pass-line]].
- [x] brief-193 DONE (54e76d3): attribute hygiene CONSERVATIVE — Protected
  (GCD/LCM/Max/Min/Abs) + NumericFunction (Max/Min/Abs). Zero eval-cost,
  no reordering (FullForm[Max[x,3]] stays "Max[x, 3]"), reductions
  unchanged; runtime attribute mirror synced; reference regenerated.
  Orderless/Flat/OneIdentity REJECTED (no-default-cost). Workspace 1385/0.
- [x] **UNIT 5 CLOSED (2026-07-07):** 186 audit + 187-193. Units 1-5 =
  396 builtins swept. Close-out decisions in the tracker (singularities
  inert per user; Orderless/Flat/OneIdentity rejected; FromDigits list
  form WL-conformant).
- [x] **UNIT 6 CLOSED (2026-07-08):** 194 audit + 195-199 (digest-format
  fail-closed, Protected sweep, option discovery, datetime ergonomics,
  DatePlus operator form). Spun out the **stateful↔pure separation line**
  (briefs 200-202, COMPLETE): `AbsoluteTime[]` = the sole numeric
  wall-clock reader; `FromDateList` = pure converter; `Now` removed;
  `SeedRandom` split → `ReseedRandom[]` + `SeedRandom[seed]`. Standing
  rule: stateful reads are single-purpose symbols gated by a SPEC-level
  capability — never crammed into a pure symbol, never a runtime check.
- [x] **UNIT 7 CLOSED (2026-07-08):** 203 audit + 204-207 (`Protected` on
  all 11 random heads; nested `{dims}` for RandomInteger/RandomChoice;
  weighted RandomChoice, exact rationals; `WhitespaceCharacters` +
  RandomPassword option discovery). Units 1-7 = 446 builtins swept, all
  fixes compat; reference 612. Deferrals in the tracker.
- [ ] **UNIT 8 (kernel A, 55) — AUDIT LANDED (brief-208, c640d8c; 2026-07-12).**
  First kernel unit. Kernel list derived: reference 612 − stdlib catalog 453 =
  159 kernel builtins; unit 8 = 55, leaving 104 for units 9-10. Findings:
  `docs/dev/usability-pass/kernel-a-findings.md`.
  **Headline**: the `Protected` enforcement surface is only 129/612 —
  `Plus[a_,b_] := 42` succeeds and `1+2` then returns `42`; all 55 unit-8
  names lack `Protected`, the protection machinery (`Set`, `Protect`,
  `Unprotect`) included. Enforcement point itself works (`RandomInteger`
  rejects). 208 carries a whole-registry `Protected` coverage section + the
  per-file fix map. Boundary kept honest: `Protected` is a mistake-guard + WL
  conformance, NOT a sandbox boundary (real plugin isolation = the host line).
  - **REAL BUG found (gate reversal, bdb125f):** `ApplyTo[x, f]` where `f`
    protects `x` mid-update lands the write on a visibly `Protected` symbol
    with no message (`x = 101`). Root cause: `store_held_update` writes without
    re-running the protection guard that the check step already uses; `ApplyTo`
    is the one compound assignment whose update (`f[current]`) runs user code
    in the store window. NOTE: my first gate pass wrongly ruled this "false"
    from a too-weak probe — see CLAUDE.md diagnosis lesson.
  - [x] **brief-209 = fix batch 1a DONE (b0c7113, 2026-07-12):** re-run
    `ensure_symbol_not_protected_for_write` at the top of `store_held_update`
    (operator threaded through; Symbol + Part owner) + `ApplyTo` regression
    test. Gate-reproduced: bypass closed, legitimate `ApplyTo` unaffected,
    Part-owner covered. Workspace 1400/0. Not a hot path; no A/B.
  - [x] **brief-210 = fix batch 2 DONE (5872796, 2026-07-12):** whole-registry
    `Protected` sweep, 129/612 → 608/612 (only `$AssertFunction`/`$ContextPath`/
    `$Packages`/`$Path` stay open). One `BuiltinSpec::protected()` const builder
    + `.protected()` per catalog entry (stdlib catalog + kernel builtin_catalog
    + mirror lockstep + regenerated reference). `Plus`/`If`/`Set`/… no longer
    redefinable by accident; WL-conformant + reversible via `Unprotect`.
    Gate: SWEEP-OK (exactly the 4 exclusions unprotected), workspace 1400/0.
    Four tests updated for the new (correct) attribute sets — necessary ripple,
    each value verified empirically (the test-control session's `If`/`Plus`
    keep harness attrs while `Identity`/`HoldAllComplete` fall through to the
    now-protected catalog).
  - [x] **brief-211 = fix batch 3 DONE (d6cbd34, 2026-07-12):** closed the
    three protection-enforcement holes — `$ContextPath`/`$Packages`/`$Path`
    special setters, `SetAttributes`/`ClearAttributes`, and `SetOptions` (the
    last reachable only with a known option; `optnf` had masked it). All reuse
    `ensure_symbol_not_protected_for_write`; the `ClearAttributes[f, Protected]`
    + `Unprotect` escape hatches preserved. Five regression tests + nine
    `Unprotect` ripple edits. Gate: workspace 1405/0, four probes reproduced.
  - [x] **brief-212 = fix batch 4 DONE (87b9067, 2026-07-12):** round-trippable
    value-table assignment. `OwnValues`/`DownValues`/`UpValues`/`SubValues[f] =
    rules` install the rules as `f`'s table (atomic replace). NOTE: the audit's
    "silent false round-trip" (E7) had ALREADY been closed by batch 2's sweep
    (the protected inspector head made it fail loudly); this batch is the
    feature half. New `value_table_assignment.rs` reuses
    `store_definition_target` (matcher-compiling install) +
    `classify_tagged_definition` (head-matching identical to `f /: … := …`).
    Validate-all-then-replace → no partial tables. Cross-symbol copy fails
    closed (head is `f`, not `g`); working form is `(DownValues[f] /. f -> g)`.
    DefaultValues deferred. Gate: workspace 1415/0 (10 new tests); A/B clean —
    unaffected controls moved MORE than the affected path (noise floor above
    the delta).
  - [x] **brief-213 = fix batch 5 DONE (868d2c6, 2026-07-12), closing:**
    (a) wildcard name-pattern targets — `Clear`/`ClearAll`/`Remove`/`Protect`/
    `Unprotect` share one normalizer and the matcher already existed (it is
    what `Names` uses), so one change gave all five pattern support;
    per-target protection preserved, no-match is a silent no-op creating no
    symbol. (b) `Information` gains `{HoldFirst, Protected}` (it used to
    EVALUATE its argument while sibling `Definition` held correctly), the
    builtin catalog summary as its `Usage` fallback, and a `Messages` field.
    Workspace 1418/0.
- [ ] **DEFERRED from unit 8 — builtin message-tag registry.** `Messages[Set]`
  is empty even right after `Set::argx` fires: builtin diagnostics come from
  ~108 ad-hoc `push_message` sites with NO static catalog of a builtin's
  possible tags/templates. Building one is its own project and pairs with
  docgen (the reference should list message tags from the same source).
  Batch 5's new `Messages` field lights up for free once it exists.
- [x] **UNIT 8 CLOSED (2026-07-12):** 208 audit (55/55) + fixes 209-213.
  Units 1-8 = **501 builtins swept**. Protection surface 129/612 → 608/612;
  one real bug found and fixed at the gate (`ApplyTo` protection bypass);
  three enforcement holes closed; value tables made round-trippable.
- [x] **UNIT 9 CLOSED (2026-07-13):** 214 audit (47/47) + fixes 215-222
  (three user rulings: Module error-path reclamation w/ mma-hol gate,
  staged `With`, replacement-Sequence splice). Evaluator hold correctness
  (`Unevaluated` restore, `HoldAll` Sequence splice, `ReleaseHold`
  one-layer). Units 1-9 = **548 swept**.
- [x] **UNIT 10 CLOSED (2026-07-14):** 223 audit (57/57) + fixes 224-230.
  Real loader `FileRead` bypass fixed at the seam; real `Pause[10^100]`
  panic fixed; three capability rulings (Input → `Capability::Interactive`;
  TestReport → Clock kept; FileNames → `FileRead` glob restored). **USABILITY
  PASS COMPLETE — 605 builtins swept, all 10 units closed.** Next: the
  host-embedding line (header above).
- [x] rum-jupyter `signature.rs` hand-rolled `sha256_compress` (found by
  brief-163's sweep) — RESOLVED 2026-07-17 (brief-250): ported onto the
  workspace `Hmac<Sha256>`; see the cleanup + message-registry line, item 3.

## Release Transformation — COMPLETE (2026-07-06)

- [x] The whole line (M0 + briefs 124-131 + M1/M1.5/M2): differential-testing
  harness and external-engine scripts removed (archived privately),
  closed-line docs and historical dev briefs archived, narrative/policy
  documents rewritten (DISCLAIMER, README, AGENTS, testing-strategy,
  ARCHITECTURE, v0.1 boundary), wording scan across crates and kept docs,
  generated builtin reference + drift test, nine-chapter language guide,
  rustdoc + Cargo metadata, the two guide-surfaced bugs fixed (native stack
  guard; ReplaceRepeated limit-as-value), final sweep green, bundle cold
  backup taken. Process docs live in git history and the private archive.
- [ ] Publication — **deferred indefinitely (2026-07-06)**: rum publishes
  only after it has diverged far enough from its notation-family origins.
  Growth happens in rum's own namespace and directions (embedded scripting,
  sandbox safety, rapid script development — the elisp/guile/lua/python
  family). Mechanics (hosting, name, tag, `repository` metadata, CI) decided
  then.

## Stdlib Expansion For Scripting — COMPLETE (2026-07-07)

Evidence base: brief-135 (2026-07-06) wrote six real scripting tasks in
current rum (all six completed, with workarounds) and mapped the
string-pattern surface. Batches, evidence-ranked:

- [x] brief-136 DONE (ba4ad60): collection/text odds-and-ends the tasks reached for —
  `Tally`, `MaxBy`/`MinBy`, `TakeLargestBy`/`TakeSmallestBy`, `ReverseSort`/
  `ReverseSortBy`, `GatherBy`, `Riffle`, exact `Mean`, `StringPadRight`,
  `StringLines`.
- [x] brief-137 DONE (d233622): string-pattern CONTRACT + fixes. The engine already
  supports far more than documented (`~~`, alternatives, blanks, `..`,
  character classes, `Except`, `Shortest`/`Longest`, named binding — all
  work, none cataloged). Catalog + guide section; implement the four inert
  atoms (`PunctuationCharacter`, `StartOfLine`, `EndOfLine`, `WordBoundary`);
  `StringSplit` `IgnoreCase`; fix four bug-like edges (`EndOfString` in
  Replace/Position, empty-string `RepeatedNull` cases, `Shortest[".."]`
  MatchQ). No regex engine — symbolic string patterns are the rum way.
- [x] brief-138 DONE (a067bda): in-language JSON codec, Pure, rum-native names
  (string↔expr; objects→Association, JSON decimals→exact rationals; folk
  scope with documented limits). This is the CORNERSTONE of the codec
  family — it sets the family conventions: `From*`/`To*` named pairs,
  fail-closed parsing (reject loudly what the folk subset does not support,
  never misparse silently), error style, option shapes.
- [x] CODEC FAMILY LINE COMPLETE (2026-07-07, briefs 145-148 + matrix): Codec family after JSON (config-format conversion is a target
  application; one shared value model makes conversion compositional, e.g.
  `ToToml[FromJson[s]]`): `FromIni`/`ToIni` (document one dialect),
  `FromToml`/`ToToml` (after the date/time batch — TOML has first-class
  datetimes), `FromXml`/`ToXml` (symbolic `XmlElement[tag, attrs, children]`
  tree; no DTD), `FromYaml`/`ToYaml` LAST (sane block subset only,
  fail-closed; full YAML is a non-goal). CSV-from-string alongside.
- [x] Hash/encoding family DONE (98da945, brief-142): `Sha256`/`Sha512`/`Sha1`/`Md5` via the
  RustCrypto crates (never hand-rolled; deps allowed by the dependency
  policy's security/format clause; live in rum-stdlib, Pure — hashes are
  deterministic byte functions), `Crc32` via `crc32fast`; self-implemented
  `ToHexString`/`FromHexString`/`ToBase64`/`FromBase64` + `ByteArrayQ` to
  round out the byte toolbox. Hex-string default output; `Md5`/`Sha1`
  limits-noted as non-collision-resistant. Hashing yes, encryption/signing
  never in kernel. One-time network step (cargo fetch) done by Claude.
- [x] brief-139 DONE (8c11068): rum-cli pipeline citizenship (host-side only, zero
  kernel impact): non-TTY stdin enters filter mode (no banner/prompt; `-`
  reads the program from stdin); piped DATA reaches `--eval` scripts as a
  host-bound `$ScriptInput` string; stream discipline pinned by tests
  (results+Print→stdout, messages→stderr, documented exit codes); EPIPE
  from a closed downstream exits quietly; streaming Print flush documented
  as a limitation (candidate follow-up). Payoff with 138:
  `cat a.json | rum-cli -e 'FromJson[$ScriptInput] // …' > out`.
- [x] brief-140 DONE (d80a640): Random family — kernel PRNG in session state with a
  FIXED default seed (Pure by the audit rule; deterministic by default),
  `SeedRandom`/`RandomInteger`/`RandomChoice`/`RandomSample`; entropy tap
  via `Capability::Random` + an `EntropySource` provider in rum-host
  (FileIo-seam pattern); `RandomBytes` for password-grade material with
  honest non-CSPRNG-in-kernel boundaries; no `RandomReal` (rationals only).
- [x] brief-141 DONE (d672a79): date/time folk subset — pure parse/format/arithmetic
  on timestamp values; current-time readers are `Clock`.
- [x] brief-144 DONE (53292ac): Tokenizer mechanism for highlighting and text tooling (S/M):
  `TokenizeWith[rules, str]` — an error-tolerant span driver over the
  (brief-137) string-pattern language: rules as symbolic data
  (`{"class" -> pattern, …}`), longest-match + rule priority, unmatched
  input becomes `"text"` spans, output `{class, {start, end}}` with the
  `StringPosition` 1-based convention; plus `RumTokens[str]` exposing
  rum-syntax's own span/trivia lexer. Grammar DEFINITIONS are rum packages
  (dun highlighting plugins), never stdlib code — no grammar zoo. Codec
  family stays fail-closed/value-oriented; this is the tolerant/span-
  oriented counterpart.
- [x] Perl-scan additions folded into brief-143 (fe77af7) — notes: (2026-07-07; Perl core + CPAN River proxies mostly
  CONFIRMED the plan): `UrlEncode`/`UrlDecode`/`UrlParse` (pure string ops —
  corrects the Python sweep's blanket urllib exclusion; the parse half is
  pure), `ParseArgs[args, spec]` (folk argv→Association, downstream of
  `$ScriptArgs`), tab expand + text-table alignment fold into the textwrap
  batch; `Memoize` NOT needed (the `f[x_] := f[x] = …` idiom covers it —
  add one guide sentence). Ecosystem scans CLOSED: further stdlib surveys
  (Ruby/Go) expected to re-confirm, not extend.
- [x] StringForm mini-audit RESOLVED (brief-143): eager backtick-slot formatting is the contract now. Original note: cataloged but inert for the standard
  placeholder call (StringForm["`1` + `2`", 3, 4] does not evaluate) —
  another semi-dark surface; decide contract (implement or remove).
- [x] Python/Perl tier-1 batch DONE (fe77af7, brief-143) — original sweep notes:
  Python-stdlib sweep candidates (2026-07-06, pure/safe filter over
  docs.python.org/3/library): tier-1 small batch — `textwrap`-style
  `StringWrap`/`StringIndent`/`StringDedent`; `html` escape/unescape;
  `shlex`-style `ShellSplit`/`ShellQuote` (pure string ops; pairs with the
  future rum-os `RunCommand`); pure path-string helpers rounding out the
  existing family (`FileExtension`, `FileBaseName`, `FileNameSplit`); exact
  `statistics` subset (`Median`, `Mode`, `Quantile` — exact rationals).
  Tier-1b algorithms — `graphlib`-style `TopologicalSort` (+ cycle error;
  edges as Rule pairs) and `difflib`-style folk LCS diff
  (`SequenceDiff`/line diff; LongestCommonSubsequence already exists).
  Attach elsewhere: `hmac` folds into the hash brief; UUID
  (parse/format pure + v4 from the PRNG/entropy) attaches to brief-140;
  `$ScriptArgs` attaches to brief-139. Parked with conditions: unicode
  normalization (table weight), compression codecs (after ByteArray audit),
  `ipaddress` folk, glob-match-on-strings, binary search. Excluded by
  design: everything float (`math`/`cmath`/`decimal`/float statistics),
  concurrency, network protocol stacks, GUI, ambient os/io, pickle
  (DumpSave exists), sqlite.
- [x] Multiline string literals DONE (81ff8e5, brief-156): """ escaped +
  \'\'\' raw (TOML-mirror), formatter-verbatim, derivative syntax pinned.
- [x] Schema/validation helpers DONE (48ceecf, brief-155): SchemaErrors +
  ConformSchema — patterns are the type language.
- [x] CLOSED WON'T-DO (user ruling 2026-07-17; was "candidates parked"):
  `ByteArray` depth audit; compression codecs; unicode normalization;
  ipaddress folk; fuzzing; codec follow-ons.
- [x] WL15 survey outcomes (2026-07-07, docs/dev/wl15-core-survey.md) —
  ALL FOUR CLOSED: (a) `SubValuesHoldAll`-equivalent — **DONE**
  (brief-255, wl15/wfr batch 2a). (b) Structured-exceptions line —
  **DONE** (brief-259, wl15/wfr batch 3a). (c) Incremental/lazy
  collections — the PURE half **DONE** (lazy-sequences line, briefs
  262-267); the STATEFUL half (`IncrementalObject`/`NextValue` cursors)
  is CLOSED WON'T-DO in the kernel (downstream package only if ever
  pulled). (d) Structured Package Format: loader half = rum-fs
  territory (never kernel); the scoping half **DONE** (sealed-scoping
  line, briefs 268-271; real-encapsulation private + re-link);
  relevant to dun plugins.
- [ ] WFR survey (2026-07-07, docs/dev/wfr-core-survey.md; idea-level
  absorption only, sources never read): unit-2 batch candidates
  `SelectPositions`/`Duplicates`/`BinarySearch`; unit-3 candidates
  `KeyGroupBy`/`AssociationKeyDeflatten`(+flatten twin)/`MergeNested`
  (config layering — pairs with codecs+ConformSchema);
  `ReplaceAllUnheld` defers to the units-8-10 hold design.
- [ ] Downstream `rum-os` (process/env; post-launch, alongside `rum-fs`):
  process execution is the maximal ambient authority and never enters the
  kernel — kernel only reserves a `Capability::Process` variant (do this in
  a nearby brief). Design sketch recorded: structured
  `RunCommand[{argv…}]` → `<|"exit", "stdout", "stderr"|>` with capped
  capture; argv-first (anti-injection), explicit `RunShell` for shell
  semantics; stdin via `"Input"` option; folk pipelines by materialized
  stdout→stdin, OS-level fd pipes only on demand; child processes wired to
  eval cancel/timeout (kill process group).

## After the Release Cut

- [x] **Toolchain / MSRV — SETTLED (2026-07-05): stay on Rust `1.85`.** It is
  Debian stable's default rustc (zero toolchain burden for adopters and
  distro builds) and the edition-2024 floor, so `rust-version = "1.85"` is
  the public MSRV. Post-`1.85` language features are not used;
  `cargo-llvm-cov` stays at the matching `0.6.21`. If an upgrade is ever
  reconsidered, re-baseline both microbench suites in the same session.
- [x] HARDENING COVERAGE LINE (A-half) COMPLETE 2026-07-07 (briefs 149-152): **Coverage rebuild** (the integration-depth gap left by removing the
  external acceptance corpus): `cargo-llvm-cov`-driven buildout prioritizing
  the host lifecycle (interrupt/cancel, wall-clock timeout, poison recovery,
  `reset()`, snapshot/restore round-trips, typed `HostValue`, capability
  sandbox, `Send`/concurrency), then error/resource-limit paths
  (recursion/iteration/parse-depth/match-depth `TerminatedEvaluation` tags,
  genuine-cycle handling), then a rum-native integration/stress corpus (deep
  recursion, large rewriting workloads, cancellation under load).
- [ ] Filesystem surface = the future downstream `rum-fs` repository (seed
  implementation archived by brief-132): scoped-fs roots first, then a
  coherent text-file set and a folk-scope JSON/CSV subset, registering
  through the kernel API and the `FileIo` provider seam. Nothing returns to
  the kernel; network I/O stays permanently out of it; streams/ports and
  binary I/O deferred (stateful handles interact with snapshot/reset/cancel).
- [x] `rum-host` downstream extension hook DONE (3cf45e6, brief-153):
  `with_builtin_installer` — ordered installers, reset()/clone survival,
  capability dispatch enforced on downstream builtins.
- [ ] Python binding (Python embeds rum): a **separate downstream
  repository** on top of `rum-host` via pyo3, started after publication
  mechanics settle ([docs/python-adapter-plan.md](./docs/python-adapter-plan.md)).
  rum-side obligations only: keep `rum-host` sufficient and publish the
  embedding-cost numbers (item above).
- [ ] Downstream embedder (separate repo, own roadmap): `dun`, a
  ratatui-based terminal editor, plans `rum` as its optional heavy-compute
  plugin engine — plugins as pure computations (input snapshots in, output
  intents out) under per-plugin resource limits, cancel/timeout, clean reset,
  and the `Pure` capability sandbox. Light paths (config, themes) stay
  native; `rum-host` is pulled in only when needed.
- [x] Embedding adoption cost DONE (d30fce2, brief-154): guide §Adoption
  Cost — session 0.09 ms steady / eval 4.5 us / 6.5 MiB stripped delta /
  23 external crates.
- [x] `vm_rhs_template/` fate — RESOLVED: DELETED (brief-244, 2026-07-17;
  see the cleanup + message-registry line, item 2, for the ruling
  evidence). The
  profiling/debug_tools fate is settled: they stay in the kernel,
  runtime-gated, hidden from restricted hosts via the Diagnostic
  capability.

## Standing Engineering Debt

- [x] `builtin_catalog/stdlib_attributes.rs` — DEBT CLEARED 2026-07-20
  (brief-280). Accepted as debt on 2026-07-18 with a "revisit only if it
  approaches the 35KB line" trigger; it grew 28k → 30,747 bytes, the user
  called it, and the trigger fired as designed.
  **The 2026-07-18 entry mispredicted the cost of splitting.** It reasoned
  that a split "would force a lookup-mechanism change (search multiple
  segments, or build+search a Vec)" — the delivered split does NEITHER: a
  `const fn` dispatch on the name's FIRST BYTE selects exactly one of four
  chunks, then does the same single `binary_search_by(name)` as before. No
  OnceLock, allocation, or extra search on the matcher-hot
  `default_stdlib_attribute_set_for_symbol` path.
  The other premise — "no responsibility boundary to cut on" — was true and
  was the KEY: a table whose only structure is its ASCII name order must be
  split along that order, not forced into a family taxonomy. Chunks are
  `<'F'` / `'F'..='M'` / `'N'..='R'` / `>='S'` (parent 8.8k; chunks
  6.8/6.6/4.6/6.3k, all far under threshold, with room to grow).
  Compile-time strength preserved rather than traded down: per-chunk strict
  sortedness + per-chunk byte-range membership over disjoint, exhaustive
  ranges is equivalent to the old single global assertion (disjoint ranges
  make a cross-chunk duplicate impossible; concatenating chunks in range
  order reproduces the original sorted array).
  Gate: all 474 entries proven identical AND identically ordered by static
  extraction from both trees; workspace 1654/0; both assertion kinds
  negatively calibrated — including the failure mode the split newly
  introduces (a correctly-sorted entry filed in the WRONG chunk), which
  fails compilation.
- [x] Matcher `Flat`+`Orderless`+`Optional` — ALL THREE GAPS CLOSED
  2026-07-20 (briefs 275-279). Re-probed at close: flat-wrapped singleton
  candidates exist (`ReplaceList[g[1], g[a_, b_:0] :> {a, b}]` →
  `{{1,0},{g[1],0}}`), the `ReplaceList` under-collection is fixed (the flat
  collect-all omission arm was first-match-shaped — the fix also recovers
  dropped rows for built-in `Plus`/`Times`), and the compound
  sequence-neighbor case is pinned. Zero `#[ignore]` remain in
  `tests/pattern_matching_orderless_optional.rs`.
  **This entry's stated spec was itself repudiated**: the `#[ignore]`'d
  tests were WK-derived expectations, retired as testing-policy fossils by
  brief-276 Part A, so they were never a legitimate target spec under the
  current policy. The line was re-grounded on an independent, WK-free
  defect — `MatchQ[g[1,2], g[a_g, b_]]` was False though `g[g[1], 2]`
  flattens to `g[1,2]` — which is now True.
- [x] `rum-host/src/tests.rs` — DEBT CLEARED 2026-07-20 (brief-281). At
  68,639 bytes it was the workspace's ONLY file over the 35k
  architecture-debt line, and it carried no documented exemption. Split
  by responsibility (not by size or alphabet — unlike brief-280's lookup
  table, this file had real boundaries mirroring the embedding line's own
  increments): `tests/common.rs` (shared helpers) plus
  `capability_profiles.rs` 26 tests / `typed_bridge.rs` 16 /
  `session_lifecycle.rs` 13 / `eval_contract.rs` 12 /
  `editor_surface.rs` 10. Parent `tests.rs` is now 118 bytes of `mod`
  declarations; `lib.rs` needed no change since `mod tests;` resolves to
  `tests/mod.rs` either way. Pure move — zero production code touched.
  Gate: all 77 test names and all 18 helper names proven to be the SAME
  SET before and after by static extraction, with nothing duplicated
  across files; workspace 1654/0.
  WATCH: `capability_profiles.rs` landed at 19,614 bytes — under the
  20k split-plan line but with almost no headroom. Codex deliberately
  left the single-child capability helpers in `common.rs` to keep it
  under. If the sandbox/profile surface grows, split it again along the
  boundary that is already visible inside it: dispatch-blocking
  (`sandboxed_host_config_*`, 10) vs named-profile pins
  (`*_profile_pins_*`) vs capability-policy composition
  (`*_policy_*`, `default_host_config_*`, `trusted_*`).
- [ ] Keep the suspension-safe helper boundary from regressing; continue
  moving helper payloads onto typed continuation values when a flow cannot be
  a `Result<Expr, EvalError>` plus one ordinary suspension frame. Monitor
  display refresh and `ValueQ` rollback stay best-effort unless their
  contract is promoted.
- [ ] Binary snapshot format (deferred, low priority): session persistence is
  textual today, which is fine at current scale; a binary path (direct
  deserialize, no re-parse/re-eval) becomes worthwhile for much larger
  session images.
- [ ] Symbolic `Plus`/`Times` like-term detection is O(n²) per expression
  (bounded upside at small term counts; accepted tradeoff). Revisit only with
  profiler evidence from a real symbolic-algebra workload.
- [ ] Deep `Expr` trees built directly through the Rust API (not through the
  parser, which is depth-guarded) can overflow the stack on drop; iterative
  `Drop` designs all regressed the hot path 20-50% and were rejected.
  Documented host limitation in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
- [ ] Formatter internals: continue splitting planner/builder/render/test
  modules as the syntax surface grows; decide later whether the string
  subsystem earns a crate boundary (do not create one by default).
- [x] REPL rendering frozen with full-output snapshots — DONE 2026-07-20
  (brief-282). The gap was assertion SHAPE, not coverage: the 23 tests in
  `rum-repl/src/tests.rs` already exercised every command surface, but via
  48 `contains(...)` calls against 1 `assert_eq!`, so a reordered command
  list, a deleted help line, or changed indentation all passed. Now every
  test pins its ENTIRE rendered output (`contains(` 48 → 0; 23 full-output
  `assert_eq!`s, expected text captured from real runs, not hand-written).
  No snapshot crate added — plain `assert_eq!`, this crate keeps its three
  path deps. Zero production change, test count unchanged, workspace
  1654/0. Negative-calibrated four ways: a deleted help line, a removed
  indent space, a changed message prefix (Codex), and — the mode the old
  assertions could never catch — REORDERING two help lines with their
  content untouched (Claude, at the gate). No non-determinism carve-outs
  were needed; the nanosecond-stamped history paths never reach the output.
  Consequence to accept knowingly: any INTENDED wording or layout change
  now requires updating the pinned text. That is the point (user ruling
  2026-07-20: freeze all of it), and the update cost is one test run plus
  reviewing the diff.
- [x] DECIDE — RULED AND DELIVERED, line closed 2026-07-21. Was: should evaluation
  tracing be a first-class REPL view? The kernel ALREADY has the capability
  (`builtin_catalog/debug.rs`, `Diagnostic`-gated; `TracePrint[1+1]` works
  today and prints `Plus[1, 1]` then `2`). What is missing is a REPL
  surface: the command set is `:help :parse :format :lower :complete
  :history :inspect :messages`, with no `:trace`. Doing it means designing
  an interaction (paging, folding, filtering by symbol, relationship to the
  `Diagnostic` capability) — real feature work, design-first per project
  convention. Not doing it means declaring the status quo final: users
  write `TracePrint[...]` in the expression themselves.
  User ruling 2026-07-20: DO design it, after the snapshot work above.
  DESIGNED + RULED + BUILT 2026-07-20: design doc
  `docs/dev/repl-trace-view-design.md`, user ruled Option A (thin
  non-interactive view), delivered by brief-283 — `:trace [--form=PAT]
  [--depth=N] [--full] EXPR`, rendered REPL-side from the structured
  `Trace[]` value (never by calling `TracePrint`, since the REPL owns its
  output stream), truncating at 40 lines. Bare-symbol `--form=f` is
  rewritten to `_f` with a notice line, because the kernel filter treats a
  bare symbol as a literal and silently returns `List[]`. 8 new snapshot
  tests, 3 negative-calibrated; workspace 1662/0. Exactly one pre-existing
  frozen snapshot changed (`:help`, one added line) — brief-282's
  snapshots' first real outing, and they caught precisely the intended
  change and nothing else.
  FOLLOW-UP DONE (brief-284): leaves rendered as `Hold[Plus[1, 1]]`; the
  `Hold` wrapper is a `Trace`-return artifact and was noise on every line.
  Peeled exactly one level, in the trace-specific `render_trace_leaf` and
  NOT in the shared `render_typed_text`, with a fallback that renders
  unchanged if the text is not `Hold[…]`-shaped. `:trace (1+2)*3` now
  reads exactly like the kernel's own `TracePrint`. Verified at the gate
  that one-level peeling is the correct rule: `Trace[Hold[1+1]]` yields
  `List[Hold[Hold[Plus[1, 1]]]]`, so the outer wrapper is the trace's and
  the inner is the user's — peeling one keeps the user's own `Hold`.
  Test count unchanged (1662/0), `:help` and every non-`:trace` snapshot
  untouched. Cause was a gap in my brief ("print each leaf's text"), not
  in the execution.
  SUBSTRATE DONE (briefs 285/286, 2026-07-20): `TraceEvent` now stores
  `Expr` instead of `String`, removing a render→parse round trip
  (capture did `to_string()` on every payload; `Trace[]` parsed each back
  via `trace_expr_from_text`, with a silent `Expr::string` fallback).
  `SharedExpr(Arc<Expr>)` makes an `Expr` clone a refcount bump, so the
  round trip bought nothing in-kernel. Rendering moved to the host
  boundary; `HostTraceEvent` keeps its `String` fields, which is what
  left `rum-jupyter` and `rum-terminal` untouched. Corpus byte-identical
  (the round trip turned out to BE the identity on everything tested).
  Gate-reproduced A/B: tracing overhead 5.5-5.8× → 2.4× (traced −60.3%
  / −58.4%); untraced moved bidirectionally (−9.15% / +1.45%) = noise,
  no regression. 1662/0.
  **Codex caught a flaw in my brief**: `push_evaluation_pass_trace` had
  NO early-return guard, so my "populate unconditionally" instruction
  would have paid two `Arc` clones per evaluation pass with tracing off
  — a real default-path regression. It added the guard, with a
  `#[cold] #[inline(never)]` slow-path split. Verified equivalent at the
  gate: `handle_monitor_trace_event` opens with exactly the same
  condition, so the new guard skips only work that was already a no-op.
  brief-285 stop-lossed cleanly first (my 4th scope omission this line):
  the six `push_definition_stored_trace` callers live in
  `definition_tags.rs` / `definitions.rs`, unlisted.
  LAYER 1 + REPL DONE (briefs 287/288, 2026-07-21): `TraceDetail` keeps
  all four fields `Trace` discards (`symbol`/`kind` from
  `DefinitionStored`, `source`/`rule` from `RuleApplied`) plus the tree,
  emitting an Association per step (design Option A). Key named
  `RuleLeft`, not `Rule`: every emission site passes the rule's LHS only,
  so `Rule` would promise a whole rule it never contained — renamed while
  the surface had no consumers. `:trace --rules` surfaces it in the REPL,
  composing with `--form`/`--depth`/`--full` for free since `TraceDetail`
  mirrors `Trace`'s argument surface. Reference 642; workspace 1675/0.
  The headline capability, pinned by test: with `f[x_]`, `f[x_Integer]`
  and `f[2]` all defined, it names the one that actually fired.
  NEXT AXIS (documented, out of scope so far): design doc §1.5 records a
  second, host-level trace channel (`HostTraceEvent`, incl.
  `RuleApplied{source, input, rule, result}`) that can name WHICH RULE
  fired — something `Trace[]` cannot do. Using it per-invocation needs
  `rum-host` to forward the kernel's `Session::enable_tracing` toggle,
  i.e. an embedding-API change.
  **Filed here rather than under Standing Engineering Debt on purpose** —
  that section is a record of known limitations, and mixing "unfinished
  work" with "undecided product question" is what let the previous
  one-line entry mislead a later reader.
