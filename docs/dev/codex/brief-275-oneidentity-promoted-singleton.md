# Brief 275 — OneIdentity promoted-singleton matching + Plus/Times head defaults

Implementation brief. First package of the matcher OneIdentity/Optional
line (TODO.md §ACTIVE PLAN, user-approved plan item 2): make the WL
coefficient idioms (`x_ + y_.`, `c_. x_^n_.`) work. Two independent
mechanisms, both grounded by code-reading on 2026-07-20:

1. The matcher's OneIdentity "promoted singleton" path exists only for
   single-argument patterns; and
2. `Default[Plus]` / `Default[Times]` are unregistered, so dot-optionals
   on arithmetic heads have no implicit default to fill with.

## Goal

Any expression matches a `OneIdentity`-headed call pattern whose
non-matched arguments are all omissible, by promotion to a one-element
argument list (WL semantics); `Default[Plus] = 0` and
`Default[Times] = 1` are seeded at install. `cargo fmt --check` and
`cargo test --workspace` green; microbench A/B shows no hot-label
regression (the matcher IS a hot path); the deliberate behavior change
is exactly the one documented below, nothing else.

## Current behavior (evidence, probed 2026-07-20 on release rum-cli @ 59d4f73)

```
MatchQ[5, x_ + y_.]                    → False        (WL: True)
5 /. x_ + y_. :> {x, y}                → 5            (WL: {5, 0})
7 /. x_*y_. :> {x, y}                  → 7            (WL: {7, 1})
Default[Plus]                          → Default[Plus]   (unregistered, inert)
Default[Times]                         → Default[Times]  (unregistered, inert)

SetAttributes[h, {Flat, OneIdentity, Orderless}]; Default[h] = 0;
  MatchQ[5, h[x_, y_.]]                → False        (proves the matcher gap is independent of Default registration)

MatchQ[a + b, x_ + y_.]                → True         (Call-headed subjects with matching head work today — regression pin)
a + b /. x_ + y_. :> {x, y}            → List[b, a]   (current binding order — regression pin, do NOT change)
```

## Grounding (code, read 2026-07-20 — verify, then build on it)

`crates/rum-runtime/src/runtime/pattern_matching/core.rs`,
`match_compiled_call_owned` (~line 190):

- The `one_identity` / `orderless` / `flat` attributes are computed up
  front from the PATTERN head (line ~197).
- The non-Call arm (`other if one_identity`, ~line 242) handles ONLY
  `[single]` pattern-argument lists via
  `match_compiled_arg_and_rest_owned(single, [], vec![other], …)`; any
  other arity is a hard False — optional omission is never consulted.
- The Call arm matches the value head first and returns unmatched on
  head mismatch — there is no OneIdentity fallback, so
  `MatchQ[f[a], x_ + y_.]` is False today (WL: True, x → f[a], y → 0).
- Dot-optional defaults resolve through
  `resolve_optional_implicit_default`
  (`crates/rum-runtime/src/runtime/definitions/application/default_values.rs`),
  which reads DOWNVALUES OF THE `Default` SYMBOL — `Default[Plus] = 0`
  seeding is pure data through the existing machinery; no new mechanism.
  The `Default` symbol carries no attributes (assignment is open).

## Specification (rum's own documented behavior; WL-conformant)

### A. Matcher: promoted-singleton matching

- In `match_compiled_call_owned`, when the pattern head has
  `OneIdentity`:
  - **Non-Call subjects** (the existing `other if one_identity` arm):
    for ANY pattern-argument arity, dispatch through the SAME
    orderless/flat argument machinery as the Call arm, with the value
    list `vec![other]` (and the flat-head expression derived from the
    literal pattern head). The existing `[single]` behavior must fall
    out unchanged as the special case.
  - **Call subjects whose head does NOT match the pattern head**: after
    the head match fails, if the pattern head has `OneIdentity`,
    rebuild the subject expression and retry through the same
    promoted-singleton dispatch (value list = `vec![subject]`).
    A successful head match keeps today's path exactly.
- Gating and cost rules:
  - The promoted path applies only when the pattern head is a LITERAL
    symbol head (the attribute lookup already requires this — if the
    head `MatcherNode` cannot produce a literal head expression, keep
    today's behavior).
  - ZERO added work on the head-matched path: no clone, no branch
    beyond what exists. The subject rebuild (`Expr::call(head, args)`)
    happens only on the head-mismatch + OneIdentity fallback.
  - Do NOT change `MatcherNode`'s representation or the compile side.
- Semantics that must hold (WL-conformant, and each becomes a test):
  - `MatchQ[5, x_ + y_.]` → True; `5 /. x_ + y_. :> {x, y}` → `{5, 0}`.
  - `7 /. x_*y_. :> {x, y}` → `{7, 1}`.
  - `MatchQ[f[a], x_ + y_.]` → True with x → `f[a]`, y → 0.
  - `MatchQ[5, x_ + y_]` (no default) → False (promotion does not
    invent arguments).
  - `MatchQ[5, x_. + y_.]` → True (all-optional).
  - `c_. x_^n_.`-style compounds work on atoms: probe
    `7 /. c_. * b_^e_. :> {c, b, e}` and pin the actual (sensible)
    result in a test; if the nested promotion (Power inside Times)
    does NOT come out WL-shaped, STOP-LOSS on that single case, keep
    the simple cases, and REPORT it — do not restructure the matcher
    to chase it.
  - Kernel-session tests set `Default[h]` manually (kernel test
    sessions do not run stdlib install).
- Regression pins (existing behavior that must NOT move):
  - `a + b /. x_ + y_. :> {x, y}` stays `List[b, a]`.
  - The six currently-passing tests in
    `pattern_matching_orderless_optional.rs`.
  - The two `#[ignore]` tests in that file stay ignored and UNTOUCHED
    (they are brief-276's subject).

### B. Install-time head defaults

- Seed `Default[Plus] = 0` and `Default[Times] = 1` during stdlib
  install (`crates/rum-stdlib/src/lib.rs`, `install_core`). Preferred
  mechanism: a small end-of-install seeding step through the session's
  EXISTING public evaluation/definition surface (e.g. evaluating the
  two assignments); if you find an existing Rust definitions-API seam
  that is cleaner, use it — but do NOT add a new public kernel API for
  this. If neither works without new surface, STOP and report.
- After install: `Default[Plus]` evaluates to `0`, `Default[Times]` to
  `1`; both remain user-overridable and Unset-able exactly like any
  `Default[f]` assignment (WL-consistent; no protection added).
- One snapshot round-trip test: snapshot a fresh session, restore, and
  verify `Default[Plus]` is still `0` and `5 /. x_ + y_. :> {x, y}`
  still fires (guards against double-seeding or restore clobber).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/pattern_matching/core.rs` (the two
    seams + small private helpers in the same file);
  - `crates/rum-stdlib/src/lib.rs` (install seeding only);
  - `crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`
    (ADD test functions only; do not touch the existing six passing
    tests or the two `#[ignore]` tests);
  - `crates/rum-stdlib/src/tests/incidental_definitions_and_rules_patterns.rs`
    (end-to-end Plus/Times idiom + install-seeding + snapshot tests).
- Files/areas you MUST NOT touch: everything else — in particular the
  matcher compile side (`matcher_and_symbols.rs`), the
  `sequence/` matchers, `default_values.rs`, `docs/**`,
  `docs/dev/codex/**`, `.git`, `Cargo.lock`. If you believe you must,
  STOP and report.
- Do NOT add unconditional cost to the default evaluation path.

## Deliverable

- The two matcher seams + the install seeding.
- Tests per Specification (kernel + stdlib sides).
- Microbench A/B evidence: BEFORE editing any source, build the base
  release `rum-cli` and copy it to `/tmp/brief275_base_rum-cli`; after
  the change build the candidate; one
  `scripts/core-language-microbench.sh` run per binary (`--rum-cli`,
  separate `--output-dir`s); paste the per-label `rum_seconds` rows for
  the matcher-flavored labels (`matchq-*`, `replaceall-simple`,
  `downvalue-pattern-*`) AND their same-tier neighbors, state the
  direction mix. Note: microsecond-tier labels are layout-lottery
  sensitive (±60% both directions is noise); judge by direction mix +
  adjacent-tier flatness.
- The report per Report format below.

## rum / Rust pitfalls (read twice)

1. **The matcher is hot.** The head-matched path must be byte-identical
   in work done. Any new allocation/clone belongs exclusively to the
   fallback (head-mismatch + OneIdentity) and promoted-atom paths.
2. **Suspension safety**: the argument matchers run under
   `drive_internal_value` continuations — route the promoted dispatch
   through the same entry points the Call arm uses
   (`match_compiled_orderless_args_with_flat_head_owned` etc.); do not
   inline or eagerly collapse them.
3. **`Expr` clones are Arc-level** — clone handles, never deep trees.
4. **Install seeding must be deterministic and silent** — no messages,
   no dependence on evaluation order beyond install's end.
5. **Stop-loss is real.** Same step fails twice for the same reason →
   STOP and report. The nested-compound case (`c_. * b_^e_.`) has an
   explicit single-case stop-loss above.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace        # known caveat: loopback-bind ZeroMQ tests may fail under the sandbox; if so, run and paste the two supplemental splits (--workspace --exclude rum-jupyter; -p rum-jupyter --lib -- --skip zeromq::tests::native) and say so
```

Microbench A/B per the Deliverable. Then paste the verbatim output of
these probes on the CANDIDATE release build (feed stdin from
/dev/null):

```
target/release/rum-cli --eval 'MatchQ[5, x_ + y_.]'
target/release/rum-cli --eval '5 /. x_ + y_. :> {x, y}'
target/release/rum-cli --eval '7 /. x_*y_. :> {x, y}'
target/release/rum-cli --eval 'MatchQ[f[a], x_ + y_.]'
target/release/rum-cli --eval 'MatchQ[5, x_ + y_]'
target/release/rum-cli --eval 'a + b /. x_ + y_. :> {x, y}'
target/release/rum-cli --eval 'Default[Plus]'
target/release/rum-cli --eval '7 /. c_. * b_^e_. :> {c, b, e}'
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all
  changes in the working tree; Claude runs the authoritative gate and
  commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- Touch NOTHING outside this repo (except `/tmp/brief275_base_rum-cli`
  and the microbench output dirs under `/tmp`), no network. The only
  commands you run are file edits within Scope, `cargo`, the release
  `rum-cli`, the microbench script, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's
  own documented behavior; no black-box comparisons against other
  vendors' engines, no oracle harnesses, no third-party engine outputs.
- **No new `unsafe`.**
- **Hot-path changes require the microbench A/B gate.** A regression on
  any hot label is a defect to fix or report, not to ship.
- **Probe budget**: one full microbench run per binary; if a number
  looks off, REPORT it — do not silently re-measure.
- Minimal diff: no drive-by reformatting, renames, or comment changes
  outside the task.
- You MUST paste the real verbatim verification output. If a run did
  not reach green / produce the evidence, say so explicitly — never
  fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command run, with the exact verbatim output
   lines (test totals, the A/B rows with direction mix, the probe
   outputs).
3. The finding / verdict — semantics implemented vs the spec, the A/B
   reading, the nested-compound case outcome, any deliberate deviation.
4. Stop-loss / open questions — where you stopped and why (empty if
   none).
