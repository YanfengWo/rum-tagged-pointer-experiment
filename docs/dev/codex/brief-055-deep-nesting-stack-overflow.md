# brief-055: deep-nesting stack overflow — graceful parse-depth guard (+ iterative Drop if needed)

## Goal

A deeply-nested input must NOT crash the process. Today a `~5000`-deep nested literal
`f[f[…f[x]…]]` aborts `rum-cli` with SIGABRT (native stack overflow) — far below any
pathological depth, and a hard process crash (fatal for a host embedding). When done,
such input instead fails GRACEFULLY (a clean parse error well below the native-stack
limit, like WK's `$RecursionLimit`) and the session/process survives; `cargo test
--workspace` stays green; no default-path cost.

## Diagnosis (already done — confirm, then fix)

- Confirmed: a `5000`-deep nested literal crashes (`exit 134` SIGABRT). The `Hold[…]`
  path (which blocks deep EVAL) crashes too, so it is NOT deep eval — eval is already
  guarded (`recursion_limit` 1024, `evaluation_pass_limit` 256 → graceful
  `TerminatedEvaluation`). The unguarded recursers are the **recursive-descent parser**
  (`crates/rum-syntax/src/parser.rs` + `parser/helpers.rs`, `parser/front.rs` — NO depth
  guard today) and the **default recursive `Drop`** of the `Arc<Expr>` tree (rum-core
  has NO manual `Drop` impl — a deep `SharedExpr`/`SharedArgs` chain drops recursively).
- macOS gotcha: a crashing run triggers `ReportCrash` and takes ~30 s+; run crashing
  probes with `ulimit -c 0` and budget for the delay (it cost the diagnosis two
  timeouts). Prefer a Rust unit test over CLI probes where possible.

## Fix

**Primary — parse-depth guard (definitely needed; covers the confirmed crash).** Add a
depth counter to the recursive-descent parser: increment when entering a nested
expression (the recursive parse step for a bracketed/grouped sub-expression),
decrement on exit, and return a clean parse ERROR (the existing `error.rs` syntax-error
path) when depth exceeds a safe `MAX_PARSE_DEPTH`. Pick `MAX_PARSE_DEPTH` well below the
native overflow (~5000) — e.g. 1024 (matching the eval `recursion_limit`) or a dedicated
constant; document the choice. The error must surface as a normal parse failure (the
`HostFailureCode::Syntax` / CLI parse-error path), NOT a panic. This also prevents a
too-deep AST from being CONSTRUCTED, so the subsequent `Drop` of a parse-sourced expr
never sees pathological depth either.

**Secondary — iterative `Drop`, ONLY IF a non-parse path can still build a droppable
deep expr.** First LOCALIZE: can a deep `Expr` arise WITHOUT going through the parser or
the eval `recursion_limit`? (e.g. a builtin that constructs depth in a Rust loop —
`Nest`/`NestList`/`Fold` — before any eval guard fires, or with `MAX_PARSE_DEPTH` raised
in a test.) Build such a deep expr in a Rust test and drop it; if it SIGABRTs, the
default recursive `Drop` is a second landmine → add a manual iterative `Drop` for
`Expr` (or `SharedExpr`/`SharedArgs`): an explicit work-stack that unwinds the tree
without native recursion, descending ONLY into `Arc`-UNIQUE children (refcount 1) so
shared subtrees are not double-freed and non-unique nodes are left to their other
owners. If the parse guard alone makes every reachable deep-expr source graceful (no
test can build a droppable deep expr), DOCUMENT that and skip the iterative Drop. Do
not add it speculatively if it cannot be shown necessary.

## Scope

- MAY modify: `crates/rum-syntax/src/parser.rs` + `parser/helpers.rs` / `parser/front.rs`
  + `error.rs` (the depth guard + its error); and IF the localization proves it needed,
  `crates/rum-core/src/expr.rs` (a manual iterative `Drop` for `Expr`/`SharedExpr`/
  `SharedArgs`).
- MUST NOT touch: any `mma-hol` clone; `Atom`/`Symbol` representation (unrelated to this
  fix); `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your report; `.git`,
  `Cargo.lock`.
- No default-path cost: the parse-depth counter is a single `usize` inc/dec on the parse
  path (cheap, not the eval hot path). A manual `Drop`, if added, must not slow the
  common shallow-drop case — keep a fast path for small/shallow exprs.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax        # parser tests + a NEW test: a too-deep literal returns a parse error, not a panic
cargo test --workspace
```
Add a `rum-syntax` test asserting a `MAX_PARSE_DEPTH+`-deep literal yields a parse
ERROR (Result::Err), not a panic/abort. If you add iterative Drop, add a `rum-core`
test that constructs + drops a very deep expr without overflow.

Then build the release CLI and paste verbatim (with `ulimit -c 0`) that deep input no
longer SIGABRTs — it should print a graceful parse error / a value and exit 0/normally:
```
cargo build --release -p rum-cli
ulimit -c 0
python3 -c "print('f['*5000 + 'x' + ']'*5000)"  | (read L; target/release/rum-cli --eval "$L;1"; echo "exit=$?")
python3 -c "print('f['*100000 + 'x' + ']'*100000)" | (read L; target/release/rum-cli --eval "$L;1"; echo "exit=$?")
```
Both must exit gracefully (NOT 134/139). Paste the output + exit codes. Also confirm a
NORMAL shallow expression still parses+evaluates correctly (e.g. `f[g[x], 2+3]`) — the
guard must not affect ordinary depth.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the working tree for Claude's gate.
- Do NOT modify files outside Scope. If the fix needs more, STOP and report.
- Full machine access, repo only, no network. Commands: edits, `cargo`, the release
  `rum-cli`, `python3` for generating the deep literal.
- Never edit any `mma-hol` file.
- Stop-loss: if `cargo test --workspace` is red after two focused fix passes for the
  same failure, STOP and report. If the iterative `Drop` proves subtle (Arc-sharing
  double-free risk), STOP and report rather than ship an unsound teardown.
- Paste REAL verbatim output (incl. exit codes); never fabricate.

## Report format

1. What changed — per file + line ranges; `MAX_PARSE_DEPTH` value + why; whether an
   iterative `Drop` was needed (with the localization evidence) or not.
2. Verification — every command + verbatim output (test result lines; the two deep-input
   exit codes; the shallow-expr sanity check).
3. The verdict — deep input now graceful? Drop landmine present or excluded by the parse
   guard?
4. Stop-loss / open questions (empty if none).
