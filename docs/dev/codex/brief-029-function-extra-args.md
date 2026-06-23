# brief-029: `Function[params, body]` must ignore EXTRA arguments (WL semantics)

## Goal

Applying a named-parameter pure function to MORE arguments than it has parameters
must bind the first N args to the N params and IGNORE the extras (Wolfram
semantics), e.g. `Function[e, e + 1][42, 99]` is `43`, `Function[{x}, x^2][5, 6, 7]`
is `25`. Today rum requires EXACT arity: `try_apply_function` errors with an
`argx` message and returns the call UNEVALUATED when `args.len() != params.len()`.
This silently breaks the standard `Catch[expr, tag, handler]` idiom — Wolfram
calls `handler[value, tag]` (2 args), and a 1-parameter handler `Function[err,
...]` then fails to apply — which is the root of the current correctness-mainline
frontier: `auto/Arith.wl`'s `refuteWithNonneg` is
`Catch[farkasRefute[facts], holErrorTag, Function[err, <add ℕ non-negativity and
retry>]]`; the handler never runs, so the non-negativity retry is skipped and
`arith-farkas: no Farkas certificate` propagates, stalling the load at
`stdlib/Int.wl`. When done, extra args are ignored, the `Catch` handler runs, and
the number tower's Farkas refutation gets its non-negativity retry.

## Context pointers

- Read `AGENTS.md` and the PROGRESS entry `2026-06-23 Correctness mainline:
  checkpoint chain ...`. Root-caused from the Int frontier: the 7 `diffs` fed to
  `farkasFM` have no rational certificate WITHOUT ℕ non-negativity (verified by
  hand: the positive-constant multiplier is forced to 0); WITH `atom ≥ 0` a cert
  exists (`d0 + d6 + (atom$3≥0) = 1 ≤ 0`). `refuteWithNonneg`'s `Catch` handler is
  supposed to add those non-negativity facts and retry, but the handler — a
  1-param `Function[err, ...]` called by `Catch` as `handler[value, tag]` (2 args)
  — never applies.
- Minimal repros (all currently wrong):
  - `Function[e, e + 1][42, 99]` -> unevaluated `Function[e, Plus[e, 1]][42, 99]`
    (should be `43`).
  - `Function[{x}, x^2][5, 6, 7]` -> unevaluated (should be `25`).
  - `Catch[Throw[42, foo], foo, Function[e, {"h", e}]]` -> unevaluated
    `Function[e, List["h", e]][42, foo]` (should be `{"h", 42}`).
  - Contrast (already correct): the Slot form `(# + 1 &)[10, 20]` -> `11` (extras
    ignored); exact-arity `Function[{x, y}, x + y][3, 4]` -> `7`.
- Key file & exact site: `crates/rum-runtime/src/runtime/scoping/callable.rs`,
  `try_apply_function`, the `FunctionSpec::Explicit { params, body, .. }` arm
  (around line 12-28):

  ```rust
  FunctionSpec::Explicit { params, body, .. } => {
      if params.len() != args.len() {                 // <-- too strict
          self.push_message("Function", "argx",
              format!("expected exactly {} argument(s), got {}", params.len(), args.len()));
          return Ok(Some(Expr::call(head.clone(), args.to_vec())));
      }
      let bindings = params.into_iter().zip(args.iter().cloned()).collect();
      substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
  }
  ```

## The fix (precise)

Change the arity guard to error ONLY when there are TOO FEW arguments; extra
arguments are ignored (the existing `zip` already binds just the first
`params.len()` args):

```rust
if args.len() < params.len() {
    self.push_message("Function", "argx",
        format!("expected at least {} argument(s), got {}", params.len(), args.len()));
    return Ok(Some(Expr::call(head.clone(), args.to_vec())));
}
let bindings = params.into_iter().zip(args.iter().cloned()).collect();
substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
```

(Only the `<` condition and the message wording change; the `zip` binding is
unchanged and naturally drops extras. The too-FEW case keeps the current
message-and-unevaluated behavior. Do not touch the SlotBased arm — it already
ignores extras correctly.)

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/scoping/callable.rs` (the one guard).
  - a test module under `crates/rum-runtime/src/runtime/tests/` for the
    regression cases.
- MUST NOT touch: any `mma-hol` clone; `crates/rum-core`; the SlotBased path or
  the substitution machinery; `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`
  except your report; `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Only relax too-MANY.** `args.len() < params.len()` (too few) keeps the
   message + unevaluated return. Exact arity is unchanged.
2. **Extras are ignored, not appended.** The result is the substituted body, not
   a call carrying leftover args.
3. **Don't regress the SlotBased arm or `parse_function`.**
4. **Minimal diff.** **Stop-loss:** if tests beyond the expected message-text
   updates break, STOP and report.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'Function[e, e + 1][42, 99]'                 # 43
$BIN --eval 'Function[{x}, x^2][5, 6, 7]'                # 25
$BIN --eval 'Function[x, x][1, 2, 3]'                    # 1
$BIN --eval 'Catch[Throw[42, foo], foo, Function[e, {"h", e}]]'   # {h, 42}
# controls (unchanged):
$BIN --eval 'Function[{x, y}, x + y][3, 4]'              # 7
$BIN --eval '(# + 1 &)[10, 20]'                          # 11
$BIN --eval 'Function[e, e + 1][7]'                      # 8
```

Paste verbatim. (Full Int.wl / number-tower re-verification via the checkpoint
chain — restored from the v4 `cp24-FTA` checkpoint + extend Int — is Claude's
follow-up; you do NOT run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`, `python3`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the guard in `callable.rs`, line range, one-line why.
2. Verification — each command, verbatim output.
3. The verdict — extra args ignored; `Catch[e, tag, Function[err,...]]` runs the
   handler; controls unchanged; tests green.
4. Stop-loss / open questions.
