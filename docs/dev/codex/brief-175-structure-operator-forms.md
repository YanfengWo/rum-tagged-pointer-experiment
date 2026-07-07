# Brief 175 — Unit-2 operator-forms thaw batch (14 symbols)

Implementation brief. The unit-2 operator-form batch, thawed now that
the perf moratorium is lifted (tracker `docs/dev/usability-pass.md`,
unit-2 THAWED note; evidence E1/E4 in
`docs/dev/usability-pass/structure-functional-findings.md`). Extends
the EXISTING callable-wrapper mechanism exactly as brief-164 did for
the string eight — same chain, no new dispatch path.

## The symbols and their operator arities

Disambiguation rule (STATE it in code comments where the arity lists
live): an arity becomes an operator form ONLY if it cannot be a valid
data-form call of the same builtin. No first-argument type sniffing,
ever.

One-argument operator forms (data forms all take the data as argument
one, so one argument is unambiguous):

- `Take[spec]`, `Drop[spec]`, `Delete[pos]`, `TakeList[spec]`,
  `Partition[n]`, `Riffle[x]`, `GatherBy[f]` (also `GatherBy[{f1,…}]`),
  `MaxBy[f]`, `MinBy[f]`, `DeleteDuplicatesBy[f]`, `ReverseSortBy[f]`.

Two-argument operator forms (their data forms are exactly-3-argument,
so two arguments are unambiguous):

- `Insert[x, pos]`, `TakeLargestBy[f, n]`, `TakeSmallestBy[f, n]`.

EXCLUDED on ambiguity grounds (do NOT add): `Partition[n, d]` and any
other arity that collides with a data form.

Semantics: `Foo[params…][data]` ≡ `Foo[data, params…]`. The
operator-arity direct call evaluates to ITSELF, inert, message-free
(today most emit `argx`/`argtu` — those messages disappearing for
exactly these arities is the intended change). Application rewrites to
the ordinary call; validation happens there, exactly as the ordinary
call would.

## Mechanism

Extend the callable-wrapper chain in
`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs` (the
brief-164 pattern: recognition + apply-rewrite + argx arbitration).
Builtin-side: exempt the operator arities from the arity messages so
the inert form is message-free (the brief-164 change shape in
`strings/patterns/search.rs`, applied to the structure_ops/
functional_ops arity checks of the 14).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Take[2][{a, b, c}], Drop[1][{a, b, c}], Delete[1][{a, b}], TakeList[{2, 1}][{a, b, c, d}], Partition[2][{a, b, c, d}], Riffle[x][{a, b}], GatherBy[OddQ][{1, 2, 3}], MaxBy[Abs][{1, -3, 2}], MinBy[Abs][{1, -3, 2}], DeleteDuplicatesBy[Abs][{-1, 1, 2}], ReverseSortBy[Identity][{3, 1, 2}], Insert[x, 2][{a, b}], TakeLargestBy[Abs, 2][{1, -3, 2}], TakeSmallestBy[Abs, 1][{1, -3, 2}]}'
# inert message-free operator forms + ordinary calls byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{Take[2], Insert[x, 2], Take[{a, b, c}, 2], Delete[{a, b}, 1], Partition[{a, b, c, d}, 2], TakeLargestBy[{1, -3, 2}, Abs, 2]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 all
evaluate to the ordinary-call results (paste — review evidence);
probe 2 shows two inert forms with NO messages and the ordinary calls
byte-identical to today.

## Microbench A/B (MANDATORY — the moratorium is lifted; this gate is real again)

The wrapper chain gains one more recognition arm consulted from call
planning. Run the alternating protocol:

1. Build the release binary from the CURRENT tree (candidate) and from
   `HEAD` (base, via `git stash` — restore after); save BOTH binaries
   aside.
2. `scripts/core-language-microbench.sh --rum-cli <saved binary>`
   alternating base/candidate, three runs each (B C B C B C).
3. Report per-label best-of-3 deltas and each side's self-noise band;
   call out any label whose |delta| exceeds its own noise band. State
   explicitly whether the new arm is consulted on symbol-head calls or
   only compound-head calls.

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`,
  the arity-message paths of the 14 in
  `crates/rum-stdlib/src/structure_ops/**` and
  `crates/rum-stdlib/src/functional_ops/**`,
  `crates/rum-stdlib/src/catalog/structure.rs` and
  `crates/rum-stdlib/src/catalog/functional.rs` (descriptions only),
  `crates/rum-stdlib/src/docgen.rs` (LIMITS notes if stated),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  kernel wrapper tests under
  `crates/rum-runtime/src/runtime/tests/**`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: other builtins' behavior, `Cargo.toml` files, the
  guide, `TODO.md`, the tracker, findings files.

## Hard rules

- Do NOT `git commit`, branch, push (the stash in the A/B step must
  end restored — `git stash list` empty, tree = your changes). No
  network, no new dependencies, no new `unsafe`.
- Behavior changes are EXACTLY: (a) the listed operator arities become
  inert message-free forms, (b) applying them rewrites to the ordinary
  call. Everything else byte-identical.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; confirm the arity-exemption list matches
   the disambiguation rule.
2. Verification — all commands verbatim.
3. Microbench A/B — the full per-label table or its worst rows, noise
   bands, and the consultation-path statement.
4. Stop-loss / open questions (empty if none).
