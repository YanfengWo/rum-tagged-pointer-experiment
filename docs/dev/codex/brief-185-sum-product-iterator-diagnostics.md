# Brief 185 — `Sum`/`Product` malformed-iterator diagnostics (unit-4 batch 3)

Implementation brief. Executes unit-4 triage batch 3 (tracker
`docs/dev/usability-pass.md`, unit-4 section; evidence E4/E10 in
`docs/dev/usability-pass/control-logic-rewrite-findings.md`). Low-risk
safety parity: `Sum` and `Product` silently return an inert call for a
malformed iterator spec, while the sibling `Do`/`Table` emit an
`argform` message for the same input. This closes the last unit-4 fix
batch. NOT a hot path (error path only) — **no microbench A/B gate**.

## The gap (Evidence E4)

```
Do[1, {i, 1, 3, 0}]      -> Do::argform + inert Do[1, {i,1,3,0}]
Table[i, {i, 1, 3, 0}]   -> Table::argform + inert Table[i, {i,1,3,0}]
Sum[i, {i, 1, 3, 0}]     -> inert Sum[i, {i,1,3,0}]      (NO message)   <-- gap
Product[i, {i, 1, 3, 0}] -> inert Product[i, {i,1,3,0}]  (NO message)   <-- gap
Do[1, bad] / Table[1, bad]     -> argform + inert
Sum[1, bad] / Product[1, bad]  -> inert, NO message                     <-- gap
```

Root cause: in
`crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs`,
`finish_iteration_aggregate_spec_eval` returns the silent inert call on
a `None` (malformed) spec:

```rust
let spec = match parse_do_spec(self, &spec_expr, aggregate.consumer())? {
    Some(spec) => spec,
    None => return Ok(Expr::call(system_expr(aggregate.name()), args)),   // silent
};
```

The `Do` precedent (same `parse_do_spec` → `None`) 4 lines away in
`branching_loops/iteration/driver.rs` (`finish_do_spec_eval`):

```rust
None => {
    return Ok(self.invalid_call_with_arg_form(
        "Do",
        &args,
        "expected a supported repetition count or iterator specification",
    ));
}
```

## Change

In `iteration_aggregate.rs`, replace the silent `None` branch in
`finish_iteration_aggregate_spec_eval` with the `Do` precedent, keyed on
the aggregate name:

```rust
None => Ok(self.invalid_call_with_arg_form(
    aggregate.name(),
    &args,
    "expected a supported repetition count or iterator specification",
)),
```

`invalid_call_with_arg_form` returns the SAME inert `Name[args]` call
(byte-identical to today) and additionally emits `Sum::argform` /
`Product::argform` with that message — exactly `Do`/`Table`'s shape.
This is the ONLY change. The `Some(spec)` valid-iteration path is
untouched: `Sum[i, {i, 1, 3}]` still yields `6`, `Product[i, {i, 1, 4}]`
still yields `24`, and nested/multi-spec aggregates are unchanged.

## Tests

Add rum-runtime tests (co-locate with the iteration / `argform`-message
tests under `crates/rum-runtime/src/runtime/tests/`):

- `Sum[i, {i, 1, 3, 0}]` and `Product[i, {i, 1, 3, 0}]` (zero step) each
  emit `Sum::argform` / `Product::argform` with the repetition-count
  message AND return the inert `Sum[i, {i,1,3,0}]` / `Product[i,
  {i,1,3,0}]` call.
- `Sum[1, bad]` and `Product[1, bad]` (non-list spec) do the same.
- Valid aggregates are unchanged and message-free: `Sum[i, {i, 1, 3}]`
  → `6`, `Product[i, {i, 1, 4}]` → `24`.
- Keep the existing `Do`/`Table` behavior tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# the E4 probe, post-fix — Sum/Product now message like Do/Table:
printf '' | cargo run -q -p rum-cli -- --eval '{Do[1, {i, 1, 3, 0}], Table[i, {i, 1, 3, 0}], Sum[i, {i, 1, 3, 0}], Product[i, {i, 1, 3, 0}], Do[1, bad], Table[1, bad], Sum[1, bad], Product[1, bad]}'
# valid aggregates still compute and stay message-free:
printf '' | cargo run -q -p rum-cli -- --eval '{Sum[i, {i, 1, 3}], Product[i, {i, 1, 4}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
the first probe now shows FOUR `argform` messages (`Do`, `Table`, `Sum`,
`Product`) with the inert list unchanged; the valid aggregates return
`{6, 24}` with no message.

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs`
  (the `None` branch only), and the iteration/message test file(s) under
  `crates/rum-runtime/src/runtime/tests/**` (new tests).
- MUST NOT touch: `Do`/`Table`/`For`/`While` behavior; the valid
  `Sum`/`Product` iteration path; `parse_do_spec`; the guide,
  `docs/reference/builtins.md` (messages are not in the reference),
  `TODO.md`, the tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve the suspend/resume continuation shape of the
  `eval_*`/`resume_*`/`finish_*` paths (this change is a single return
  branch — do not restructure the surrounding continuation).
- Compat: the inert return value is byte-identical to today; only a
  message is added, matching `Do`/`Table`. Valid iteration is unchanged.
- No A/B gate needed (error path, not eval/dispatch/matcher hot path).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the file + line range and the one-line why.
2. Verification — all commands verbatim (test totals + the two probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
