# Brief 289 — capability-classification & limit-coverage consistency audit (diagnostic)

## Goal

Produce one evidence-backed findings report that answers a single engineering-hygiene
question across the whole builtin surface: **does each builtin's declared `capability`
field — and the session's declared resource limits — match what the implementation
actually does?** The generated builtin reference documents each builtin's capability,
so any drift between the declared label and the runtime effect is a documentation
defect to record ahead of publishing that reference. This is a **diagnostic audit: no
source changes.** The deliverable is the findings file, nothing else.

This is the same shape as the kernel usability audits (declared-label vs actual-behavior,
e.g. the earlier `Exit` Pure→Process reclassification and the `TestReport` Clock
decision). Match that register.

## Context pointers

- Read `AGENTS.md` (operational notes + Testing Policy hard lines) before anything.
- Capability model: `crates/rum-runtime/src/runtime/capabilities.rs` — eight capabilities
  (`Pure`/`FileRead`/`FileWrite`/`Clock`/`Random`/`Diagnostic`/`Process`/`Interactive`);
  `CapabilityPolicy` is a default-deny allowlist with an `unrestricted()` trusted default.
- The single dispatch checkpoint: `crates/rum-runtime/src/runtime/builtins_context_loader.rs:68`
  — `if self.capability_policy.is_unrestricted() || self.capability_policy.allows(capability) { … }`.
  This is the one place a builtin's declared capability is consulted before the builtin runs.
- Each builtin declares its capability in its `BuiltinSpec` (`.capability` field); the catalog
  is what `core_builtin_specs()` enumerates. Reference rendering: `crates/rum-stdlib/src/docgen.rs`.
- Session resource limits: the session config / `crates/rum-runtime/src/runtime/session/session_api.rs`
  (`set_eval_timeout`, plus the recursion / iteration / parse-depth / match-depth limits and the
  memory ceiling — locate their setters and the points that enforce them).
- Prior audit of this exact shape to imitate: `docs/dev/usability-pass/kernel-a-findings.md`.
- Acceptance is mechanical: the findings file with the enumerated tables + the three grep
  re-verifications, pasted verbatim. Not prose.

## Scope

- File you MAY create/modify: **`docs/dev/publication-audit/capability-consistency-findings.md`**
  (create the directory + file; this file IS the deliverable) — and nothing else.
- MUST NOT touch: all source, all tests, any other doc, `.git`, git config, `Cargo.lock`.
  **No source change of any kind.** Where a label and behavior disagree, you RECORD it — you
  do not fix it. Claude decides every reclassification (some impure-looking labels are correct
  by design: a diagnostic or timing builtin legitimately reads a clock).

## Deliverable — four sections, each WRITTEN TO THE FILE THE MOMENT IT IS COMPLETE

**Write each section to disk incrementally as you finish it. Do NOT hold the whole report in
memory and write once at the end — partial progress must survive on disk.** Start the file with
its first section before moving on.

1. **Capability-classification table.** Enumerate every builtin from the catalog with its
   declared `capability`. For each, state in one line the external/impure effect the
   implementation actually performs (reads wall-clock? reads fs via the provider seam? draws
   randomness? requests host input? requests host termination? mutates process/global state?
   or none = genuinely pure). Flag every row where the declared label does not match the actual
   effect, in EITHER direction:
   - declared `Pure` but the code reaches a clock / fs / random / host / global-mutable effect
     (label too weak — the reference under-states what the builtin touches);
   - declared with an effect capability but the implementation performs no such effect
     (label too strong — needlessly restrictive).
   For each flagged row give the `file:line` of the spec AND of the effecting code.

2. **Gate-coverage table.** The checkpoint at `builtins_context_loader.rs:68` is the single
   place the declared capability is consulted. Enumerate the dispatch entry points through which
   a capability-bearing builtin's effect can be reached, and confirm per path that the effect is
   downstream of that checkpoint — i.e. a policy that does not allow the capability actually
   prevents the effect from running. Flag any path that reaches an effect without passing the
   checkpoint. (A prior instance of this class was a loader `Get`/`Needs` path that reached a
   file read before the check ran; confirm whether any analogous path exists today.)

3. **Limit-coverage table.** Enumerate the session resource limits (recursion depth, iteration
   count, parse depth, match depth, eval wall-clock timeout, memory ceiling). For each, name the
   builtin(s) that can reach that resource and confirm the limit actually bounds them —
   including under an extreme argument value. Flag any builtin that can drive a resource past its
   configured limit, or into a host-process fault, because an argument is consumed before the
   limit clamps it (e.g. a duration or size argument that overflows the arithmetic ahead of the
   ceiling check).

4. **Hard-line re-verification (three checks, pasted verbatim into the file AND the report).**
   - `unsafe` occurrences in `crates/rum-core` and `crates/rum-runtime` (expected 0; the only
     `unsafe` in the repo is the documented off-by-default alloc-count `GlobalAlloc` in `rum-cli`).
   - direct `std::fs` and `std::net` usage in non-test code across the workspace (expected: fs
     appears only behind the `FileIo` provider seam, never as a direct call; net absent).
   - a determinism spot-check of `Capability::Pure` builtins: list any `Pure` builtin whose
     implementation touches wall-clock, RNG, environment, or process-global mutable state.

## Method notes (read twice)

1. **Enumerate from the catalog programmatically** (a small `python3`/grep pass over the spec
   source), THEN cross-check a sample against live `rum-cli` probes. A script-derived list can
   under-count (backtick quoting, multi-line entries, `pub(super)` visibility have all fooled
   extraction before) — state your extraction method and sample-verify the count before trusting it.
2. **Read the code path to confirm an effect; do not infer it from the name.** A name like
   `FileNameJoin` sounds like I/O but is pure string manipulation; a `Pure` name may still reach
   a global. Confirm at the source.
3. **One finding = evidence on both sides.** A flagged mismatch needs the spec `file:line` and
   the effecting-code `file:line`, plus a one-line proposed reclassification for Claude to rule on.
4. **Do not summarize rows away.** If a table is large, still write the full table to the file.
5. **Stop-loss is real.** If enumeration or a probe fails twice for the same reason, STOP and
   write what blocked you in the report — do not thrash.

## Verification (you run it; the evidence is the deliverable)

There is no source change, so there is no source test gate. Confirm instead:

- All four sections are present in `docs/dev/publication-audit/capability-consistency-findings.md`.
- The three hard-line checks are pasted verbatim in the file AND in your final report.
- At the very end only — AFTER all four sections are on disk — run once and paste the summary line:

```
cargo fmt --check
cargo test --workspace
```

(You changed no source, so the tree you leave must still be green; this only confirms you left it
untouched.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the findings file in the working
  tree; Claude runs the authoritative gate.
- Do NOT modify any file except the one findings file named in Scope. If you believe you must,
  STOP and write that in the report instead.
- Touch nothing outside this repo; no network. Commands: file edits within Scope, `cargo`, the
  release `rum-cli`, and `python3` / grep for enumeration and counting.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's own documented behavior;
  no black-box comparison against other vendors' engines, no oracle harness, no compatibility
  matrix, no third-party engine output committed.
- **No source change**, therefore no `unsafe` question arises — you are reading and reporting.
- **Write the findings to disk incrementally as each section completes.** The deliverable must
  never exist only in your final message.
- Paste real verbatim output. If a run did not complete or a table could not be finished, say so
  explicitly — never fabricate a row or a count.

## Report format (your final message)

1. What was produced — the findings file path and its section list ("diagnostic only, no source
   change").
2. Verification — the three hard-line checks verbatim + the final `cargo test --workspace`
   summary line.
3. The findings — the flagged rows from each of the four tables (label-vs-behavior mismatches,
   any gate- or limit-coverage gap), each with `file:line` evidence on both sides and a one-line
   proposed reclassification for Claude to decide.
4. Stop-loss / open questions — where you stopped and why; anything you could not settle (empty
   if none).
