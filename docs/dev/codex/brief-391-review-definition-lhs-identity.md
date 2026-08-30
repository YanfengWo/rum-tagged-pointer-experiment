# brief-391 — review: definition left-hand-side identity (3 of 4)

Part three of four independent consistency reviews. **Scope: NONE — review
only, no source change.** Reading, grepping and throwaway probes are in scope;
leave no probe files in the repository.

**Write your findings section first, before running any verification
commands.** Keep each numbered section self-contained.

Parts one and two each returned something useful, and in both cases part of the
value was correcting the background I wrote. **Do that again if this background
is wrong — that is a finding, not a digression.**

Part two found that a typed value described as constructible only two ways
could in fact be built from raw expressions through an exported decoder. **The
same question applies here**, to different types, and is question 2 below.

## Background

Briefs 379-383 addressed eight defects sharing one invariant: **the identity a
definition is stored under must come from the same attribute-aware preparation
as the identity a lookup constructs.** Violations ran in both directions —
four positions failed to prepare when they should have, two prepared a position
that had to keep its raw symbol, and one needed the `Flat`/`Orderless`
normalization the ground index was not applying.

The resulting arrangement:

- `PreparedDefinitionLhs` prepares an lhs by the head's hold attributes: active
  positions evaluate once, including delayed own-values; held positions keep
  their raw subtree. Pattern regions have their own policy (binder names stay
  raw, `PatternTest`/`Condition` predicates stay deferred, `HoldPattern` and
  friends are opaque).
- Storage and removal travel as tokens whose constructors are private to
  `definitions/definition_lhs_preparation/prepared_store.rs`, so the intent is
  that a raw lhs from source-language evaluation cannot be stored.
- Two paths deliberately do not prepare: trusted static catalog installation
  (`session/session_api.rs`), and whole-state restoration, which restores an
  already-built definition and runs no user code.
- A stored identity is a state snapshot: later own-value or attribute changes
  affect future preparation, never existing definitions.

## What to check

Report each as a finding with `path:line` evidence, or state that you checked
it and found nothing.

1. **Does the invariant hold in both directions today?** Construct cases where
   a position must prepare and where it must stay raw, and confirm the stored
   identity matches what a lookup builds. Include a head whose attributes
   changed after the definition was installed, since preparation reads
   attributes at store time.
2. **Can a store or removal token be constructed outside its module?** Part two
   found exactly this pattern elsewhere. Check the export surface —
   `kernel_api`, `#[doc(hidden)]` re-exports, anything reachable from
   `rum-stdlib` or an external crate — and say whether a token can be built
   from parts, decoded from a continuation value, or otherwise obtained without
   going through preparation.
3. **Are the two non-preparing paths still narrow?** Confirm each is reachable
   only as described, and that neither has acquired a second caller that
   handles source-language input.
4. **Is the pattern-region policy complete?** Preparation must never apply a
   rule to pattern syntax. Name any wrapper or nested shape where an active
   position inside a pattern region could be evaluated.
5. **Does preparation ever apply the definition being installed?** The design
   requires stopping before outer dispatch. Look for a path — a callable head,
   a nested active argument, a resumed preparation — where the outer lhs could
   be applied to itself.

## What this review is not

Do not propose refactors, file splits, or performance changes. If you find
nothing, say so — a clean result is a useful outcome and does not need padding.

## Verification

After the findings are written:

```
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime definition_lhs_preparation
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime definition_store_sealing
scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime definitions_tagged_preparation
```

Paste the results. If a finding needs a probe, write it under `/tmp`, run it
with `stdin < /dev/null`, paste the output, and delete it.

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
4. Anything you could not settle, including any error in this brief's
   background.
