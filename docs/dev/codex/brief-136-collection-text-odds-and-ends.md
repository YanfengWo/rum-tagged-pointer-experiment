# Brief 136 — Collection/text odds-and-ends (stdlib batch)

Implementation brief. First batch of the stdlib-expansion mainline
(`TODO.md` §Stdlib Expansion For Scripting), sized directly from the
brief-135 friction list. Folk scope: simple, exact-on-common, documented
limits; behavior is rum's own contract.

## Goal

Eleven new builtins registered in the appropriate `rum-stdlib` families with
catalog specs, tests across the coverage axes, regenerated reference (drift
green), and a green workspace: `Tally`, `MaxBy`, `MinBy`, `TakeLargestBy`,
`TakeSmallestBy`, `ReverseSort`, `ReverseSortBy`, `GatherBy`, `Riffle`,
`Mean`, `StringPadRight`, `StringLines`. (Twelve names; `StringPadRight`
mirrors the existing `StringPadLeft`.)

## Semantics (rum's contract — keep it this simple)

- `Tally[list]` → `{{elem, count}, …}` in first-occurrence order; structural
  sameness (`SameQ`) decides equality. `Tally[{}]` → `{}`.
- `MaxBy[list, f]` / `MinBy[list, f]` → the FIRST element attaining the
  max/min of `f` under the canonical ordering used by `Sort`. Empty list →
  message (mirror the invalid-arg style of the family) + inert.
- `TakeLargestBy[list, f, n]` / `TakeSmallestBy[list, f, n]` → the `n`
  elements largest/smallest by `f`, ordered by descending/ascending `f`,
  stable for ties; `n` ≥ length → whole list sorted accordingly.
- `ReverseSort[list]` = `Reverse[Sort[list]]`; `ReverseSortBy[list, f]` =
  `Reverse[SortBy[list, f]]` (stable accordingly).
- `GatherBy[list, f]` → groups (lists) in first-occurrence order of the key;
  key equality by `SameQ` on `f[elem]`.
- `Riffle[list, sep]` → single-separator form only (folk); document that the
  cyclic/list-separator forms are out of scope.
- `Mean[list]` → `Total[list] / Length[list]`, exact; empty list → message +
  inert; non-numeric elements simply flow through exact arithmetic (the
  result may stay symbolic), same spirit as `Total`.
- `StringPadRight[s, n]` / `[s, n, pad]` → mirror `StringPadLeft`'s existing
  contract exactly (truncation behavior included — read its implementation
  and tests first).
- `StringLines[s]` → split on `"\r\n"` or `"\n"`; a single trailing newline
  yields no trailing empty string; preserve interior empty lines. Document
  exactly this.

Heads with obvious existing homes: combinatorics/structure for the list
ones, strings for the two string ones. Attributes: `Protected` (match the
family norm); no Listable unless the family norm clearly says so.

## Scope

- MAY modify: `crates/rum-stdlib/src/**` (implementations, catalog specs,
  tests), `docs/reference/` via the generator only.
- MUST NOT touch: `rum-runtime`/`rum-core`/`rum-host`, guide chapters,
  scripts, `Cargo.*`.

## Tests

Per builtin, cover the axes that matter (canonical / boundary: empty,
singleton, ties / held-symbolic flow-through where relevant / invalid arity
+ message + inert shape). Reuse the family test-file layout. The brief-135
task scripts are good end-to-end shapes: add one integration-style test that
reproduces the "top-3 counts" idiom
(`Tally` + `TakeLargestBy`) and one exact-`Mean` aggregation.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference
rg -c '^### ' docs/reference/builtins.md          # expect 518
cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
# paste probes for each new builtin (one canonical + one boundary each)
cargo run -q -p rum-cli -- --eval 'TakeLargestBy[Tally[Characters["mississippi"]], Last, 3]'
cargo run -q -p rum-cli -- --eval 'Mean[{1, 2, 1/2}]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- No changes to existing builtins' behavior; additions only.
- No new `unsafe`. Not an eval-hot-path change (no microbench gate needed);
  if you find yourself editing dispatch/eval code, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file; the family each builtin landed in.
2. Verification — all commands verbatim, incl. per-builtin probes.
3. Any semantics you had to decide beyond this brief (list them — they go
   into the reference limits notes if non-obvious).
4. Stop-loss / open questions (empty if none).
