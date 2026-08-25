# brief-326 — robustness against malformed and extreme input

## Goal

Find inputs that make the process die, hang, or consume unbounded resources
instead of returning a value or a message. This is a **diagnostic survey**: a
ranked list of reproducible cases with `path:line` attribution and drafted
implementation briefs — not a code change.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes with
the release `rum-cli` are in scope; leave no probe files in the repository.
Probes must feed stdin from `/dev/null` — the CLI waits for input otherwise.

A second, independent survey of this same question is being run separately. Do
not try to match it; two derivations compared afterwards are worth more than one
confirmed twice.

## What counts as a finding

rum's documented contract is that evaluation either returns a value or reports a
message, and that resource ceilings refuse work gracefully. A finding is an
input that breaks that contract:

1. **Process death.** A native stack overflow, an abort, or a panic escaping to
   the process. `exit=-6`, `exit=134`, `thread 'main' has overflowed its stack`,
   or any `panicked at` reaching the top level.
2. **A hang.** Evaluation that neither completes nor hits a documented limit.
   Distinguish this from slow: if it finishes at a smaller size and the growth
   is a curve, it belongs in the complexity survey, not here.
3. **Unbounded resource use.** Memory or handles growing without a ceiling
   refusing the work — including a ceiling that exists but is never reached
   because the growth happens somewhere it does not measure.
4. **A limit that reports the wrong thing.** A guard that fires but names an
   unrelated cause, or a message whose text points the user at the wrong fix,
   is a real defect: it costs a user more than a clear failure would.

Ordinary rejected input that produces a clear message is **not** a finding.

## Ground already covered — do not re-report

Thirteen builtins used to abort on wide input and were made iterative:
`Replace`, `ReplaceAll`, `Cases`, `Count`, `Position`, `DeleteCases`,
`ReplaceList`, `Reap` rendering, `UpValue`/`SubValue` dispatch, `StringReplace`,
`StringCases`, `StringApply`. The parser's depth guard was split into delimiter,
operator-chain and statement-chain caps. `Module` bindings without initializers
were made iterative.

Known and already recorded, so report only if the assessment is wrong:

- `Module` bindings **with** initializers still recurse and abort near 8000 —
  found by the parallel complexity survey, already confirmed.
- Deep `Expr` trees built through the Rust API can overflow on drop; three
  iterative `Drop` designs regressed the hot path 20-50% and were rejected.

## Where to look

Weight the effort by what a user can actually feed the system:

- **Source text**: deeply nested or pathological syntax, very long lines,
  unterminated constructs, CRLF, unusual whitespace, control characters,
  malformed escapes, invalid or truncated UTF-8, a BOM, an empty file.
- **Numbers**: enormous exponents, huge rationals, zero and negative arguments
  where a count is expected, values near integer type boundaries, and anything
  that turns a size argument into an allocation.
- **Structures**: very deep nesting built by evaluation rather than by parsing
  (the parser's caps do not cover it), self-referential definitions, wide
  argument lists, empty containers where one element is assumed.
- **Recursion and mutual recursion** through every path that can call back into
  evaluation: patterns with conditions, delayed rules, format and print,
  comparison and sorting predicates, and the codec decoders.
- **The codecs** (`FromToml`, `FromJson`, `FromYaml`, `FromCsv`, `FromIni`) on
  truncated, deeply nested, and adversarially shaped documents.
- **Resource ceilings themselves**: what happens exactly at, and one past, each
  limit; whether a limit that refuses one operation leaves the session usable.

Prefer breadth here: a single reproducible abort is worth more than a careful
account of an input that merely returns a message.

## What each finding must carry

1. **The input**, minimized — the smallest form that still reproduces.
2. **The observed failure**, verbatim, including the exit code.
3. **Attribution** — `path:line` for the code that fails, and why it fails
   there.
4. **The contract it breaks** — which of the four categories above, and what the
   documented behavior would be.
5. **Reachability** — can this arrive from an ordinary script, or only through
   the host API? Both matter; say which.
6. **Whether an existing guard should have caught it**, and why it did not.

Rank by (severity × reachability). A crash reachable from a plain script
outranks a crash reachable only by building a pathological tree through the
Rust API.

**Report the negative cases too** — inputs you expected to break something and
that behaved correctly. That list is what stops the next survey re-walking the
same ground.

## Deliverable

1. The ranked finding table.
2. The negative list.
3. A batching plan: coherent, independently gateable batches, ordered by
   severity.
4. Drafted implementation briefs in `TEMPLATE.md` shape — raw material for the
   director to rewrite, not to dispatch.
5. Open questions.

## Pitfalls (read twice)

- **Minimize before reporting.** An unminimized case wastes the implementation
  brief's budget and often hides the real trigger.
- **Distinguish slow from stuck.** Measure at two sizes before calling something
  a hang.
- **A guard that fires is not a bug** — unless it names the wrong cause.
- **Findings age.** 103 commits landed recently; confirm behavior on the current
  tree rather than on any earlier report.
- **Do not fix anything.** A diagnosis that names the wrong cause costs one
  dispatch; a fix aimed at the wrong cause costs a gate cycle and a diff to
  unwind.

## Hard rules

- No source changes. No commits, branches, pushes, or git config changes.
- Touch nothing outside this repo; no network. Commands: file reads, grep/find,
  `python3`, the release `rum-cli`, and `cargo` only if a counter needs a build.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Every claim carries a reproduction or a `path:line`. A claim you could not
  confirm is reported as unconfirmed, not asserted.

## Report format (your final message)

1. Method — what you probed, how, and what you deliberately skipped.
2. The ranked finding table.
3. The negative list.
4. The batching plan and drafted briefs.
5. Open questions for the director.
