# Brief 117 — file-slimming batch 4: vm_rhs_template.rs (53KB) + lexer.rs (41KB)

## Goal

Fourth slimming batch: two non-eval-hot debt files in one pass.
`crates/rum-runtime/src/runtime/vm_rhs_template.rs` (52,898 chars — the
OPT-IN closure-VM experiment, env-gated off by default) and
`crates/rum-syntax/src/lexer.rs` (40,780 chars) each become module
directories per brief-114's SPLITTING PRINCIPLES (split map first, by
logic/function, stable/churn separation, sibling DAG, verbatim bodies,
facade keeps external imports unchanged). Zero behavior change; workspace
green (1301/0); every new file ≤20k. No microbench: the VM is opt-in-off
(its disabled check must stay verbatim), and the lexer is load-time, not
eval-path — assert both in the report.

## Context pointers

- Read `AGENTS.md`, `docs/code-organization-guidelines.md`, brief-114
  principles.
- `vm_rhs_template.rs`: the opt-in RHS closure-template VM
  (docs/bytecode-vm-design.md context: compile-to-closures that DELEGATE to
  the real evaluator). Suggested axis (from the batch-3 report; verify):
  data model (template/instruction types), compiler (template construction),
  executor (closure application/resume), tests (the in-file test module,
  incl. the test-local builtin registrations added during the rebalance
  line). FUTURE-REFACTORING note: this file's stay/feature-gate fate is an
  open decision — boundaries must allow wholesale gating/moving later.
  Fix-in-passing ALLOWED (report it): the pre-existing `source_template`
  dead-code warning may be resolved ONLY if the fix is removing a genuinely
  unused item or adding the missing use — no behavior change.
- `lexer.rs` (rum-syntax): suggested axis (verify): core tokenization loop,
  escape/string handling, number literals, diagnostics/errors, tests if
  in-file. The lexer is PERFORMANCE-RELEVANT at load time (snapshot chains
  lex big files) but not on the eval path — bodies verbatim keeps that moot.
- Both splits: prefer `pub(super)`/`pub(crate)` minimal visibility; the
  facades (`vm_rhs_template/mod.rs`, `lexer/mod.rs`) re-export the old
  surface so no caller changes.

## Scope

- MAY modify: the two files → their directories; `use` lines inside;
  parent mod declarations only if syntax demands (likely zero);
  the dead-code warning fix-in-passing in vm_rhs_template (optional,
  reported).
- MUST NOT touch: any caller outside the two directories, any behavior,
  `Cargo.toml`/`Cargo.lock`, any `mma-hol` clone, `docs/**` except your
  report, `.git`.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test --workspace
ls crates/rum-runtime/src/runtime/vm_rhs_template.rs crates/rum-syntax/src/lexer.rs 2>&1   # both gone
find crates/rum-runtime/src/runtime/vm_rhs_template crates/rum-syntax/src/lexer -name "*.rs" | while read f; do wc -c "$f"; done | sort -rn
git status --short -- crates/                                                               # exactly: 2 deletions + 2 new directories
cargo build --release -p rum-cli 2>&1 | grep -c "warning" || true                           # report the count (dead-code fix evidence)
```

Every new file ≤20k (justify exceptions per principle 1). Paste all outputs
verbatim + both SPLIT MAPS.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, `grep`,
  `python3`. Never edit any `mma-hol` file. **No new `unsafe`.**
- Stop-loss: a boundary that can't keep callers unchanged, or the same
  compile error twice → STOP and report.

## Report format

1. BOTH SPLIT MAPS (module → responsibility → chars; DAG; the VM
   future-gate alignment note).
2. What changed — inventory; untouched-callers evidence; the dead-code
   disposition.
3. Verification — verbatim outputs incl. size tables.
4. Verdict — opt-in-off / load-time assertions; advice for batch 5
   (arithmetic 42k, HOT) and batch 6 (the two test files).
5. Stop-loss / open questions.
