# brief-030: implement `Nothing` — the System symbol removed from `List`

## Goal

rum has no `Nothing`. In Wolfram, `Nothing` is a System symbol that is
automatically REMOVED from any `List` in which it appears:
`{a, Nothing, b}` is `{a, b}`, `{Nothing}` is `{}`. Today rum leaves it
(`{1, Nothing, 2}` -> `List[1, Nothing, 2]`) and a bare `Nothing` even resolves
to the WRONG context (`Context[Nothing]` is `"Global`"`, `Names["System`Nothing"]`
is empty). This is the current correctness-mainline blocker: `auto/RealArith.wl`'s
`negativePart` uses the standard WL idiom of mapping unwanted entries to `Nothing`
and letting them vanish from the resulting list/association; in rum the stray
`Nothing` survives, so a real linear form (`rLin[...]`) comes out malformed (an
`Association[{Nothing}]` and an unevaluated `rFilterZeros[Merge[...]]`),
`rLinIsZero[negRec]` is then non-Boolean, the `If[rLinIsZero[negRec], ...]` in
`realAtomNormConv` takes NEITHER branch, its Module local `leftTerm` is left
UNBOUND, and `realLinNormConv[leftTerm]` raises `realarith-norm: term is not a
real linear expression`, stalling the load at `stdlib/Real/Seq.wl`. When done,
`Nothing` is a System symbol and vanishes from lists, so the idiom works.

## Context pointers

- Read `AGENTS.md` and the PROGRESS `2026-06-23 ... v5 ... Real/Seq` entry.
  Root-caused by tracing `realAtomNormConv` on the cp35 checkpoint: the failing
  `negativePart[scaledT]` returned `rLin[0, Association[{...Nothing}], <|...|>]`
  — the literal `Nothing` (resolved to a non-System symbol) was not dropped.
- WL semantics to match:
  - `{a, Nothing, b}` -> `{a, b}`; `{Nothing, Nothing}` -> `{}`; nested:
    `{a, {b, Nothing}}` -> `{a, {b}}`.
  - List-SPECIFIC: a general head keeps it — `f[a, Nothing, b]` stays
    `f[a, Nothing, b]`. (Do NOT remove `Nothing` from arbitrary heads.)
  - `Nothing` as an Association VALUE is KEPT (`<|a -> Nothing|>` stays); only its
    presence as a LIST ELEMENT removes it — so `Association[{a -> 1, Nothing}]`
    becomes `Association[{a -> 1}]` because the `{...}` list drops the `Nothing`
    first.
  - `Context[Nothing]` is `"System`"`; `Names["System`Nothing"]` is `{Nothing}`.
- Key files:
  - System-symbol registration: `crates/rum-runtime/src/runtime/builtin_catalog.rs`
    (`core_builtin_specs`) and however bare names resolve to `System`` (so
    `Nothing` is known to be a System symbol like `Null`/`True`/`False`). Find
    where those value-like System symbols are registered and add `Nothing`.
  - List evaluation / argument preparation:
    `crates/rum-runtime/src/runtime/eval_call_prepare.rs` (`append_prepared_argument`
    / `splice_sequence_argument`, line ~65-76, runs the Sequence splice during
    call prep). Nothing-removal is the List-specific analogue — remove
    `System`Nothing` elements when (and only when) the call head is `List`. Pick
    the cleanest hook (e.g. a List-specific normalization after args are prepared,
    or filtering in List's evaluation), not an unconditional all-heads filter.

## Scope

- Files you MAY modify: the System-symbol registration site and the List
  evaluation/arg-prep site under `crates/rum-runtime/src/runtime/`, plus a test
  module under `crates/rum-runtime/src/runtime/tests/`.
- MUST NOT touch: any `mma-hol` clone; `crates/rum-core` `Expr`/`Symbol`/serde
  shape; the generic Sequence-splice behavior for non-List heads; `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` except your report; `.git`, git config,
  `Cargo.lock`.
- Do NOT add cost to the default eval path for non-List calls — the Nothing
  filter must be gated on the head being `List` (a cheap check), not run for
  every call.

## rum / Rust pitfalls (read twice)

1. **List-only.** `f[a, Nothing]` (general head) keeps `Nothing`. Only `List`
   drops it.
2. **Level-1 list elements.** Remove `Nothing` that is a direct element of a
   `List`; nested lists each drop their own (handled naturally as each List
   evaluates).
3. **Resolution.** A bare `Nothing` in source must resolve to `System`Nothing`
   (so mma-hol's unqualified `Nothing` is the magic one), not a fresh
   context-local symbol.
4. **Don't break empty/normal lists.** `{}` stays `{}`, `{a, b}` stays `{a, b}`.
5. **Minimal diff.** **Stop-loss:** if the registration or the List hook fights
   the architecture, implement the List-removal first (the functional unblock)
   and report the resolution part if separate.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval '{1, Nothing, 2}'                 # {1, 2}
$BIN --eval '{Nothing, a, Nothing, b}'        # {a, b}
$BIN --eval '{Nothing, Nothing}'              # {}
$BIN --eval '{a, {b, Nothing}}'               # {a, {b}}
$BIN --eval 'Association[{a -> 1, Nothing, b -> 2}]'   # <|a -> 1, b -> 2|>
$BIN --eval 'Context[Nothing]'                # System`
$BIN --eval 'Names["System`Nothing"]'         # {Nothing}
# control — general head keeps it:
$BIN --eval 'FullForm[f[a, Nothing, b]]'      # f[a, Nothing, b]
```

Paste verbatim. (The real unblock — restore the v5 `cp35-auto_RealArith_wl.mx`
checkpoint into a scratch `bootstrap.mx` and `extend_snapshot stdlib/Real/Seq.wl`,
expecting NO `realarith-norm` error — is Claude's follow-up; you do NOT run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`, `python3`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the registration + the List hook, files + line ranges.
2. Verification — each command, verbatim output.
3. The verdict — `Nothing` resolves to System and vanishes from lists; general
   heads keep it; tests green.
4. Stop-loss / open questions.
