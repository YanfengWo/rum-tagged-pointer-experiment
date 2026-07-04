# Brief 121 — runtime/ directory fan-out grouping (74 → ~48 direct children)

## Goal

Execute the first application of the new §Directory Fan-Out policy
(`docs/code-organization-guidelines.md`): group `crates/rum-runtime/src/runtime/`'s
74 direct children into FIVE domain subdirectories (specified below — the
grouping design is fixed; you execute and may only adjust with reported
justification). Pure organization: files move as-is (`git mv` semantics),
zero behavior change, zero pub-face change; the runtime's internal glob and
`kernel_api` paths are updated mechanically. Workspace green (1301/0).

## The five groups (design fixed)

1. `definitions/` ← `definitions.rs`, `definition_analysis_helpers.rs`,
   `definition_catalog.rs` + `definition_catalog/`, `definition_clear.rs`,
   `definition_tags.rs`, `definition_updates.rs`, and the existing
   `definition_application/` directory (becomes `definitions/application/`).
2. `messages/` ← `builtins_messages.rs`, `builtins_messages_inspect.rs`,
   `message_catalog.rs`, `message_format.rs`, `message_selector_parsing.rs`,
   `parse_and_message_helpers.rs`.
3. `session/` ← `session_api.rs`, `session_eval.rs`, `session_helpers.rs`,
   `session_requests.rs`, `session_state.rs`.
4. `substitution/` ← `binding_substitution.rs`, `lexical_presence.rs`,
   `lexical_substitution.rs`, `slot_substitution.rs`,
   `function_and_subst_helpers.rs`.
5. `kernel_data/` ← the rebalance line's kernel-resident data/machinery
   family: `association_data.rs`, `functional_data.rs`, `rewrite_data.rs` +
   `rewrite_data/`, `structure_data/`, `control_data/`, `text_rendering.rs`.

Everything else stays at the top level (the eval core, pattern matching,
scoping, builtin catalog/adapters, kernel_api, model, profiling, etc. keep
their visibility). Naming: inner file names keep their current names (e.g.
`definitions/definition_updates.rs` may drop the redundant prefix →
`definitions/updates.rs` ONLY if you rename consistently across a whole
group and report it; keeping original names is also acceptable — pick ONE
convention per group and say which).

## Mechanics

- Each group gets a `mod.rs` that declares its children and re-exports their
  contents at the same visibility as before (`pub(super) use` /
  `pub(in crate::runtime) use` transparently), so `runtime/mod.rs`'s glob
  block shrinks to referencing the five group modules and every existing
  bare-name reference inside the runtime keeps resolving.
- `kernel_api.rs` re-export paths and any explicit `crate::runtime::X::…`
  paths update mechanically (compiler-driven).
- File CONTENTS untouched except: their own `use super::…` lines that module
  depth changes force (report the pattern once, not per file).
- Tests keep their homes; `runtime/tests/` is NOT part of this grouping.
- No A/B needed: module re-arrangement with no body changes has no codegen
  surface — but run the sentinel probe once and paste it
  (`Table[j+1,{j,100000}]` timing via rum-cli, expect the 60-70ns/iter
  order).

## Scope

- MAY modify: the moved files' locations + their `use` lines; the five new
  `mod.rs` files; `runtime/mod.rs` (mod decls + glob block);
  `kernel_api.rs` (path retargets only); explicit-path references the
  compiler forces (list the files).
- MUST NOT touch: file bodies beyond `use` lines, any test content, other
  crates, `Cargo.toml`/`Cargo.lock`, any `mma-hol` clone, `docs/**` except
  your report, `.git`.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test --workspace
ls crates/rum-runtime/src/runtime/ | wc -l          # expect ~48
ls crates/rum-runtime/src/runtime/
cargo run -p rum-cli --release -- --eval "First[AbsoluteTiming[Table[j+1,{j,100000}]]] // N" 2>/dev/null | tail -1
git status --short -- crates/ | head -40
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, `grep`,
  `python3`. Never edit any `mma-hol` file. **No new `unsafe`.**
- Stop-loss: a group that cannot re-export transparently (callers would need
  edits beyond mechanical paths), same compile error twice → STOP and report
  which boundary resists.

## Report format

1. Per-group outcome: children moved, naming convention chosen, the
   re-export shape.
2. What changed — the final `ls` of runtime/ + child counts before/after;
   the use-line pattern note; kernel_api retarget list.
3. Verification — verbatim outputs incl. the sentinel probe.
4. Verdict — fan-out number achieved; any group that resisted; whether
   further grouping (eval cluster?) looks worthwhile or forced.
5. Stop-loss / open questions.
