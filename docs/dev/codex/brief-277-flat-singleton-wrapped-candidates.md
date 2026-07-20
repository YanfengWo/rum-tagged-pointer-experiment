# Brief 277 — non-OneIdentity `Flat` singleton wrapped candidates

Implements what was brief-276's Part B. That attempt stop-lossed
CORRECTLY and cleanly, before any source edit: the `one_identity`
threading brief-276 prescribed (compute in `core.rs`, thread down)
reaches `sequence/entry.rs`, `sequence/orderless/optional.rs` and
`kernel_data/rewrite_data/replace/list/matching.rs`, none of which were
in that brief's Scope. The fault was the brief's, not the execution's.

**This brief removes the threading entirely** — see Implementation
guidance. Do not reintroduce a `core.rs`-rooted parameter chain; the
whole point of the redesign is that no signature crosses a file
boundary.

Part A of brief-276 (the fossil respec + contract doc) is already
committed as `92520ef`. This brief only implements, then updates the
two pinned tests and the doc to the ACHIEVED behavior.

## Why this brief exists (read before touching code)

`Flat` heads match a grouped selection of values by folding them back
under the head: `flat_match_value` (`sequence.rs:420`) wraps a
multi-element selection as `g[v1, v2, …]` but returns a single element
BARE. So for a one-element grouping, rum today offers the matcher only
the bare `1`, never `g[1]`. For a `Flat` head WITHOUT `OneIdentity`,
`g[1]` and `1` are genuinely different expressions, so the bare-only
candidate set is incomplete — a pattern that can only match `g[1]`
fails where it should succeed.

Confirmed grounding from brief-276: **no rum built-in head is affected.**
Every `Flat` built-in rum installs also carries `OneIdentity`, and
`OneIdentity` heads keep the bare-only path by construction (for them
`g[1]` and `1` are the same thing, and brief-275 already handles the
promotion direction). Only a user-defined `Flat`-without-`OneIdentity`
head reaches the new code. That is why this is a small correctness
completion, not a hot-path change — and why the microbench is expected
to be FLAT.

## Specification (the target contract)

For a `Flat` pattern head WITHOUT `OneIdentity`: when a plain
`Blank`-style pattern argument (`MatcherArg::Single` / the
`MatcherNode` single-choice loop) is matched against a grouped
selection of exactly ONE value, rum tries TWO candidate values for
that slot, in this order:

1. the BARE element (today's behavior — unchanged, and tried FIRST, so
   every existing pin survives as the winning candidate);
2. the element re-wrapped in the flat head,
   `Expr::call(flat_head, vec![bare_element])` (e.g. `g[1]`), as an
   ADDITIONAL candidate tried immediately after the bare one and
   before any longer consume-length grouping.

Applies symmetrically to the plain (non-orderless) `Flat` single-choice
loop and to the `Orderless`+`Flat` candidate generation.

`MatchQ` / `/.` (first-match): bare-first ordering means NO currently
passing test changes its result — the wrapped form is reached only when
bare fails to complete a full match INCLUDING the rest of the pattern.

`ReplaceList` (collect-all): bare-success and wrapped-success rows both
appear, wrapped immediately after bare, before the next length is tried.

## Implementation guidance (follow this shape)

The redesign rests on one observation: **`flat_head` is already in
scope at every point where a singleton candidate is generated.** So
`one_identity` never needs to travel — derive it where it is used.

Helper to use at both sites (`flat_head` is always a literal
system/user symbol `Expr` at these call sites, per brief-275's
grounding):

```rust
matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity))
```

### 1. `orderless/candidates.rs` — the generation point

Give `orderless_candidate_choices` ONE new parameter,
`wrap_singleton: Option<&Expr>` (the flat head to wrap with; `None`
means "bare only"). Keep the existing `allow_flat_single: bool` — a
`OneIdentity` flat head still wants `allow_flat_single = true` with
`wrap_singleton = None`, so the two flags are not redundant.

In the `MatcherArg::Single(_) if allow_flat_single` arm: after computing
today's length-`1..=max_len` partitions, if `wrap_singleton` is
`Some(head)`, then for every choice whose `selected_values` has length
exactly 1, insert an ADDITIONAL `OrderlessCandidateChoice` immediately
after it — identical `rest_values`, but
`selected_values = vec![Expr::call(head.clone(), vec![original])]`.
When `wrap_singleton` is `None`, the function must return a `Vec`
byte-identical to today's, with no extra allocation or pass.

No change is needed in `selected.rs`:
`match_compiled_orderless_flat_selected_values_owned`'s `[value]` arm
already does a plain `match_compiled_owned(pattern, value.clone(), …)`
with no awareness of bare-vs-wrapped, so a pre-wrapped `Expr` flows
through unmodified. **The entire orderless half is a candidate-
GENERATION change; the matching logic is untouched.**

### 2. `orderless/entry.rs` — compute locally, pass at two lines

Exactly two call sites pass `allow_flat_single = true` — line 51 in
`match_compiled_orderless_flat_arg_choices_owned` and line 112 in
`collect_compiled_orderless_flat_arg_choice_matches_owned`. Both
functions already take `flat_head: Expr`. Compute `one_identity`
locally in each and pass `(!one_identity).then_some(&flat_head)`.

The other two call sites (lines 21, 80 — the non-flat arms) pass
`None`.

Verify for yourself with `grep` that these are ALL four call sites
before editing, and confirm that both flat-Optional entry paths
(`optional.rs:54` present-arm and `optional.rs:250` →
`sequence/entry.rs:85` omission-arm) route through
`match_compiled_orderless_flat_arg_choices_owned` — i.e. that they
inherit the new candidates without being edited. If either does NOT,
STOP and report rather than widening scope.

### 3. `sequence.rs` — the plain `Flat` loop

`match_compiled_flat_single_choices_owned` (line 125) and
`collect_compiled_flat_single_choice_matches_owned` (line 309) are
private `fn`s whose every call site is inside `sequence.rs` itself
(lines 112/144/168 and 295/332/353 respectively — grep to confirm
before editing). So here, thread a `one_identity: bool` parameter
through them: all six call sites are in-file, and the two functions
RECURSE, so a locally recomputed attribute lookup would be paid on
every consume step. Compute it ONCE at the two outer call sites
(lines 112 and 295) and pass it down through the recursive calls
unchanged.

At `consume == 1`, when `!one_identity`: after the existing bare-value
attempt, ALSO try the wrapped value
`Expr::call(flat_head.clone(), vec![selected_values[0].clone()])`
through the identical continuation shape —
- match fn: only if the bare attempt INCLUDING its rest-match did not
  fully succeed (i.e. slot into the two existing "fall through to
  `consume + 1`" branches at lines 143-155 and 167-179, trying wrapped
  BEFORE advancing to `consume + 1`);
- collect fn: unconditionally, appending the wrapped results after the
  bare results, still before recursing to `consume + 1`.

Keep this a local addition at the `consume == 1` branch point. Do NOT
restructure the `consume` loop's general shape, and do NOT touch
`flat_match_value` — it is correct, and `selected.rs` also calls it.

## Ground Truth (measured on `92520ef`, BEFORE this change)

These are today's outputs, i.e. the values the two tests currently pin.
They are the BASELINE, not the target:

```text
g[1,2] /. g[a_, b_:0] :> {a, b}                        List[2, 1]
ReplaceList[g[1,2], g[a_, b_:0] :> {a, b}]             List[List[2, 1], List[1, 2], List[g[1, 2], 0]]
ReplaceList[g[1],   g[a_, b_:0] :> {a, b}]             List[List[1, 0]]
ReplaceList[g[1,2], g[a_, b___, c_:0] :> {a, {b}, c}]  List[List[2, List[], 1], List[1, List[], 2], List[1, List[2], 0]]
```

Expected DIRECTION of change (a check on your work, NOT a value to
pin): `ReplaceList` rows should only ever be ADDED, never removed or
reordered relative to each other — e.g. `ReplaceList[g[1], …]` should
gain a `List[g[1], 0]` row after the existing `List[1, 0]`. Every
first-match (`MatchQ`, `/.`) result above must be UNCHANGED. If any
existing row disappears, changes value, or moves relative to another
existing row, that is a bug in the change — not a new truth to pin.

**Do not hand-derive the new expected values.** Read them from actual
release-CLI output and pin THAT.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs`;
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/entry.rs`;
  - `crates/rum-runtime/src/runtime/pattern_matching/sequence.rs`;
  - `crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`
    (update the two respecced tests to achieved output);
  - `docs/dev/matcher-flat-optional-contract.md` (rewrite the `## Status`
    section to say the contract IS implemented as of this brief, and
    drop the "specification, not current behavior" qualifier from the
    next heading).
- Files/areas you MUST NOT touch: `pattern_matching/core.rs` (it was
  brief-275's subject and this design deliberately does not need it),
  `sequence/entry.rs`, `sequence/orderless/optional.rs`,
  `sequence/orderless/selected.rs`, `kernel_data/**`, `TODO.md`,
  `PROGRESS.md`, `AGENTS.md`, any other `docs/**`, anything under
  `docs/dev/codex/` except this file's own report, `.git`,
  `Cargo.lock`, any stdlib crate, `rum-host`.
- **If the implementation appears to require touching a MUST-NOT file,
  STOP and report that** — exactly as brief-276 did. That was the right
  call then and it is the right call now. Do not widen scope silently.
- Do NOT add unconditional cost to the default evaluation path. No
  `OneIdentity` head (`Plus`, `Times`, `Union`, `Intersection`, or any
  user `OneIdentity` head) may see ANY new work: the `None` /
  `one_identity == true` path must short-circuit before any allocation.

## Stop-loss

Run the whole of `pattern_matching_orderless_optional.rs` plus every
other `pattern_matching_*` test file in
`crates/rum-runtime/src/runtime/tests/` after each incremental step. If
- the same step fails twice for the same reason, OR
- any previously-passing test outside the two respecced ones changes
  its output, OR
- a first-match (`MatchQ` / `/.`) result changes anywhere, OR
- the implementation requires a MUST-NOT file,

then STOP: revert every source change (`git checkout --` the three
source files), leave the tests and doc at their committed `92520ef`
state, and report. A clean stop-loss is an acceptable outcome; a
silently widened scope is not.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-runtime pattern_matching -- --include-ignored
```

Baseline is `1653 passed / 0 failed` on `92520ef`; state your totals.
(Known caveat: loopback-bind ZeroMQ tests may fail under the sandbox;
if so, run and paste the two supplemental splits — `--workspace
--exclude rum-jupyter`, then `-p rum-jupyter --lib -- --skip
zeromq::tests::native` — and say so.)

Microbench A/B: build base BEFORE any source edit (copy to
`/tmp/brief277_base_rum-cli`), build candidate after; one
`scripts/core-language-microbench.sh` run per binary; paste
`matchq-*` / `downvalue-pattern-*` / `replaceall-simple` rows plus
same-tier neighbors, and state the direction mix. Given the confirmed
zero built-in exposure, FLAT is the expected and required outcome —
any movement on those labels needs an explanation before you call it
noise.

Then paste verbatim output of these probes on the CANDIDATE release
build (stdin from `/dev/null`):

```
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1,2], g[a_, b_:0]]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; g[1,2] /. g[a_, b_:0] :> {a, b}'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b_:0] :> {a, b}]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; MatchQ[g[1], g[a_, b_:0]]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; g[1] /. g[a_, b_:0] :> {a, b}'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1], g[a_, b_:0] :> {a, b}]'
target/release/rum-cli --eval 'SetAttributes[g, {Flat, Orderless}]; ReplaceList[g[1,2], g[a_, b___, c_:0] :> {a, {b}, c}]'
target/release/rum-cli --eval 'SetAttributes[k, Flat]; ReplaceList[k[1], k[a_]]'
target/release/rum-cli --eval '1+1'
target/release/rum-cli --eval 'MatchQ[5, x_ + y_.]'
target/release/rum-cli --eval 'Union[{1,2},{2,3}]'
```

The last three are regressions guarding the `OneIdentity` short-circuit
and brief-275's work; the `k` probe exercises the plain (non-orderless)
`Flat` half, which the `g` probes do not reach.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.
- No new `unsafe`.

## Report format (your final message)

1. What you changed, file by file.
2. Verbatim verification output (tests, microbench rows, probes).
3. Finding / verdict — the achieved contract, with the ACHIEVED
   `ReplaceList` rows quoted, and an explicit statement that no
   first-match result changed.
4. Stop-loss / open questions — if you stopped, the exact reason and
   the file that forced it.
