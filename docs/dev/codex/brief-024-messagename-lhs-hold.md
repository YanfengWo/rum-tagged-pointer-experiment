# brief-024: MessageName assignment must hold its target symbol (BUG-MSG, snapshot usage loss)

## Goal

`sym::tag = value` (a `MessageName` assignment, e.g. `TRUTH::usage = "..."`) must
attach the message to the SYMBOL even when that symbol already has an OwnValue.
Today rum's definition-LHS normalization resolves the target symbol to its value
first, so `sym::tag = value` is silently dropped when `sym` has a value. This is
order-dependent and breaks the snapshot round-trip: `DumpSave` emits a symbol's
value BEFORE its `::usage`, so on `Get`-restore the value is set first and the
following `::usage` assignment is then dropped — every restored mma-hol snapshot
silently loses ~all theorem `::usage` strings (the `snapshot-reliability.sh`
harness from brief-023 fails its cycle-byte-identity check because of exactly
this; tests stay green because usage is documentation). When done, a `::usage`
assignment registers regardless of the symbol's value, the snapshot round-trip
preserves usage messages byte-for-byte, and the reliability harness's
message-loss gap is closed.

## Minimal repro (the bug)

```
rum-cli --eval 'Sandbox`b::usage = "d"; Sandbox`b = 5; Messages[Sandbox`b]'   # {Sandbox`b::usage}  (usage set BEFORE value: works)
rum-cli --eval 'Sandbox`a = 5; Sandbox`a::usage = "d"; Messages[Sandbox`a]'    # {}  BUG (value first: usage dropped)
```

Both must yield `{...::usage}`.

## Root cause (already diagnosed — implement directly)

In `crates/rum-runtime/src/runtime/definitions.rs`, `normalize_definition_lhs_expr`
(around line 447) normalizes a definition's LHS. A bare symbol arm calls
`resolve_definition_lhs_head_value`, which resolves the symbol to its OwnValue.
The function special-cases `Pattern`, `Blank*`, `PatternTest`, `Condition`,
`HoldPattern`, `Default`, and pattern wrappers to preserve their operands — but
there is NO case for `MessageName`. So the general `Expr::Call { head, args }`
arm (around line 517) recurses into `MessageName[sym, tag]`'s args and resolves
`sym` to its value. `MessageName[a, "usage"]` becomes `MessageName[5, "usage"]`,
`message_name_key_from_expr` / `resolve_message_name_args` then can't build a
`MessageNameKey` from the integer, the assignment target never normalizes to
`DefinitionAssignmentTarget::Message`, and `define_message` is never called.

Cold mma-hol sets `TRUTH::usage` before `TRUTH`'s value (no value yet -> symbol
preserved -> registers); `DumpSave` re-emits value-before-usage; `Get`-restore
sets the value first, so the trailing `::usage` hits the bug and is lost.

## The fix (precise)

Add a `MessageName` arm to `normalize_definition_lhs_expr` that preserves its
operands WITHOUT resolving the target symbol — mirror the existing `HoldPattern`
arm (around line 501):

```rust
Expr::Call { head, args } if is_system_symbol_name(head, "MessageName") => {
    Expr::call_with_shared_args(head.as_ref().clone(), args.clone())
}
```

Place it among the other operand-preserving arms (before the general
`Expr::Call { head, args }` arm). Do not change the message storage, DumpSave, or
evaluation paths — only stop the LHS normalizer from resolving the MessageName
target symbol. (The cleanest, smallest fix; do not also reorder DumpSave output.)

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions.rs` (the one arm).
  - ONE test file under `crates/rum-runtime/src/runtime/tests/` for the
    regression test.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone (for the snapshot check, copy to a
    throwaway scratch dir — NEVER write to `/Users/fft/Developments/mma-hol-acceptance`);
  - DumpSave/message-storage/evaluation hot paths beyond the named arm;
  - `crates/rum-core` `Expr`/`Symbol`/serde shape;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your own
    report, `.git`, git config, `Cargo.lock`.
- This is a parse/normalization-time change; it must not add per-evaluation-step
  cost.

## Deliverable

- The `MessageName` arm in `normalize_definition_lhs_expr`.
- A regression test: a symbol with a value, then a `::usage` assignment, asserts
  `Messages[sym]` contains the message (both orders: value-then-usage and
  usage-then-value).
- In your report: the two repro lines before/after, the synthetic snapshot
  round-trip (usage survives restore+redump), and the `snapshot-reliability.sh`
  result (note whether it is now fully green or whether a residual non-usage diff
  remains).

## rum / Rust pitfalls (read twice)

1. **Only MessageName.** Do not change how other LHS heads normalize. `f[x] :=`,
   `Options[s]=`, etc. must be unaffected.
2. **Both orders must work** after the fix: usage-before-value AND
   value-before-usage.
3. **Evaluation unchanged.** `sym::tag` still EVALUATES the same (this brief is
   about the assignment LHS, not message lookup). Run the full suite.
4. **Minimal diff**; no reformatting.
5. **Scratch only** for any snapshot build; confirm the pristine clone stays
   empty in `git status`.
6. **Stop-loss:** if more than a couple of tests change in non-obvious ways, or
   the synthetic round-trip still drops usage, STOP and report.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# 1. the bug, both orders:
$BIN --eval 'Sandbox`b::usage = "d"; Sandbox`b = 5; Messages[Sandbox`b]'   # expect {Sandbox`b::usage}
$BIN --eval 'Sandbox`a = 5; Sandbox`a::usage = "d"; Messages[Sandbox`a]'   # expect {Sandbox`a::usage}

# 2. synthetic round-trip preserves usage:
rm -f /tmp/u.mx /tmp/u2.mx
$BIN --eval 'Sandbox`a = 5; Sandbox`a::usage = "doc-a"; DumpSave["/tmp/u.mx", "Sandbox`"]; "ok"'
grep 'usage' /tmp/u.mx                 # cold now contains the usage
$BIN --eval 'Get["/tmp/u.mx"]; DumpSave["/tmp/u2.mx", "Sandbox`"]; "ok"'
grep 'usage' /tmp/u2.mx                 # restore+redump STILL contains it
wc -c /tmp/u.mx /tmp/u2.mx              # equal byte counts

# 3. the reliability harness (scratch copy; ~1-2 min):
bash scripts/snapshot-reliability.sh --mma-hol /Users/fft/Developments/mma-hol-acceptance --output-dir /tmp/cdx024-snaprel
cat /tmp/cdx024-snaprel/summary.tsv
echo "exit=$?"
git -C /Users/fft/Developments/mma-hol-acceptance status --short   # MUST be empty
```

Paste verbatim. The acceptance is repro #1 (both `{...::usage}`) and #2 (u.mx ==
u2.mx with usage in both). For #3, report whether the harness is fully green; if
a small residual diff remains that is NOT usage-related (e.g. a regenerated
`$ContextPath`/`$Packages` preamble), report it explicitly and STOP — do not
chase it in this brief.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone; scratch copy only.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo (except named `/tmp`
  scratch), no network. Commands: file edits within Scope, `cargo`, release
  `rum-cli`, `bash`/`rsync`/`grep`, `python3`.
- Minimal diff; no drive-by churn.
- You MUST paste real verbatim output; if not green, say so — never fake it.

## Report format (final message)

1. What changed — file, line range, one-line why.
2. Verification — each command, verbatim output.
3. The verdict — both repro orders register the message; synthetic round-trip
   preserves usage; the harness result (fully green, or the precise residual).
4. Stop-loss / open questions — anything unsure (esp. any residual harness diff).
