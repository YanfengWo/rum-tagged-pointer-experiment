# brief-383 — tagged definitions, `TagUnset`, and sealing the raw store

Step 3 of plan-378, the last one. Landed: 1a primitives (`5698e8e5`), 1b
preparation with `Set`/`SetDelayed` (`aa441ff0`), 1c `Unset` and value tables
(`45c5aa33`), 2 the association fallback (`e90fcc62`). Six of family B's seven
defects are closed. This step closes B2 and then makes the whole class
unrepresentable.

## Goal, part one — B2

`TagSet`, `UpSet` and their delayed forms store a raw lhs, so the rule they
install is unreachable from any input. **All six install forms still reproduce
on the current tree (`e90fcc62`)**, director's re-probe:

```
tg = 5;  h /: h[tag1, tg] = 7
  stores {HoldPattern[h[tag1, tg]] :> 7};  h[tag1, 5] does not match
tgD = 5; hd /: hd[tagd, tgD] := 17          (TagSetDelayed, down-value)
vv = 5;  g /: k[g, vv] = 11                 (TagSet, up-value)
vvD = 5; gD /: kD[gD, vvD] := 21            (TagSetDelayed, up-value)
ww = 5;  m[ww] ^= 22                        (UpSet)
wwD = 5; mD[wwD] ^:= 32                     (UpSetDelayed)
```

Working controls that must stay working: a pattern lhs
(`cc /: cc[ctag, qq_] = 88` reaches `cc[ctag, 9]`), and a `HoldFirst` head
(`hh /: hh[hv, htag] = 51` reaches `hh[hv, htag]`) — held positions keep raw
symbols under the settled contract.

### `TagUnset` must change in the same step, and here is why

`TagUnset` currently works: `iT = 3; tu /: tu[tuT, iT] = 77; tu /: tu[tuT, iT] =.`
does remove the definition. **That is a coincidence, not correctness** — both
sides use an unprepared lhs, so two raw forms happen to be equal. The moment
`TagSet` prepares, a `TagUnset` that does not will stop finding what was just
stored.

This is exactly the shape 1c hit: `Unset` on a held position "worked" only
because 1b had made `Set` store the raw held form. Same trap, one step later.
`TagUnset` prepares its lhs under current state and then classifies against the
explicit tag, and the snapshot contract applies to it as it does to `Unset`.

### The ruling that applies here

**User ruling, 2026-08-28: keep rum's `UpSet::nosym`.** Once `m[ww] ^= 22` with
`ww = 5` prepares to `m[5]`, no symbol in the lhs can serve as a tag. rum
already reports exactly that for the literal `m2[7] ^= 33`
(`UpSet::nosym: No valid tag symbol occurs in the assignment lhs m2[7]`), and
the prepared form must reach the same diagnostic and install nothing. The wider
WL family reaches a differently-shaped message there; rum keeps the more direct
one and stays internally consistent. Design input only — no engine comparison
enters the repository.

### Ordering

Per plan-378: `TagSet` normalizes its explicit tag first; immediate forms
evaluate the RHS next, preserving current order; the lhs is prepared after the
RHS, and delayed forms skip RHS evaluation; ownership is classified from the
**canonical** lhs, and `UpSet` infers its tags from that.

## Goal, part two — sealing

This is the part that outlasts the bug. Finish the migration so that **storing
a raw, unprepared lhs from source-language evaluation is not expressible**,
rather than merely absent:

- Replace the `(target, lhs)` storage APIs with a sealed store type carrying
  both the canonical lhs and the classified owner; removal consumes a
  corresponding prepared removal token.
- **Prefer a compile-time privacy boundary over a call-site inventory test.**
  A list someone must maintain is the weaker form of this repair; a private
  constructor that only preparation can call is the stronger one. If both are
  achievable, do both, and say which one actually carries the guarantee.
- Exactly two non-preparing paths remain legitimate, and both must be explicit
  and narrow: trusted static system-catalog installation
  (`session/session_api.rs:203`), and whole-state restoration (Block/snapshot
  cloning, which restores an already-built `SymbolDefinition` and runs no user
  code).
- The staged migration allowlist that 1b-2 carried for the not-yet-migrated
  consumers must end this step **empty of source-language consumers**.

Report the final call-site inventory: every store and removal site, and what
guarantees it is not raw.

This project's standing preference is explicit: **a repair the compiler or a
derive enforces beats one that relies on people remembering, even at an order
of magnitude more cost.** The frame-roots line is the model — a missing
traversal became a compile error, the queue became private, and the invariant
test holds handwritten cases to a justified allowlist.

## What must not change

Everything 1a-2 fixed. In the differential gate these must be byte-identical:
B3 and B4, held ground `Flat`/`Orderless`, `Unset`, value-table assignment, and
the `AssociateTo`/`KeyDropFrom` fallback. A regression there is a stop, not a
diff line to justify.

Also unchanged: `HoldAllComplete`'s suppression of up-value dispatch is
documented behavior (`docs/guide/evaluation.md:111`, tested at
`holding_and_control.rs:277`), not a defect to fix here.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_clear.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs`
  and its resume-frame module
- `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs`
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`
- `crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs`
- `crates/rum-runtime/src/runtime/definitions/mod.rs`
- `crates/rum-runtime/src/runtime/session/session_api.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-stdlib/src/associations/mutation.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- the runtime and stdlib test modules covering tagged definitions and the
  sealing boundary
- `docs/guide/evaluation.md`

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple of an
in-scope change — an exhaustive `match` arm forced by a new variant, an
exhaustive literal forced by a changed field set, a call site forced by a
changed signature, a `FrameRootTrace` derive forced by a type entering frame
state — you MAY edit unlisted; list each under "mechanical ripple". Anything
changing behavior, contract, or a test expectation in an unlisted file: STOP
and report. When unsure, it is the second kind.

Sealing will touch call sites this list does not name — that is expected and
covered by the mechanical-ripple rule as long as the change is a signature or
visibility consequence. A call site whose *behavior* must change to compile is
not mechanical: report it.

Out of scope: `Expr`/`Symbol` representation, public serde shapes,
`Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, other `docs/**`.

## Suspension safety

- Reuse 1b's zipper; do not fork a second preparation path.
- `TagSet`/`UpSet` can suspend in the tag, in the RHS, and in the lhs; prove
  exactly-once with counters, and no replay after resume.
- Every new state/frame derives `FrameRootTrace` and is listed in
  `runtime/tests/frame_root_trace/derived_inventory.rs`; frames are inserted
  only through `resume_or_suspend!`; the queue stays private. A new manual
  `FrameRootTrace` impl needs the same justification step 2's did — structural
  inability to hold a root, not convenience.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The bounded differential gate

**Non-empty in exactly the expected places, empty everywhere else, every
differing line justified.**

- Reuse and extend the probe from `/tmp/rum-brief-382-*/`.
- **Calibrate first** against a pre-family-A binary and show it reports a diff.
- Then diff `HEAD` against your candidate.

### Test matrix

- All six install forms, across down-, up- and sub-value ownership.
- `m[ww] ^= 22` with `ww = 5` reports `UpSet::nosym` and installs nothing.
- Pattern symbols, typed blanks, and held positions stay raw and reachable.
- All five hold attributes, partial holds, nested and callable heads.
- `TagUnset` under unchanged state removes the canonical rule; after an
  own-value rebind it does not remove the earlier identity; after clearing that
  own-value it addresses the raw identity. `Clear` still removes everything.
- Multiple inferred `UpSet` tags keep current protection behavior.
- Both `/:` and operator syntaxes reach the changed entry points.
- Sealing: a test that the non-preparing constructors are unreachable from
  source-language evaluation — ideally by construction, i.e. the code does not
  compile if someone tries.

**Assert after the invalidating action**, not only before: after
`SetAttributes` on an already-defined head; after the lhs own-value is rebound
and then cleared; after `Clear` of the owning head or tag. Say per test which
invalidation it acts after, and name the entry point per form.

### Performance gate

- rum-vs-rum A/B, one variable, profiling OFF, same script both binaries,
  **best-of-3 per binary**; report each label's three-run range.
- **Controls apply**: this change is on the definition-store path, not the
  common call path, so the pure-computation labels are a genuine control
  group. Report the control median beside the affected median.
- Targets: `downvalue-*`, `upvalue-*`, `subvalue-*`, `set-ownvalue`.
- If a label looks like a regression but the disassembly of the affected
  function is unchanged, **build a third binary whose only difference is an
  inert `#[inline(never)]` dead function in an unrelated file** and measure it
  too. Step 2 had two labels move +4.94%/+7.63% that way; the inert arm moved
  them +2.60%/+5.10%, further than the real change, settling them as
  layout-sensitive. One extra build, and conclusive.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`. `rum-core` and `rum-runtime` are zero-`unsafe`.
- No unconditional cost on the ordinary evaluation path.
- No owned-tree deep clones; `Expr`/`SharedExpr`/`SharedArgs` are `Arc`-shared.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed. WL
  semantics may inform design and be described in prose only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape.
- **Stop-loss** — report rather than improvise if: sealing cannot be achieved
  without a behavioral change at a call site; the two legitimate non-preparing
  paths cannot be made narrow without widening an allowlist you cannot justify
  structurally; a 1a-2 differential line regresses; or a differing line appears
  you cannot justify.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one-line why.
2. Mechanical ripple — every file edited under the criterion above.
3. **The final call-site inventory** — every store and removal site, and what
   guarantees each is not raw. State plainly which guarantee is compile-time
   and which is a test.
4. Verification — `fmt`, workspace counts, the calibration, the bounded diff
   with every differing line justified, and the exactly-once counters.
5. The A/B — control median, affected median, three-run ranges, and the inert
   arm if you needed one.
6. Stop-loss / open questions.
