# Brief 274 — named-slot fill from Association arguments (kernel)

Implementation brief. Final package of the row-workload pure line
(TODO.md §ACTIVE PLAN, audit F2): named slots parse correctly
(`#dept` → `Slot["dept"]`) but never fill from an `Association`
argument, so `Select[rows, #dept == "x" &]` silently returns `{}`. The
fill machinery is one kernel function; this brief extends it,
association-aware and purely syntactic.

## Goal

`Slot[name]` substitution consults `Association` arguments (in-order,
alongside the existing bare-rule scan) with `Lookup`-consistent
absent-key behavior, per the exact spec below. No message machinery, no
evaluator-callback, no signature change to `substitute_slots`. `cargo
fmt --check` and `cargo test --workspace` green, and the microbench A/B
shows no hot-label regression (function application is a hot path; the
changed arm must be reached only by string slots).

## Current behavior (evidence, probed 2026-07-20 on release rum-cli @ 2a5e663)

```
(#dept &)[<|"dept"->"x"|>]        → Slot["dept"]        (WRONG for the row workload: association never consulted)
Select[rows, #dept == "x" &]      → List[]              (the audit-F2 silent trap; rows = list of associations)
(#k &)["k"->1]                    → 1                   (bare-rule fill — PRESERVE)
(#k &)[k->1]                      → 1                   (symbol-LHS bare rule — PRESERVE)
(#k &)["j"->1, "k"->2]            → 2                   (in-order scan across args — PRESERVE)
(#1 &)[5]                         → 5                   (numbered slots — MUST stay byte-identical)
(#dept &)[5]                      → Slot["dept"]        (no rule/association source at all — PRESERVE as-is)
(#k &)[<|"k"->1|>, "k"->9]        → 9                   (today: association skipped; CHANGES to 1 under the new in-order rule — deliberate)
```

## Specification (rum's own documented behavior)

All changes live in `lookup_named_slot` (and, if needed, small private
helpers beside it) in
`crates/rum-runtime/src/runtime/substitution/slot_substitution.rs`.

- `Slot[name]` (string) resolution scans the function's arguments IN
  ORDER; for each argument:
  - a bare `Rule`/`RuleDelayed` call: match its LHS against `name`
    (Symbol-by-name or String-by-value — the existing logic, unchanged);
  - NEW — an `Association`-headed call: scan ITS `Rule`/`RuleDelayed`
    entries with the SAME LHS matching rule; a hit returns that entry's
    value. Non-rule entries inside a malformed association are skipped.
  - Anything else: skipped (as today).
  The FIRST hit in this in-order walk wins. (This deliberately changes
  the mixed case pinned above: an earlier `Association` now shadows a
  later bare rule.)
- If the walk finds NO hit but saw at least one `Association`-headed
  argument: substitute `Missing["KeyAbsent", name]` (two string-atom
  args; `Lookup`-consistent — `Lookup[<|"a"->1|>, "b"]` already returns
  exactly this shape).
- If the walk finds no hit and saw no association: return `None` —
  the slot stays unsubstituted, exactly today's behavior.
- The substitution stays PURELY SYNTACTIC: no `Session`, no evaluation,
  no messages, no allocation on paths that do not resolve a string slot.
  Numbered `Slot`/`SlotSequence` arms must be untouched.

## Context pointers

- Read `AGENTS.md` (Testing Policy hard lines) first.
- `crates/rum-runtime/src/runtime/substitution/slot_substitution.rs` —
  `substitute_single_slot` (the `("Slot", String)` arm calls
  `lookup_named_slot`) and `lookup_named_slot` itself. The whole change
  fits here.
- `crates/rum-runtime/src/runtime/scoping/callable.rs` — the only
  caller of `substitute_slots`; read for context, DO NOT modify.
- Existing named-slot lowering tests:
  `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs`
  (`Function[Slot[\"x\"]]` around line 223) — put the new fill-semantics
  tests beside them.
- `scripts/core-language-microbench.sh` — the A/B harness
  (`--rum-cli PATH` accepts a prebuilt binary; per-label
  `timings.tsv`).
- Acceptance is mechanical: the named tests and the A/B gate decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/substitution/slot_substitution.rs`;
  - `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part2.rs`
    (add tests; do not rewrite existing ones).
- Files/areas you MUST NOT touch: everything else — in particular
  `scoping/callable.rs`, `substitute_slots`' public signature, the
  lowering, any stdlib crate, `docs/**`, `docs/dev/codex/**`, `.git`,
  `Cargo.lock`. If you believe you must, STOP and report.
- Do NOT add unconditional cost to the default evaluation path: the
  Integer-slot arms and the non-slot traversal must not gain a branch,
  an allocation, or a clone.

## Deliverable

- The `lookup_named_slot` extension per spec.
- Kernel tests covering AT MINIMUM (eval-level through a session, plus
  substitute-level if the file has that idiom): association fill
  (string key); symbol-key association fill; `RuleDelayed` entry inside
  an association; absent key with an association present →
  `Missing["KeyAbsent", name]`; no association and no rule → slot stays
  unsubstituted (pin today's behavior); bare-rule fills preserved (the
  three pinned probes above); the mixed-args shadowing change (the
  `<|"k"->1|>, "k"->9` case now → 1); numbered-slot regression pins
  (`#1`, `##` untouched).
- The microbench A/B evidence: BEFORE editing any source, build the
  base release `rum-cli` and copy it to
  `/tmp/brief274_base_rum-cli`; after the change build the candidate;
  run `scripts/core-language-microbench.sh` once per binary
  (`--rum-cli` flag, separate `--output-dir`s); paste the per-label
  `rum_seconds` rows for every function-application-flavored label and
  its same-tier neighbors, and state the direction mix. Note for
  reading: the microsecond-tier list labels are layout-lottery
  sensitive (±60% swings both directions are noise); judge by
  direction mix + adjacent-tier flatness, not single magnitudes.
- The report per Report format below.

## rum / Rust pitfalls (read twice)

1. **This is THE hot path's neighborhood.** `substitute_slots` runs on
   every pure-function application. The new work must be reached ONLY
   from the `("Slot", String)` arm; do not restructure the match, the
   traversal, or the numbered arms.
2. **No owned-tree clones**: return `rule_args[1].clone()` (Arc-backed
   `Expr` clone) exactly like the existing code; build the `Missing`
   call only on the miss-with-association path.
3. **Purely syntactic**: no session access, no evaluation of association
   entries — a `RuleDelayed` value substitutes verbatim, like the
   existing bare-`RuleDelayed` path.
4. **One variable per measurement**: the A/B compares base vs candidate
   with profiling env OFF; if a label moves, check its same-tier
   neighbors before concluding anything.
5. **Stop-loss is real.** Same step fails twice for the same reason →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace        # known caveat: loopback-bind ZeroMQ tests may fail under the sandbox; if so, run and paste the two supplemental splits (--workspace --exclude rum-jupyter; -p rum-jupyter --lib -- --skip zeromq::tests::native) and say so
```

Microbench A/B per the Deliverable (base binary copied to
`/tmp/brief274_base_rum-cli` BEFORE the first edit).

Then paste the verbatim output of these probes on the CANDIDATE
release build:

```
target/release/rum-cli --eval '(#dept &)[<|"dept"->"x"|>]'
target/release/rum-cli --eval 'rows = {<|"id"->1,"dept"->"x"|>, <|"id"->2,"dept"->"y"|>, <|"id"->3,"dept"->"x"|>}; Select[rows, #dept == "x" &]'
target/release/rum-cli --eval 'Map[#id &, {<|"id"->1|>, <|"id"->2|>}]'
target/release/rum-cli --eval '(#zzz &)[<|"dept"->"x"|>]'
target/release/rum-cli --eval '(#dept &)[5]'
target/release/rum-cli --eval '(#k &)[<|"k"->1|>, "k"->9]'
target/release/rum-cli --eval '(#k &)["j"->1, "k"->2]'
target/release/rum-cli --eval '(#1 &)[5]'
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all
  changes in the working tree; Claude runs the authoritative gate and
  commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- Touch NOTHING outside this repo (except `/tmp/brief274_base_rum-cli`
  and the microbench output dirs under `/tmp`), no network. The only
  commands you run are file edits within Scope, `cargo`, the release
  `rum-cli`, the microbench script, and `python3` for parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's
  own documented behavior; no black-box comparisons against other
  vendors' engines, no oracle harnesses, no third-party engine outputs.
- **No new `unsafe`.** `rum-runtime` is ZERO-`unsafe`; keep it that way.
- **Hot-path changes require the microbench A/B gate** — a regression on
  any hot label is a defect to fix or report, not to ship.
- **Probe budget**: one full microbench run per binary; if a number
  looks off, REPORT it — do not silently re-measure everything.
- Minimal diff: no drive-by reformatting, renames, or comment changes
  outside the task.
- You MUST paste the real verbatim verification output. If a run did not
  reach green / produce the evidence, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command run, with the exact verbatim output lines
   (test totals, the A/B per-label rows with direction mix, the eight
   probe outputs).
3. The finding / verdict — semantics implemented vs the spec, the A/B
   reading, any deliberate deviation (should be NONE without a
   stop-loss).
4. Stop-loss / open questions — where you stopped and why (empty if
   none).
