# brief-386 — comparison operators are not associative; introduce `Inequality`

`TODO.md` has carried "parentheses are transparent to comparison-chain
collection" since the second sweep, recorded as half-fixable. **That record is
wrong on both counts** — the defect is wider than it says, and the framing that
made half of it look unfixable was itself the mistake.

## The defect

Comparison operators are parsed as left-associative binaries and the chain is
collected during lowering. Parentheses produce the same tree as a chain, so the
collector cannot tell them apart and flattens both. Director's probes on the
current tree:

```
a == b == c      -> Equal[a, b, c]          correct
(a == b) == c    -> Equal[a, b, c]          wrong
a == (b == c)    -> Equal[a, b, c]          wrong
(a < b) < c      -> Less[a, b, c]           wrong
(1 == 1) == 1    -> True                    wrong
```

The whole comparison family is affected, not just `==`: `<`, `!=`, `===` all
flatten through parentheses the same way.

**The framing to discard.** `TODO.md` says the right-hand cases are fixable
while `(a == b) == c` and `a == b == c` "share one AST and cannot be told apart
after parsing". Both halves are wrong: `a == (b == c)` flattens too, and the
sharing is not a fact about comparison chains but a consequence of parsing them
as associative binaries in the first place. **These operators have no
associativity.** Collect the chain in the parser and a parenthesised
subexpression is one operand of the outer chain by construction — no marker, no
after-the-fact disambiguation.

## The second defect, in the same machinery

Mixed chains lower to `And`, which repeats the middle operand:

```
a < b == c   ->  And[Less[a, b], Equal[b, c]]        b appears twice
```

Measured consequences, director's probes, against WL as design input:

| Probe | WL | rum |
|---|---|---|
| `1 < f[] < 3`, `f` counts calls | 1 | 1 |
| `1 < g[] == 2`, mixed | **1** | **2** — evaluated twice |
| `1 < 0 < h[]` | 1 | 1 |
| `1 < 0 == k[]`, mixed | **1** | **0** — rum short-circuits |

## The decided shape (user ruling, 2026-08-30)

**Introduce `Inequality`, matching the wider WL family's name, argument layout
and semantics.** Design input only — described here in prose, never entering
the repository as a test oracle.

Measured semantics to implement:

1. **Alternating layout**: `Inequality[e1, op1, e2, op2, e3, ...]`, operators as
   symbols. `a < b == c` is `Inequality[a, Less, b, Equal, c]`; three segments
   extend the same way (`a < b <= c == d` →
   `Inequality[a, Less, b, LessEqual, c, Equal, d]`).
2. **A same-operator chain does not use it**: `a == b == c` stays
   `Equal[a, b, c]`, `a < b < c` stays `Less[a, b, c]`. `Inequality` is for
   mixed chains only.
3. **Attributes are `{Protected}` and nothing else.** No hold attribute. This
   is the mechanism behind the next two points, not an incidental fact.
4. **Every operand evaluates exactly once, and there is no short-circuit.**
   `1 < 0 < h[]` still evaluates `h[]` even though the first segment is already
   False. rum currently short-circuits mixed chains; that goes away, and it
   goes away *because* the operands are ordinary arguments of a
   non-holding head.
5. **Binary degenerates**: `Inequality[a, Less, b]` evaluates to `Less[a, b]`,
   so the head survives only with two or more segments.
6. **An undetermined chain is preserved**: `Inequality[1, Less, x, Less, 2]`
   with `x` unassigned stays as written.

### Why no short-circuit, since it would be easy

Adding `HoldAll` and driving evaluation by hand — what `And`/`Or`/`If` do —
would give a short-circuit cheaply. It is deliberately not done, and the
reason matters for anyone tempted to add it later.

Comparison chains routinely return **undetermined** results: probe P16 above
shows `Inequality[1, Less, x, Less, 2]` with `x` unassigned surviving as
written. Short-circuiting presupposes that "false" is a state you can settle
early — two-valued logic. Once the third state appears, the rule becomes
ambiguous: if `1 < x` cannot be decided, `x < 2` cannot be skipped either,
because the whole expression is retained as a symbolic object and every part
of it has to be in evaluated form. A half-evaluated chain would break later
matching and simplification. Evaluating everything and then deciding avoids
the question entirely, which is why "each operand exactly once" and "no
short-circuit" are the same mechanism rather than two policies.

`And` is different in kind: its operands are guards where the later one
*depends* on the earlier holding (`x != 0 && 1/x > 1`), so short-circuiting
there is semantics, not optimization. A comparison chain's operands are values
of equal standing.

Two further arguments, independent of the above. Sharing a name with the wider
family while diverging in evaluation semantics is worse than not having the
symbol at all — anyone carrying knowledge across would hit a difference that is
hard to see. And the saving is negligible: chains are two or three segments of
cheap operands, so short-circuiting only pays when an operand is expensive or
effectful, and an effectful comparison chain is itself suspect code. If a
short-circuiting variant is ever wanted, it belongs in an option or a
rum-native sibling, per the standing directive that default behavior stays
conformant and enhancements go elsewhere.

## Goal

Comparison chains are collected in the parser, so parentheses bound them
structurally; a same-operator chain lowers to its n-ary head; a mixed chain
lowers to `Inequality`.

Both defects are one repair. Once the parser produces a chain node, "what does
this node lower to" is exactly the second question, and doing them separately
leaves an awkward intermediate state.

## Required first step — inventory

Report before changing anything:

- Every operator that participates in chain collection today, and its lowering.
- Every test that pins a chain's `FullForm`, evaluation result, evaluation
  count, or short-circuit behavior — **including the lowering golden**, which
  `TODO.md` says records the current (wrong) parenthesis behavior with a
  comment. That golden is an anchor on the defect; updating it is expected, and
  its comment must be rewritten to state the new contract.
- The complete registration chain a new builtin needs. **Trace an existing
  sibling** — `Equal` or `Less` — through every place it is registered rather
  than recalling the list: kernel dispatch, stdlib shim and catalog if
  applicable, the attribute mirror that the vocabulary test enforces, and the
  generated `docs/reference/builtins.md` with its drift test. A missed
  registration site has cost this project a dispatch before.

## Scope

Files you MAY modify: the comparison-chain parsing and lowering paths in
`crates/rum-syntax/src/**`, the comparison builtins and their registration
sites in `crates/rum-runtime/src/**` and `crates/rum-stdlib/src/**`, the
generated builtin reference, and the test modules covering comparison chains
and the vocabulary.

**The MAY list is a floor, not a ceiling.** A purely mechanical ripple — an
exhaustive `match` arm forced by a new variant, a call site forced by a changed
signature, an attribute-mirror row forced by a new builtin, a re-export forced
by a moved item — you MAY edit unlisted; list each under "mechanical ripple".
Anything changing behavior, contract, or a test expectation in an unlisted
file: STOP and report. When unsure, it is the second kind.

Out of scope: the evaluator's general dispatch, `Expr`/`Symbol` representation,
public serde shapes, `Cargo.lock`, `.git`, `TODO.md`, `PROGRESS.md`,
`AGENTS.md`, other `docs/**`.

## Verification

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### Test matrix

- Parenthesis boundaries in both directions and for at least three operators:
  `(a==b)==c` → `Equal[Equal[a,b],c]`, `a==(b==c)` → `Equal[a,Equal[b,c]]`,
  `(a<b)<c` → `Less[Less[a,b],c]`. Assert `FullForm`, not just the value.
- The unparenthesised chains still flatten: `a==b==c` → `Equal[a,b,c]`.
- `(1==1)==1` evaluates to `True == 1` and stays unevaluated; `1==(1==1)` to
  `1 == True`; `1==1==1` to `True`.
- Mixed chains produce `Inequality` with the alternating layout, including a
  three-segment chain with three different operators.
- **Evaluation counts**: a mixed chain evaluates each operand exactly once —
  the `1 < g[] == 2` probe with a counter must read 1, not 2.
- **No short-circuit**: `1 < 0 == k[]` evaluates `k[]`, counter reads 1, and
  the result is False.
- Binary degeneration, and an undetermined chain preserved.
- `Attributes[Inequality]` is exactly `{Protected}`.
- The vocabulary and reference-drift tests cover the new builtin.

Every probe carries a liveness assertion — a counter that proves the operand
ran, not merely that a value came back.

### Deliberately not covered by a snapshot

Do not update a pinned `FullForm` or evaluation result to whatever the code now
prints. Where a pin moves, state the before and after and why the new value is
right. The lowering golden is the one anchor known in advance to move.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness, no third-party output committed. The WL
  semantics above are design input, stated in prose; they do not enter the
  repository as an oracle.
- No unconditional cost on the ordinary evaluation path. Chain collection
  happens in the parser; say what a non-comparison expression pays, which
  should be nothing.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`.
- **Stop-loss** — report rather than improvise if: chain collection in the
  parser cannot preserve an existing operator's precedence or arity; a pinned
  behavior moves that is not a comparison chain; or the registration chain
  requires touching a file whose behavior would change.

## Report format

1. The inventory — operators, pinned tests, and the registration chain traced
   from a sibling.
2. What changed — per file, line ranges, one-line why.
3. Mechanical ripple.
4. Verification — `fmt`, workspace counts, the evaluation-count probes with
   their counters, and every moved pin with before/after and justification.
5. Stop-loss / open questions.
