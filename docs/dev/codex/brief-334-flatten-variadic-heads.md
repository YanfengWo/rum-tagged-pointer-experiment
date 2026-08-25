# brief-334 — flatten the variadic non-`Flat` heads (plan-330 step 4)

Fourth of five steps from `docs/dev/codex/plan-330-left-chain-lowering.md`.
Steps 1-3 landed in `70af8378`, `62839ffa` and `541ba6af`.

Four more tokens flatten into one n-ary call:

- `.` → `Dot`
- `<>` → `StringJoin`
- `|` → `Alternatives`
- `~~` → `StringExpression`

After this, seven operators remain nested by design: `**` `::` `/.` `//.` `:`
`?` `//`. They stay that way — their heads are binary by nature.

## How this differs from step 3

Step 3's four heads carry `Flat`, so nesting was invisible to the evaluator.
**None of these four do.** The flattening is therefore observable in held and
inert values, and that is accepted rather than hidden.

**Do not add `Flat` to any of them.** That would change direct calls throughout
evaluation and matching — far wider than syntax lowering, and not what this
plan authorises.

## Director's ruling on Dot

**Dot flattens.** `eval_dot` already accepts any nonempty arity and left-folds
over its operands, so n-ary is the head's native shape rather than something
invented here. The golden row `("a . b . c", "Dot[Dot[a, b], c]")` in
`lowering_golden.rs` is to be updated deliberately, not worked around.

## Measured today (director's probes — do not re-derive)

Equivalent in the valid domain:

```
StringJoin[StringJoin["a","b"],"c"]            → abc
StringJoin["a","b","c"]                        → abc
Dot[Dot[{1,2},{{1,0},{0,1}}],{1,1}]            → 3
Dot[{1,2},{{1,0},{0,1}},{1,1}]                 → 3
```

Equivalent in matching:

```
MatchQ[b, Alternatives[Alternatives[a,b],c]]   → True     flat: True
MatchQ[c, Alternatives[Alternatives[a,b],c]]   → True     flat: True
MatchQ[d, Alternatives[Alternatives[a,b],c]]   → False    flat: False
StringMatchQ["abc", StringExpression[StringExpression["a","b"],"c"]] → True   flat: True
StringMatchQ["abd", …]                         → False    flat: False
```

Observably different — the intended, accepted change:

```
FullForm[StringJoin[StringJoin[a,b],c]]        → StringJoin[StringJoin[a, b], c]
FullForm[StringJoin[a,b,c]]                    → StringJoin[a, b, c]
FullForm[Dot[Dot[a,b],c]]                      → Dot[Dot[a, b], c]
FullForm[Dot[a,b,c]]                           → Dot[a, b, c]

MatchQ[Hold[StringJoin[StringJoin[a,b],c]], Hold[Verbatim[StringJoin][_,_]]]   → True
MatchQ[Hold[StringJoin[a,b,c]],             Hold[Verbatim[StringJoin][_,_]]]   → False
MatchQ[Hold[StringJoin[a,b,c]],             Hold[Verbatim[StringJoin][_,_,_]]] → True
```

## What must not change

- **Valid-domain evaluated results.** String concatenation, `Dot` over real
  arrays, pattern matching through `Alternatives`, string matching through
  `StringExpression`. This is the hard gate, as in step 3.
- **Matching semantics.** The measured `MatchQ`/`StringMatchQ` rows above must
  read the same after the change.
- **Direct canonical-call opacity.** `StringJoin[a,b] <> c <> d` stays
  `StringJoin[StringJoin[a, b], c, d]`.
- The seven operators that remain nested.
- `docs/reference/builtins.md` — no catalog change, so it must regenerate
  byte-identical. If it does not, stop and report.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`
- `crates/rum-runtime/src/runtime/tests/lowering_golden.rs`
- `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`
- new test modules under `crates/rum-runtime/src/runtime/tests/`, plus the
  `mod.rs` line registering them
- `docs/lowering-canonical-forms.md`
- `docs/guide/syntax.md`

Out of scope: everything else — catalogs, attributes, the matcher, the string
pattern engine. Do not add unconditional cost to the default evaluation path.

## Deliverable

- The four heads flattened; the seven binary ones untouched.
- The Dot golden updated per the ruling.
- Goldens for each of the four, plus direct-call opacity.
- **Positive tests for the residual-shape change**: held symbolic `Dot` and
  `StringJoin`, and a `Verbatim` arity case. `docs/testing-strategy.md:39-55`
  treats held inputs and inert residual shape as contract axes, so these are
  contract, not incidental.
- Tests pinning `Alternatives` and `StringExpression` matching unchanged.
- The two docs updated to describe the n-ary forms for these four.
- **A complete list of shape changes** in the report.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
cargo run -q -p rum-stdlib --bin generate-builtin-reference && git diff --stat docs/reference/builtins.md
```

The reference must show no diff.

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise.

**Capture a behavior corpus before you change anything and diff it after.**
Unlike step 3 this corpus has two halves, and they are judged differently:

- **Valid-domain results and matching** — string concatenation over literals,
  `Dot` over arrays and matrices, `MatchQ` through `Alternatives` including
  nested alternatives, `StringMatchQ` through `StringExpression`. **Any
  difference here fails the brief.**
- **Held and inert shapes** — symbolic chains, `FullForm` of unsupported
  operands, `Verbatim` arity matches. **Differences here are expected**;
  enumerate them.

Then the deep probes, at 20 000 links for `.`, `<>`, `|` and `~~`:

```
Print[Head[Hold[ a <op> a <op> ... ]]];
```

`Hold` must print.

No microbench A/B: lowering runs once per parsed form. Say so if you believe
otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- **Do not add `Flat` or any other attribute to any head**, and do not change a
  catalog spec. If flattening appears to need one, that is a stop-loss: report
  it instead.
- **If you record current behavior in a golden and it looks wrong, say so and
  annotate the row.** A defect written into a golden becomes an expectation.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If flattening one of the four changes a valid-domain
  result or a matching outcome, stop and report that head. Three of four
  landing is a good outcome; four with a broken gate is not.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — the builder and which heads route through it, with line
   ranges.
2. The complete shape-change list.
3. The behavior corpus, split into the two halves above: the valid-domain half
   with its diff (must be empty), and the held/inert half enumerated.
4. Verification — commands with verbatim output, the builtin reference diff,
   and the four deep probes.
5. Stop-loss / open questions.
