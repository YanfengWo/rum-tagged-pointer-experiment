# brief-389 — review: frame root traversal completeness (1 of 4)

A consistency review of recent work, in four independent parts. This is part
one. **Scope: NONE — review only, no source change.** Reading, grepping and
throwaway probes are in scope; leave no probe files in the repository.

**Write your findings section first, before running any verification
commands.** If the run is interrupted the findings are what matter, so they
must already be on the page. Keep each numbered section self-contained.

## Background

Briefs 364-375 replaced closure-captured suspension state with named structs
whose `Expr` traversal is derived from their fields, so that a type entering
frame state without a traversal fails to compile. The supporting pieces:

- `SuspensionFrame: FrameRootTrace` makes a missing traversal a compile error.
- The frame queue is private, so there is one insertion route
  (`resume_or_suspend!`).
- A repository invariant test holds handwritten traversals to a short
  allowlist, `MANUAL_FRAME_ROOT_TRACE_IMPLS` in
  `crates/rum-runtime/src/runtime/suspension/trace.rs`, whose entries are meant
  to be types structurally incapable of holding a root.
- The inventory lives in
  `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`.

Later work (briefs 377-388) added frame and continuation types on top of this.
Two allowlist entries were added in that period: a bare `fn` pointer alias, and
whatever else you find.

## What to check

Report each as a finding with `path:line` evidence, or state plainly that you
checked it and found nothing.

1. **Can any type reach frame state without a derived traversal?** The claim is
   that this is a compile error. Test the claim rather than trusting it: find
   the weakest point — a generic parameter, a boxed trait object, a type behind
   an indirection the derive does not descend into — and say whether it holds.
2. **Is every allowlist entry still structurally unable to hold an `Expr`?**
   Check each entry as it stands today, not as it was justified when added. A
   `fn` pointer captures nothing; a struct that gained a field since its entry
   was written may no longer qualify.
3. **Is the queue still the single insertion route?** Look for any path that
   places a frame without going through the macro, including test-only helpers
   that might have widened visibility.
4. **Does the inventory match reality?** The invariant test compares against a
   recorded list. Confirm the list is complete for the types that exist now —
   in particular the continuation types added by briefs 380-383 — and that no
   entry names a type that no longer exists.
5. **Are the derived traversals correct for the composite cases?** A field
   holding `Option<Expr>`, `Vec<Expr>`, `SharedArgs`, a nested struct, or a
   collection of nested structs must all be walked. Name any field the derive
   descends into incorrectly or skips.

## What this review is not

Do not propose refactors, file splits, or performance changes. The question is
whether the stated invariant holds, not whether the code could be nicer. If you
find nothing, say so — a clean result is a useful outcome and does not need to
be padded with suggestions.

## Verification

After the findings are written:

```
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime frame_root_trace
```

Paste the result. If a finding needs a probe to confirm, write the probe under
`/tmp`, run it, paste the output, and delete it.

## Hard rules

- Do NOT modify any file in the repository. Do NOT `git commit`, branch, push,
  or touch git config. The worktree must be clean when you finish.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- Report only what you verified. If you suspect something but could not confirm
  it, say which and what would settle it.

## Report format

1. Findings — numbered, each with `path:line` evidence and a one-line statement
   of what would go wrong. Write this section first.
2. Checks that came back clean, listed briefly.
3. Verification output.
4. Anything you could not settle.
