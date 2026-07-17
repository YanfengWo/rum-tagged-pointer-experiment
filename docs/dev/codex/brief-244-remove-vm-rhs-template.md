# Brief 244 — Remove the vm_rhs_template experiment (ruled DELETE 2026-07-17)

## Goal

Remove the opt-in `vm_rhs_template` compiled-RHS experiment entirely, leaving
the default (flag-OFF) behavior byte-identical. The module is env-gated OFF
(`RUM_VM_RHS_TEMPLATE`), has no consumer, and its ON path no longer passes the
workspace suite (native stack overflow on the guard-adjacent recursion test),
so the fate ruling is deletion; the code stays recoverable in git history.
When you are done: the directory is gone, the seam call sites run the plain
substitution path directly, the workspace is green, and an interleaved
microbench A/B shows no regression on the definition-application labels.

## Context pointers

- Read `AGENTS.md` first (operational notes, Testing Policy).
- `crates/rum-runtime/src/runtime/vm_rhs_template/` — the module to delete
  (7 files: mod/cache/compiler/executor/model/safety/tests).
- `crates/rum-runtime/src/runtime/mod.rs:56` (`mod vm_rhs_template;`) and
  `:67` (`vm_rhs_template::*` inside the glob re-export) — the only wiring.
- The five usage clusters (each is a prepare/trace/evaluate triple):
  - `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`
    lines ~111-136, ~204-240, ~293-329;
  - `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`
    lines ~122-158, ~211-247.
- The plain path the OFF arm already takes:
  `substitute_rule_bindings_for_rhs` (
  `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:88`)
  and the `Session::evaluate_definition_rhs` method.
- Acceptance is mechanical: the named tests and the A/B rows decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/vm_rhs_template/` — DELETE the whole
    directory.
  - `crates/rum-runtime/src/runtime/mod.rs` — MECHANICAL ONLY: remove the
    `mod vm_rhs_template;` line and `vm_rhs_template::*` from the glob
    re-export; nothing else.
  - `crates/rum-runtime/src/runtime/definitions/application/down_values.rs`
    and `.../sub_values.rs` — ONLY the five listed clusters plus any `use`
    lines that become necessary/unused because of the change.
- Files/areas you MUST NOT touch: everything else — all other sources,
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` (Claude updates docs at
  close-out), `.git`, `Cargo.lock`.
- The rewrite at each cluster inlines the OFF arm EXACTLY (today's default
  behavior), e.g. for the first down_values cluster:

  ```rust
  let rhs = {
      let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
      substitute_rule_bindings_for_rhs(
          &down_value.rhs,
          &SymbolExprMap::default(),
          down_value.rhs_may_splice_without_bindings,
      )
  };
  let trace_rhs = rhs.clone();
  // …push_rule_applied_trace / profiling lines unchanged…
  let result = self.evaluate_definition_rhs(option_scope, rhs);
  ```

  Notes pinned from reading the current code, do not deviate:
  - `trace_rhs_for_prepared_template`'s Substituted arm returns the
    SUBSTITUTED expr clone unconditionally (the `include_payload` /
    `tracing_enabled` argument only matters for the deleted Compiled arm),
    so the replacement is an unconditional `rhs.clone()` — do NOT move it
    under a tracing check or "optimize" it away; byte-identical means
    identical.
  - `evaluate_prepared_rhs_template`'s Substituted arm is a direct
    `session.evaluate_definition_rhs(option_scope, rhs)` with NO
    suspend/lazy wrapper (that wrapper belonged to the Compiled arm only).
  - The `self.evaluation_epoch` argument disappears with the deleted
    function; remove only locals/args that become unused at these exact
    sites — no wider cleanup.
- Do NOT leave a stub, a feature flag, or a re-landing hook; the deletion is
  total. Do NOT add any new cost to the path (this change strictly removes
  one relaxed atomic load + branch + enum wrap).

## Deliverable

- The deletion + the five cluster rewrites.
- Green `cargo fmt --check` and `cargo test --workspace` (paste totals).
- A post-change grep proving zero remaining references:
  `rg -n 'vm_rhs_template|RUM_VM_RHS_TEMPLATE|PreparedRhsTemplate' crates/`
  must return nothing (the TODO.md mention is out of your scope and stays).
- The interleaved microbench A/B rows (procedure below): every label's
  rum_seconds for all four runs, with the definition-application labels and
  their same-tier neighbors called out.

## rum / Rust pitfalls (read twice)

1. Timing numbers come from `scripts/core-language-microbench.sh` only;
   profiling env vars OFF for every timing run.
2. **Build the BASE binary BEFORE touching any file**:
   `cargo build --release -p rum-cli`, then
   `cp target/release/rum-cli target/ab-base-rum-cli`. After the change,
   build the candidate (`cargo build --release -p rum-cli` again;
   candidate = `target/release/rum-cli`).
3. Run the suite four times, INTERLEAVED (base, cand, base, cand):

   ```
   bash scripts/core-language-microbench.sh --rum-cli target/ab-base-rum-cli --output-dir /tmp/brief244-base1
   bash scripts/core-language-microbench.sh --rum-cli target/release/rum-cli  --output-dir /tmp/brief244-cand1
   bash scripts/core-language-microbench.sh --rum-cli target/ab-base-rum-cli --output-dir /tmp/brief244-base2
   bash scripts/core-language-microbench.sh --rum-cli target/release/rum-cli  --output-dir /tmp/brief244-cand2
   ```

4. **Noise verdict rule (follow it, do not improvise):** for each label, the
   within-arm spread |base1−base2| (and |cand1−cand2|) is that label's noise
   floor. A label counts as a regression ONLY if the candidate is slower than
   base in BOTH pairs AND the delta exceeds that label's own within-arm
   spread. Do not STOP on a single suspect label with a razor-thin spread —
   report the numbers and let the gate judge.
5. One variable per measurement: nothing else changes between arms — same
   machine state, no parallel jobs during timing runs.
6. Suspension safety: the Substituted arm never had a suspension wrapper —
   do not add one; conversely do not remove the surrounding
   profiling/trace calls that are outside the triple.
7. Stop-loss: if a cluster does not reduce cleanly to the three-line OFF-arm
   shape (hidden dependency on the deleted types), STOP and report instead
   of inventing a workaround.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
rg -n 'vm_rhs_template|RUM_VM_RHS_TEMPLATE|PreparedRhsTemplate' crates/
```

Then the four interleaved microbench runs above. Paste verbatim: the test
totals, the empty grep, and the per-label rows (all labels; flag the
definition-application ones). Never claim a result without the verbatim
lines.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope; if you believe you must, STOP and write
  that in the report.
- Full machine access, but touch NOTHING outside this repo, no network. Only
  file edits within Scope, `cargo`, the release `rum-cli`, the microbench
  script, and `python3` for parsing.
- Testing Policy hard line (`AGENTS.md`): no external-engine comparisons in
  any form.
- No new `unsafe` (this brief only deletes).
- Hot-path rule: this change touches the definition-application path — the
  A/B gate above is mandatory, and a confirmed regression (per the noise
  verdict rule) is a defect to report, not to ship.
- Minimal diff: no drive-by reformatting or renames outside the clusters.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim test totals, empty grep, and per-label A/B rows
   for all four runs.
3. Verdict — regression or none, per the noise verdict rule, with the
   definition-application labels and neighbors called out.
4. Stop-loss / open questions (empty if none).
