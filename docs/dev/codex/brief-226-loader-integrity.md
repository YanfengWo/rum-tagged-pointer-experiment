# Brief 226 — BeginPackage fails closed on a failed dependency load

Fix brief — unit-10 batch 3 (safety, M). Source:
`docs/dev/usability-pass/kernel-c-findings.md` (top-gap #4 + the
host-isolation row #5, Evidence E3). Builds directly on brief-225
(a5cdecd): that batch added a dependency-failure rollback that fires ONLY
for the capability-denied case; this batch generalizes it to ANY failed
dependency load, closing the half-entered-package hole.

## The defect (gate-reproduced on unrestricted CLI)

```
BeginPackage["FailMain`", {"NoSuchDep`"}]
! Get::notfound: could not locate `NoSuchDep`` …
! Needs::load: context `NoSuchDep`` could not be loaded
{$Context, $ContextPath, $Packages}
→ {"FailMain`", {"FailMain`", "NoSuchDep`", "System`"}, {"FailMain`"}}
```

A missing dependency emits the loader messages but the `$Failed` return
from `eval_needs` is IGNORED: `BeginPackage` still enters `FailMain\``,
puts the un-loadable `NoSuchDep\`` on `$ContextPath`, and records
`FailMain\`` in `$Packages`. The session is left in a half-loaded,
misleading state — a package that "opened" while its declared dependency
does not exist.

## The seam (post-225)

`eval_begin_package` (`builtins_context_loader.rs:335-357`) loops the
dependencies calling `self.eval_needs(...)`. brief-225 added a rollback
block, but its trigger is the capability check
(`!allows(FileRead)`) — so it fires for a sandbox denial but NOT for a
missing file, because under the unrestricted CLI policy the branch is
skipped and the loop falls through into the package.

## Required change

- **Generalize the rollback trigger to any `$Failed` dependency return.**
  A dependency `eval_needs` that returns `$Failed` (for ANY reason —
  missing file, no provider, capability denial) rolls back the
  just-opened package frame, restores `current_context`, `context_path`,
  and `loaded_packages` to their pre-`BeginPackage` values, and returns
  the `$Failed` (with the loader messages already emitted). Refactor the
  225 rollback body into a single helper and call it from the one
  `$Failed` check — the capability-denied case becomes a subset, so the
  225 behavior is preserved byte-identically and the missing-file case
  now fails closed the same way.
- **Preservation pins (baseline-verify before AND after):**
  - a well-formed `BeginPackage["P\`", {"Dep\`"}]` with `Dep\`` present
    loads and enters the package exactly as today (the dependency chain
    `mainVal` probe from brief-225's gate — reproduce it);
  - `BeginPackage["P\`"]` with no dependencies unchanged;
  - the 225 capability-denial rollback under
    `sandboxed().with_file_io` unchanged (the existing rum-host test must
    still pass — it now goes through the generalized path);
  - a SECOND dependency failing after a FIRST succeeded also rolls back
    fully (test `{GoodDep\`, NoSuchDep\`}` — the good dep's
    `loaded_packages` entry is truncated back).
- After a failed dependency, `$Context`/`$ContextPath`/`$Packages` equal
  their pre-call values and the target context is NOT created.

## Deliberately OUT of scope — record, do not fix

- **Unbalanced `Begin` inside a `Get`** (finding row #5):
  `Get["file-with-unbalanced-Begin"]` leaves the new context active after
  the load (`$Context` = the leaked context). WL does NOT restore context
  around `Get` either — a loaded file is responsible for balancing its
  own `Begin`/`End`, so matching WL means LEAVING this. The real
  request-boundary isolation is per-request rollback, which belongs to
  the dun host-embedding line (already on its gap list). Add a
  one-paragraph note to the findings' host-isolation row pointing at the
  host line; change NO `Get` behavior. (If you find the leak is LOUDER
  than WL — e.g. it corrupts a later request in the same session — report
  that as a separate finding; do not fix it here.)

## Method

1. Reproduce the defect + all preservation pins on the pre-change tree
   (baseline-verify, the standing lesson).
2. Refactor the 225 rollback into a helper; switch the trigger to
   `is_system_symbol_name(&dependency_result, "$Failed")`.
3. Tests: the missing-dependency fail-closed case (state restored, target
   context absent); the second-dependency-fails case; the well-formed
   load unchanged; the 225 capability test still green.

## Perf gate + acceptance

- **No microbench A/B** — the change is confined to the BeginPackage
  dependency-failure branch (not generic evaluation, not a hot path).
  State this in the report with the reasoning.
- **mma-hol Strict gate at MY gate** — this changes `BeginPackage`
  behavior; the director runs the full acceptance suite before crediting.
  Note it; do not run it yourself. (A well-formed suite never hits the
  failed-dependency path, so no behavior change is expected there — but
  the gate confirms it.)

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes (use /tmp scratch .wl), before/after:
#  - BeginPackage["FailMain`", {"NoSuchDep`"}] then {$Context, $ContextPath,
#    $Packages} — after: all pre-call values, messages still emitted
#  - well-formed dependency load unchanged (mainVal chain)
#  - second-dependency-fails rollback
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/builtins_context_loader.rs`
  (the dependency loop + the extracted rollback helper)
- Tests: runtime loader tests + the rum-host `sandboxed…with_file_io`
  test file (the 225 case now runs through the generalized path — verify,
  extend); list each; one new test file allowed
- `docs/dev/usability-pass/kernel-c-findings.md` — ONLY to append the
  one-paragraph host-line pointer to row #5 (design-input note; no other
  findings edits)

MUST NOT touch: `eval_get`/`eval_needs`'s own logic (225's seam is
settled), the capability guard, `Get`'s context behavior (the deferred
row), the dispatcher, `Save`/`DumpSave`, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`, no capability changes.
- Well-formed loads and the 225 capability-denial behavior are
  byte-identical; baseline-verify every preservation pin.
- WL comparison from documented knowledge only; the unbalanced-Begin row
  is deferred, not guessed-and-fixed.
- Stop-loss: if generalizing the trigger breaks the 225 capability test
  or any well-formed load, STOP and report.

## Report format

1. The change: the generalized trigger, the extracted helper, why 225's
   case is a byte-identical subset.
2. Probe panel before/after verbatim (fail-closed state restore;
   well-formed unchanged; second-dep rollback).
3. Test survey old→new.
4. Verification outputs verbatim; the no-A/B justification; a note that
   the mma-hol Strict gate is the director's.
5. Deviations / open questions (empty if none).
