# Brief 231 — structured termination reason on successful eval responses

Implementation brief — host-embedding line, item b1 (see
`docs/dev/host-embedding-line.md`, concern (b) gate 5). When a top-level
evaluation is ended early by a budget (wall-clock timeout, a cancel
handle, recursion/iteration/work limits), the session deliberately
returns a NORMAL value — the synthesized form
`TerminatedEvaluation["Cancelled"]` / `TerminatedEvaluation["TimeConstraint"]`
/ etc. — so partial results and messages survive. That in-band design
stays. The defect is ergonomic: the response carries no structured field
saying "this value was synthesized by the runtime because the evaluation
was terminated, for THIS reason", so an embedder (the rum-host API and
every adapter above it) must string-match the rendered value — and a
user-constructed `TerminatedEvaluation["Cancelled"]` literal is
indistinguishable from the real thing. Add the structured field end to
end: runtime response → host success type. No WL-visible behavior
changes: every rendered value, message, and exit path stays byte-
identical; the change is additive response metadata.

## Mechanism (pre-diagnosed; follow these seams)

1. The single reason source is
   `Session::terminated_evaluation_reason_for_error`
   (`crates/rum-runtime/src/runtime/session/session_helpers.rs:211`) —
   9 reasons: RecursionLimit, StackLimit, PatternMatchRecursionLimit,
   Cancelled, TimeConstraint, IterationLimit, GeneratedElementLimit,
   IntegerWorkLimit, IterationWorkLimit.
2. The conversion points are `finalize_top_level_eval_value` and
   `finalize_cycle_termination_value`
   (`crates/rum-runtime/src/runtime/session/session_eval.rs:22-56`):
   they turn the error into the `TerminatedEvaluation[reason]` value and
   RETURN it as `Ok` — the reason is known there and then discarded.
3. Callers that build responses from the converted value:
   - `eval_request` (`session/session_requests.rs:124-158`)
   - `resume_eval_request_with_input` (`session_requests.rs:187-223`)
   - `eval_text_with_terminated_handling` → `EvalOutcome`
     (`session/session_eval.rs:186-224`), aggregated by
     `eval_script_with_terminated_handling` (`:240-281`) and reaching
     `eval_script_request` / `eval_file_request`
     (`session_requests.rs:226-266`).
4. Two in-kernel consumers already pair the value with the reason STRING
   (`debug_tools/evaluation_data.rs:65-91`,
   `debug_tools/test_report/evaluation.rs:189-192`) — they must keep
   producing the exact same strings.

## Required change

1. **Reason enum (runtime).** In `runtime/model/requests.rs` (next to
   `SessionEvalResponse`), add a `Clone, Copy, Debug, PartialEq, Eq`
   enum `EvalTerminationReason` with the 9 variants named exactly as the
   current tags: `Cancelled`, `TimeConstraint`, `RecursionLimit`,
   `StackLimit`, `PatternMatchRecursionLimit`, `IterationLimit`,
   `GeneratedElementLimit`, `IntegerWorkLimit`, `IterationWorkLimit`;
   plus `pub fn tag(self) -> &'static str` returning the current string
   for each. Re-export it wherever `SessionEvalResponse` is re-exported
   (model `mod.rs`, crate root) so `rum-host` can import it.
2. **Reason source returns the enum.**
   `terminated_evaluation_reason_for_error` returns
   `Option<EvalTerminationReason>`;
   `terminated_evaluation_expr_for_error` builds the expr from
   `reason.tag()`. Adapt the two debug_tools consumers via `.tag()` —
   their rendered output must stay byte-identical (existing tests pin
   this).
3. **Finalize helpers report the reason.** Change
   `finalize_top_level_eval_value` and
   `finalize_cycle_termination_value` to return the reason alongside the
   value on the conversion path (e.g.
   `Result<(Expr, Option<EvalTerminationReason>), EvalError>`), `None`
   on the ordinary path. No hidden session field; thread it explicitly.
4. **Response fields.** `EvalOutcome` and `SessionEvalResponse`
   (`runtime/model/requests.rs:57,94`) each gain
   `pub termination: Option<EvalTerminationReason>`. Semantics: the
   field describes the RETURNED `value` — `Some(reason)` iff that value
   is the runtime-synthesized `TerminatedEvaluation[...]` from this
   request's finalize step. In `eval_script_with_terminated_handling`,
   overwrite the running field with each form's outcome so the final
   field matches the final value (an earlier form's termination is
   already visible through its `General::*` message; do not change that
   behavior). `eval_request` / `resume_eval_request_with_input` /
   `eval_script_request` / `eval_file_request` populate the field from
   the threaded reason.
5. **Host mirror.** In `crates/rum-host/src/model/outcome.rs`, add a
   documented `HostTerminationReason` mirror enum (same 9 variants,
   `From<EvalTerminationReason>` impl, follow the `HostFailureCode`
   mirroring style) and give `HostEvalSuccess` a documented
   `pub termination: Option<HostTerminationReason>` populated in
   `eval_success_from_runtime`. Export the new type from `lib.rs`
   (crate has `#![warn(missing_docs)]` — document both additions).
6. **Construction ripple.** Update every struct-literal construction of
   the three widened types: `session_eval.rs` (2× `EvalOutcome`),
   `session_requests.rs` (4× `SessionEvalResponse`),
   `outcome.rs` (`HostEvalSuccess`), and the test helper in
   `crates/rum-terminal/src/lib.rs:248` (`termination: None`). If other
   literal constructions surface in test code, set the honest value and
   list each in the report.

## Tests (all in existing test files)

- `rum-host/src/tests.rs`: the cancel test (~`:1150`) additionally
  asserts `termination == Some(HostTerminationReason::Cancelled)`; the
  timeout test (~`:1183`) asserts `Some(...::TimeConstraint)`; the
  generous-timeout success test (~`:1200`) asserts `None`.
- New assertions/tests (rum-host or runtime request tests, your pick —
  name them in the report):
  a. an ordinary eval (`1 + 1`) has `termination == None`;
  b. evaluating the LITERAL source `TerminatedEvaluation["Cancelled"]`
     yields Success with `termination == None` (provenance: only the
     runtime's own synthesis sets the field);
  c. a recursion-limit eval (e.g. `f[x_] := f[x + 1]; f[0]`) yields the
     in-band `TerminatedEvaluation["RecursionLimit"]` value AND
     `termination == Some(...::RecursionLimit)`.

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/model/requests.rs`
- `crates/rum-runtime/src/runtime/model/mod.rs` (re-export only)
- `crates/rum-runtime/src/lib.rs` (re-export only, if needed)
- `crates/rum-runtime/src/runtime/session/session_helpers.rs`
- `crates/rum-runtime/src/runtime/session/session_eval.rs`
- `crates/rum-runtime/src/runtime/session/session_requests.rs`
- `crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs`
- `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs`
- `crates/rum-host/src/model/outcome.rs`, `crates/rum-host/src/model/mod.rs`,
  `crates/rum-host/src/lib.rs`
- `crates/rum-host/src/tests.rs`, `crates/rum-host/src/lifecycle_contract_tests.rs`
- `crates/rum-terminal/src/lib.rs` (the `SessionEvalResponse` test helper only)
- test files that construct the widened structs literally (list each)

MUST NOT touch: the synthesized `TerminatedEvaluation[...]` value shape
or any rendered text, `HostFailureCode` and the failure path, message
emission, the evaluator/dispatcher, `docs/**`, anything else.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: the finalize seam runs once per top-level form,
  not per evaluation step; state this in the report. Do not add any
  per-step work.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- Additive metadata only: if any existing test needs its EXPECTED value
  or rendered text changed (beyond adding the new field to a literal or
  a new assertion), STOP and report — that means WL-visible behavior
  moved, which this brief forbids.
- Stop-loss: if threading the reason through the finalize helpers forces
  edits outside the MAY list, STOP and report the file list instead of
  expanding scope.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Test survey — each new/extended assertion, named.
3. Verification — the exact commands with verbatim output (test totals).
4. The no-A/B justification (one sentence).
5. Deviations / open questions (empty if none).
