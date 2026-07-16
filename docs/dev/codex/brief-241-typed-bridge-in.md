# Brief 241 — typed value bridge, in: expr construction from HostTyped + HostSession::call (injection-safe typed invocation)

Implementation brief — embedding API line, item a4 (see
`docs/dev/embedding-api-line.md`, finding F1: the only way to inject
host data today is source-string concatenation — escaping hell and an
injection surface; rum-cli smuggled argv through the input-suspension
machinery until a2b). Add the input half of the typed bridge:
`HostTyped → Expr` construction with a depth guard, and
`HostSession::call(head, args)` — build the call expression directly
and evaluate it through the full request pipeline. One small kernel
seam is required and authorized: a request-level entry that accepts an
already-built `Expr`.

## Pre-diagnosed facts (verified this session)

- `Symbol::new(context, name)` is public (`rum-core/src/expr.rs:109`);
  `Symbol` Display = `qualified_name()` which prints the SHORT name
  for Global/System symbols (`expr.rs:174-180`) — so out-direction
  `HostTyped::Symbol` strings are short names for those contexts.
- `Session::resolve_symbol_name(&self, raw: &str, create_if_missing: bool)
  -> Option<Symbol>` is already public (`runtime/resolution.rs:101`) —
  THE symbol-resolution seam. Use it for the `call` head and for
  in-direction `Symbol` values so a short name resolves exactly as it
  would in source (context path finds System builtins; probe and pin
  a fully-qualified name too).
- `Session::evaluate(&mut self, expr: Expr)` exists
  (`resolution.rs:292`) but is BELOW the request protocol (no
  checkpoint/termination/message aggregation). Do not build `call` on
  it.
- `Session::eval_request(&str)` (`session_requests.rs:108-158`) is
  parse → lower → [checkpoint/message bookkeeping → eval → finalize →
  response assembly]. The bracketed tail is what `eval_expr_request`
  must share.
- Out-direction conversion caps at `MAX_TYPED_DEPTH = 64`
  (`rum-host/src/model/outcome.rs:8`). The in-direction guard reuses
  the same constant — a tree built under this cap is far below any
  drop-recursion danger (the standing deep-`Expr`-drop limitation stays
  documented and untriggered; do NOT touch `Drop` machinery).

## Required change

### Part A — kernel seam (`rum-runtime`, minimal)

1. **`Session::eval_expr_request(&mut self, expr: Expr) -> Result<SessionEvalResponse, SessionRequestFailure>`**
   — extract the post-lower tail of `eval_request` into a shared
   private helper; `eval_request` becomes parse + lower + shared tail;
   `eval_expr_request` is the shared tail applied to the given expr.
   `eval_request`'s behavior must stay byte-identical (existing tests
   + the Part D A/B pin it). Rustdoc: request-level evaluation of an
   already-built expression; the expression is evaluated as-is — no
   lowering pass runs (callers are responsible for resolved symbols).

### Part B — typed construction (`rum-host/src/model/outcome.rs`)

2. **`expr_from_typed(session, typed, ...) -> Result<Expr, String>`**
   (exact signature shape yours; needs `resolve_symbol_name` access):
   - `Integer(i64)` / `BigInteger(decimal string, validated)` /
     `Rational { numerator, denominator }` (validated: decimal parse,
     nonzero denominator; rum's exact types normalize — document that
     `2/4` round-trips as `1/2`) → the corresponding atoms (probe
     `rum-core`'s public constructors; the out-direction
     `host_typed_from_atom` match names the variants to invert).
   - `Bool(b)` → system `True`/`False`; `Str` → string atom.
   - `Symbol(raw)` → `resolve_symbol_name(raw, true)`; `None` → Err.
   - `List(items)` → system `List` call; `Association(entries)` →
     a system `Association` call over `Rule[k, v]` entries (the
     evaluator normalizes it — `call` evaluates anyway);
     `Rule { lhs, rhs }` → system `Rule` call.
   - `Other { .. }` → Err, always (rustdoc: `Other` is an
     output-only lossy view; there is deliberately no input path).
   - Depth guard: recursion carries a depth counter; exceeding
     `MAX_TYPED_DEPTH` → Err naming the cap.

### Part C — the contract entry (`rum-host/src/session.rs`)

3. **`HostSession::call(&mut self, head: &str, args: &[HostTyped]) -> HostEvalOutcome`**
   — poisoned precheck + `guard` (the `eval` pattern); resolve `head`
   via `resolve_symbol_name(head, true)` (reject empty/unresolvable →
   synthesized failure); convert each arg via `expr_from_typed`;
   build `Expr::Call`; evaluate via `eval_expr_request`; map through
   the existing `eval_success_from_runtime`/`eval_failure_from_runtime`.
   Conversion failures → `synthesized_host_failure(HostFailureCode::Other,
   <clear message naming the offending arg index>)` — do NOT add a
   `HostFailureCode` variant.
   Rustdoc: the injection-safe invocation path — arguments are data,
   never parsed as source; evaluation semantics are ordinary (args
   were built, not held); `Symbol` args resolve exactly as source
   text would.

### Part D — microbench A/B (HARD GATE, interleaved)

`eval_request` was refactored, so the perf gate applies even though
zero behavior changed. Discipline (this supersedes any sequential
procedure you may infer from earlier briefs):

- Build the baseline binary from HEAD (pre-change), then the candidate
  from the working tree; run
  `scripts/core-language-microbench.sh --rum-cli <bin> --output-dir <dir>`
  **strictly interleaved: base, cand, base, cand, base, cand** (three
  pairs). Persist all six `timings.tsv`.
- Compare per-label best-of-3 vs best-of-3; the noise floor is the
  base runs' own spread. Expectation: the refactor is a function
  extraction on the once-per-form request path — the in-process label
  loops never touch it; deltas must sit inside the base spread. A
  label exceeding it in the best-of comparison → STOP and report (do
  not optimize).
- Profiling env OFF; no concurrent builds/tests during the runs.

## Tests (`rum-host/src/tests.rs` unless noted)

a. `call("StringJoin", [Str("a"), Str("b")])` → Success `"ab"` (short
   head resolved to System).
b. `call("Plus", [Integer(1), Integer(2)])` → Success, typed
   `Integer(3)`.
c. **Injection safety pin**: a `Str` argument containing
   `"], Print[666], x = "` (quotes/brackets/newlines) round-trips as
   DATA — the result is the exact string, nothing extra evaluated, no
   `x` defined afterwards.
d. Association round-trip: `call("Identity", [Association(...)])`
   returns the same `Association` typed value (in→expr→out symmetry).
e. `Symbol` args: a short name resolving to an existing definition and
   a fully-qualified name both work; `call("Set", [Symbol("x"),
   Integer(7)])` then `eval("x")` → 7 (the set-value sugar works
   through ordinary call semantics).
f. Rejections, each with a clear message: `Other` arg; malformed
   `BigInteger`; zero-denominator `Rational`; depth > cap.
g. Rational normalization: `Rational{2,4}` round-trips as
   `Rational{1,2}` (documented).
h. `rum-runtime` (existing request-test file): `eval_expr_request`
   carries the request protocol — messages aggregate and `termination`
   is set for a budget-terminated expr (reuse an existing
   recursion-limit fixture shape).

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/session/session_requests.rs`
- `crates/rum-runtime/src/runtime/tests/session_requests/api.rs` (or
  the request-test file you extend — name it)
- `crates/rum-host/src/model/outcome.rs`
- `crates/rum-host/src/session.rs`
- `crates/rum-host/src/lib.rs` (docs/re-exports only, if needed)
- `crates/rum-host/src/tests.rs`
- `docs/guide/embedding.md` (Typed invocation section)

MUST NOT touch: the evaluator core / lowering / resolution internals
(you only CALL `resolve_symbol_name`), `Drop` implementations,
`HostFailureCode`/`HostTyped` definitions (the enum gains nothing —
this brief only consumes it), `rum-jupyter`/`rum-cli`/`rum-repl`/
`rum-terminal`/`rum-stdlib`/`rum-syntax`/`rum-core`, `docs/**` beyond
the listed file.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

Plus Part D: the six tsv paths, per-label best-of-3 table, noise
assessment, verdict.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies, no new `unsafe`.
- `eval_request` behavior is frozen: no existing test expectation may
  change anywhere. If the refactor forces one, STOP and report.
- Stop-loss: if `rum-core` lacks a public constructor needed for an
  atom (integer/rational), STOP and report the missing constructor
  instead of adding one to `rum-core`.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Probed facts: the atom constructors used; `resolve_symbol_name`
   behavior for qualified names; where the shared request tail was cut.
3. Test survey — each new test, named.
4. Verification — commands with verbatim output (test totals).
5. Part D A/B — tsv paths, per-label best-of-3 rows, noise verdict.
6. Deviations / open questions (empty if none).
