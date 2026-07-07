# Brief 184 — Rewrite options: `ReplaceRepeated` `MaxIterations` + `Replace` `Heads` (unit-4 batch 2)

Implementation brief. Executes unit-4 triage batch 2 (tracker
`docs/dev/usability-pass.md`, unit-4 section; evidence E7/E8/E10 in
`docs/dev/usability-pass/control-logic-rewrite-findings.md`). Two
independent, compat-preserving option additions in the rewrite family.
NEITHER is call planning (no operator-form seam) — this is options
registration + trailing-option parsing + traversal/loop plumbing, so
**NO microbench A/B gate is required**. Default call forms stay
byte-identical.

The options-defaults table (`options/defaults.rs`) is a DISCOVERY
surface only — it feeds `Options[...]`/`OptionValue`/`SetOptions`, and
`system_option_bool` reads the registered/`SetOptions` value. Explicit
trailing options passed in the CALL are parsed by the builtin itself
(the `Cases` precedent: `system_option_bool("Cases","Heads",false)`
seeds the default, then the trailing `Heads -> b` in the call overrides
it). Both changes below mirror that two-source pattern.

## Change 1 — `Replace` gains `Heads -> False` (flexibility)

Today `Replace[expr, rules, levelspec]` takes 2–3 args and rejects a
trailing `Heads` option: `Replace[f[1], f -> g, {0, Infinity},
Heads -> True]` → `Replace::argtu: expected between 2 and 3
argument(s), got 4` (Evidence E8), so head replacement under a
levelspec is unreachable. `Cases`/`DeleteCases` already expose `Heads`.

1. In `crates/rum-runtime/src/runtime/options/defaults.rs`, add
   `Replace` to the `Heads -> False` default group (the same
   `bool_option(false)` group `Cases`/`DeleteCases`/`Count`/`MemberQ`/
   `Level`/`Depth` already sit in). This makes `Options[Replace]`
   report `{Heads -> False}`.
2. In `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`,
   accept trailing option rules after the (optional) levelspec.
   Partition the post-`rules` args: an arg whose head is `Rule` or
   `RuleDelayed` with a recognized option-symbol LHS (`Heads`) is an
   option; at most one remaining positional arg is the levelspec
   (unchanged path). Seed `include_heads` from
   `system_option_bool("Replace", "Heads", false)`; an explicit
   trailing `Heads -> b` overrides it. Keep the existing 2–3-arg
   positional behavior working; the arg-range/error helper must still
   fire for genuinely malformed calls.
3. Thread `include_heads` through `replace_owned` /
   `replace_call_children_owned`: when true, the head is ALSO a
   replacement target (mirror `collect_cases_in_standard_call` in
   `cases/collect.rs`, which pushes the head into `children` when
   `traversal.include_heads`). When false (the default) the traversal
   is byte-identical to today — the head is never visited.

Acceptance (Evidence E8, verbatim behavior):
- `Replace[f[1], f -> g, {0, Infinity}, Heads -> True]` → `g[1]`.
- `Replace[f[1], f -> g, {0, Infinity}]` → `f[1]` (default unchanged —
  head not traversed).
- `Cases[f[1], f, {0, Infinity}, Heads -> True]` → `{f}` (untouched —
  reference only, do not regress).

## Change 2 — `ReplaceRepeated` gains `MaxIterations -> Automatic` (safety)

Today `ReplaceRepeated[expr, rules]` is exactly 2 args; a trailing
`MaxIterations -> n` is rejected: `ReplaceRepeated[a, a -> a + 1,
MaxIterations -> 2]` → `ReplaceRepeated::argx: expected exactly 2
argument(s), got 3` (Evidence E7), and `Options[ReplaceRepeated]` is
empty, so a script cannot lower the per-call runaway budget below the
session ceiling.

1. In `options/defaults.rs`, register `MaxIterations -> Automatic` for
   `ReplaceRepeated` (`system_expr("Automatic")` value). This makes
   `Options[ReplaceRepeated]` report `{MaxIterations -> Automatic}`.
2. In `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs`,
   accept a trailing `MaxIterations -> v` option (change the exact-2
   check to "2 positional args, plus an optional trailing
   `MaxIterations` option rule"). Compute the effective limit against
   the session ceiling `self.resource_limits.replace_repeated_limit`:
   - no option, or `Automatic`, or `Infinity` → the session ceiling
     (behavior UNCHANGED — this is the sole path today);
   - a positive integer `n` → `min(n, ceiling)` — a per-call LOWER
     budget; it may tighten but MUST NOT raise above the safety ceiling;
   - a non-positive / non-integer / otherwise malformed value (and not
     `Automatic`/`Infinity`) → FAIL CLOSED: emit an invalid-option
     message via the existing invalid-call helper (mirror
     `invalid_call_with_arg_form`) and return the unevaluated call. Do
     NOT silently fall back to the ceiling — a bad budget must not read
     as "no limit".
3. Thread the computed effective limit into
   `resume_replace_repeated_loop` in place of the hard-coded
   `self.resource_limits.replace_repeated_limit` reads. Reaching it
   raises the SAME `EvalError::IterationLimitExceeded { limit }` as
   today, with `limit` = the effective value.

Scope note (do NOT expand): this makes only the limit VALUE
controllable. rum keeps its existing error-on-limit shape; matching
WL's partial-result-plus-`rrlim`-message-on-overflow is a SEPARATE
behavior decision and is OUT of scope here.

Acceptance (Evidence E7):
- `ReplaceRepeated[a, a -> a + 1, MaxIterations -> 2]` → limits at 2
  (the `IterationLimitExceeded` surfaces `limit` 2), not the session
  default.
- `ReplaceRepeated[a, {a -> b, b -> c}]` → `c` (default path unchanged).
- `ReplaceRepeated[a, {a -> b, b -> c}, MaxIterations -> Automatic]` →
  `c` (Automatic == the session default; well within budget).

## Tests

Add rum-runtime tests in the rewrite test module(s)
(`crates/rum-runtime/src/runtime/tests/structure_data/replace.rs` and/or
the sibling cases/rewrite modules — put each test with its family):

- Replace `Heads -> True` traverses the head (`Replace[f[1], f -> g,
  {0, Infinity}, Heads -> True]` → `g[1]`); the default (`Heads`
  omitted / `Heads -> False`) leaves the head untouched (`f[1]`).
- `Options[Replace]` reports `{Heads -> False}`;
  `Options[ReplaceRepeated]` reports `{MaxIterations -> Automatic}`.
- ReplaceRepeated: an explicit lower `MaxIterations` is honored (limits
  earlier than the session default); `Automatic` equals the default
  path; the ceiling clamp holds (a `MaxIterations` above the session
  ceiling does not raise the effective limit above the ceiling); a
  malformed `MaxIterations` value fails closed (message + unevaluated
  call, NOT a silent unlimited run).
- Keep every existing `Replace`/`ReplaceRepeated` test passing
  unchanged (arity messages for genuinely malformed calls, levelspec
  behavior, `//.` shorthand, the session-limit default).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# the two evidence probes, post-fix:
printf '' | cargo run -q -p rum-cli -- --eval '{Replace[f[1], f -> g, {0, Infinity}, Heads -> True], Replace[f[1], f -> g, {0, Infinity}], Cases[f[1], f, {0, Infinity}, Heads -> True]}'
printf '' | cargo run -q -p rum-cli -- --eval '{Options[Replace], Options[ReplaceRepeated], ReplaceRepeated[a, {a -> b, b -> c}], ReplaceRepeated[a, {a -> b, b -> c}, MaxIterations -> Automatic]}'
printf '' | cargo run -q -p rum-cli -- --eval 'ReplaceRepeated[a, a -> a + 1, MaxIterations -> 2]'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added,
all existing rewrite tests pass); `Replace[..., Heads -> True]` → `g[1]`
while the default → `f[1]`; `Options[Replace]` → `{Heads -> False}`,
`Options[ReplaceRepeated]` → `{MaxIterations -> Automatic}`;
`ReplaceRepeated[a, a -> a + 1, MaxIterations -> 2]` limits at 2.

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`,
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs`,
  `crates/rum-runtime/src/runtime/options/defaults.rs`,
  and the rewrite test files under
  `crates/rum-runtime/src/runtime/tests/**` (new tests; and updating any
  existing `Options[Replace]`/`Options[ReplaceRepeated]` golden that now
  lists the registered default).
- MUST NOT touch: `Cases`/`DeleteCases`/`MatchQ`/`ReplaceAll`/
  `ReplaceList` behavior or their operator forms; the `Replace` /
  `ReplaceRepeated` DEFAULT (no-option) call results; the guide,
  `docs/reference/builtins.md` (options are not in the generated
  reference — no regen), `TODO.md`, the tracker, findings files,
  `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve the suspend/resume continuation shape of
  every `eval_*`/`resume_*` path you touch (do not collapse a
  `suspend_with_frame*` into an eager call).
- Compat: default call forms (`Replace` without `Heads`,
  `ReplaceRepeated` without `MaxIterations`, `//.`) are byte-identical
  to today. The session `replace_repeated_limit` stays a hard safety
  ceiling — a per-call `MaxIterations` can only LOWER the effective
  budget, never raise it.
- No microbench A/B gate needed (options/traversal, not call planning),
  but the default traversal path must not gain unconditional cost: the
  `include_heads` head-visit is a single branch guarded by a flag that
  defaults false.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file, line ranges; the trailing-option parse and
   the head-traversal / effective-limit plumbing; the precedent
   mirrored.
2. Verification — all commands verbatim (test totals + the three
   probes + `git status --short`).
3. Stop-loss / open questions (empty if none).
