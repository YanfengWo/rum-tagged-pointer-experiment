# Pair/Simp Hotspot Probes

These files are external diagnostics for the unmodified `mma-hol` Pair/Simp
performance gate. They are not acceptance tests and must not be copied into
`mma-hol` as a workaround.

## Discipline

Do not choose the next probe because it is small, easy to run, or has readable
profile output. Start from the current acceptance root: the unmodified
`Pair.wl` load or the warm public mkPair probe that reproduces the gate. First
measure that root with profiling off, then split it into child phases whose
times close over the parent wall time. Descend only into the largest closed
child segment, and record the child percentage before drilling further.

If a probe does not have a closed parent segment, it is side-lane evidence. Keep
it for later, but do not let it redefine the Pair/Simp frontier. The
one-case ``HOL`Bool`EXCLUDEDMIDDLE[p]`` probes are an example: they show a real
local `PropTaut` constant-factor gap, but they were not proven to own the
hundreds-of-seconds `Pair.wl` gate.

Run probes through the bounded wrapper script:

```bash
scripts/pair-simp-hotspot-probes.sh --only basic-only-pair4 --timeout 120
```

The script runs from the `mma-hol` checkout root, so the probe files can use
relative `Get` paths while staying outside the downstream tree. The default
checkout is `/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol`; override it
with `--mma-hol PATH` or `RUM_MMA_HOL_ROOT`.

For `runtime-profile-*` probes, the runner enables runtime profile
environments by default and defers collection until `RumProfileReset[]`.
Evaluation-cache power-of-two incremental logs are disabled by default in this
runner path; set `RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=1` if those
checkpoints are needed. Lexical source summaries are enabled by default so
presence scans can be attributed to the active definition source.
Evaluation-cache snapshots also include call-stability counts grouped by
active definition source and call head, plus invalidation site/family summaries
for symbol writes, dirty-bloom saturation, and call-generation bumps.
Use the low-disturbance category/count mode while comparing target timings:

```bash
RUM_PROFILE_LEXICAL_DETAIL=1 \
RUM_PROFILE_LEXICAL_SHAPES=0 \
RUM_PROFILE_LEXICAL_BINDINGS=0 \
scripts/pair-simp-hotspot-probes.sh --only runtime-profile-opened-body-user-basic --timeout 240
```

For structural identity diagnostics, add
`RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS=1`. That emits source/target/shape
fingerprint groups for large presence scans. To diagnose expression rebuild
and rule-RHS fresh-local churn, enable shape summaries with
`RUM_PROFILE_LEXICAL_SHAPES=1`; that also emits text/exact expression rebuild
groups and `rule-rhs-alpha-rename` counts. These modes are intentionally
heavier than the timing-comparison mode and should be used only when the
question is tree-reuse or fresh-identity churn.

For provenance diagnostics, add `RUM_PROFILE_LEXICAL_PROVENANCE=1`. This emits
`lexical provenance` rows that classify large presence scans by target origin,
scanned-expression origin, and whether the scanned expression's rule-RHS alpha
ids are older than the exact fresh local target. This is a diagnostic probe for
the Pair/Simp `arg_prefilter` lane, not an optimization knob.

For rule-substitution and evaluator-dispatch diagnostics, use the narrower
opt-in timers instead of enabling heavy shape summaries:

- `RUM_PROFILE_RULE_RHS_TIMING=1` prints inclusive rule-RHS substitution and
  rebuild phase timings.
- `RUM_PROFILE_DEFINITION_RULE_TIMING=1` prints inclusive timings by matched
  DownValue index, useful for distinguishing branches such as `tyMatch`
  `tyVar` vs. `tyApp`, or `stripOrigin` `abs` vs. leaf cases.
- `RUM_PROFILE_BUILTIN_TIMING=1` prints builtin handler timings, including a
  `builtin-self-timing` ranking with child builtin time subtracted.

To get a no-profile timing baseline from a `runtime-profile-*` file, set the
profile environment variables to `0` before invoking the runner. The probe's
`RumProfileReset[]` / `RumProfileSnapshot[]` calls are then effectively inert,
but the same target expression is still measured by `AbsoluteTiming`.

Current probe roles:

- `basic-only-pair4.wls`: completing failed-dispatch hotspot for basic rules.
- `pair4-stock-repeat8.wls`: stock repeated basic-only shape aligned with the
  older `/private/tmp/rum-task2d-basic-pair4-stock.wl` probe.
- `rhs-body-branches-user-basic.wls`: opened RHS body logical left/right child
  split for the outer `And` form.
- `rhs-body-and-phase-user-basic.wls`: manual `doAndCong` split into left
  child, right child with left context, theorem plumbing, and original
  `descendOnce`.
- `rhs-opened-body-entry-phase-user-basic.wls`: manual `simpFixpointConv`
  entry split into `buildBaseConv`, top rewrite, and `descendOnce` phases.
- `rhs-opened-body-user-basic.wls`: full opened RHS body split.
- `runtime-profile-opened-body-user-basic.wls`: preheat, reset runtime profile
  counters, run opened-body `simpFixpointConv`, and snapshot counters.
- `runtime-profile-preheat-basic.wls`: reset before `basicSimpset[]`, run only
  the basic simpset preheat, and snapshot counters.
- `runtime-profile-preheat-congruence.wls`: run `basicSimpset[]`, reset, run
  the three lazy congruence lemmas, and snapshot counters.
- `runtime-profile-imp-cong-lemma.wls`,
  `runtime-profile-and-cong-lemma.wls`, and
  `runtime-profile-or-cong-lemma.wls`: isolate each lazy congruence lemma.
  These establish that congruence preheat is three same-shaped `PropTaut`
  theorem-construction runs, not Simp wrapper overhead.
- `runtime-profile-proptaut-imp-cong-schema.wls`: hand-build the implication
  congruence formula and run it directly through `PropTaut`, bypassing the
  lazy Simp lemma wrapper.
- `runtime-profile-excluded-middle-boolvar.wls`: bare
  ``HOL`Bool`EXCLUDEDMIDDLE[p]``; the smallest retained one-case-split
  `PropTaut` side-lane hot shape, not the active Pair root unless a closed
  parent phase tree proves congruence preheat dominates.
- `runtime-profile-proptaut-boolvars-p-or-notp.wls`: minimal
  `propTautBoolVars[p ∨ ¬p]` one-variable tautology, useful for splitting
  `caseSplit` versus `EXCLUDEDMIDDLE`.
- `runtime-profile-proptaut-free-bool-vars-schema.wls`: repeated
  `propFreeBoolVars` scan over the congruence schema, used to rule variable
  collection out as the main `PropTaut` owner.
- `runtime-profile-tymatch-var-repeat.wls` and
  `runtime-profile-tymatch-fold-repeat.wls`: `tyMatch` branch micro-probes for
  `tyVar` and recursive `tyApp` / `Fold` shapes.
- `runtime-profile-loop-null-repeat.wls`,
  `runtime-profile-module-frame-repeat.wls`, and
  `runtime-profile-fold-callback-repeat.wls`: loop, trivial `Module`, and
  trivial `Fold` callback baselines for interpreting repeat-probe overhead.
- `runtime-profile-striporigin-abs-repeat.wls`,
  `runtime-profile-typeof-abs-repeat.wls`, and
  `runtime-profile-vsubst-abs-repeat.wls`: abs-recursive term walker
  micro-probes.
- `runtime-profile-outer-abs-inner-user-basic.wls`: preheat, reset counters,
  run the inner `simpFixpointConv` reached by the outer abs branch, and
  snapshot counters.
- `runtime-profile-three-abs-descend-user-basic.wls`: preheat, reset counters,
  run `descendOnce` on the three-abs term, and snapshot counters.
- `rhs-lambda-user-only.wls`: RHS lambda descent with only the user `mkPair`
  rule.
- `rhs-lambda-basic-only.wls`: RHS lambda descent with only basic rules.
- `rhs-lambda-abs-phase-user-basic.wls`: manual split of the outer abs branch
  into open/BETA, inner `simpFixpointConv`, `ABS`, and original `descendOnce`.
- `rhs-lambda-depths-user-basic.wls`: RHS lambda descent after opening zero to
  four lambda binders, to expose binder-level cost multiplication.
- `rhs-lambda-inner-entry-phase-user-basic.wls`: manual `simpFixpointConv`
  split for the target obtained after opening the outer lambda once.
- `rhs-lambda-user-basic.wls`: RHS lambda descent with user plus basic rules.
- `user-basic-pair2.wls`: long/timeout public-shape reproducer for
  `mkPair[x, y]` with user plus basic rules.

Timeout probes may have empty stdout under `rum-cli`, because `Print` output is
collected and emitted only after evaluation returns. Treat a timeout as a
bounded non-completion result, not as phase-level evidence unless the probe has
been split into completing phases.
