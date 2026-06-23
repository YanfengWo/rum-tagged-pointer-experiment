# brief-031: DIAGNOSTIC — capture the `mkComb` type mismatch at the Real/Seq frontier

## Goal

This is a **diagnostic brief — produce evidence, NOT a code change.** The
correctness-mainline load is stuck at `stdlib/Real/Seq.wl` (index 36, 88%) with
`mkComb: argument type mismatch` (tag `term`, types `fun`/`real`). Deliver the
verbatim data that lets Claude pin the root cause: (1) the exact failing
`mkComb[f, x]` — `FullForm[f]`, `typeOf[f]`, `FullForm[x]`, `typeOf[x]`, the
`expected` domain — and the WL call stack at the throw; (2) the enclosing
`normLinRAux` real-multiply step that produced the mistyped term — the input
`t`, the sub-records `ar`/`br`, and `buildLinRAny[ar]`/`buildLinRAny[br]` with
`typeOf` of each; and (3) **best-effort** the same intermediates evaluated under
WolframKernel for the SAME captured `t`, to show the first rum-vs-WK divergence.
No rum Rust source changes. No edits to the canonical mma-hol clone.

## Context pointers

- Read `AGENTS.md` (the unmodified-`mma-hol` hard line) and the `PROGRESS.md`
  `2026-06-23` entry (the v5 checkpoint chain + the 5 bugs already fixed; the
  Real/Seq frontier is multi-layered — `Nothing` fixed the first layer, this
  `mkComb` mismatch is the next).
- The defect surfaces here (read it; do not edit the canonical copy):
  `auto/RealArith.wl` in `/Users/fft/Developments/mma-hol-acceptance`:
  - `normLinRAux[t_ /; realMulTermQ[t]]` (line ~1181): at line ~1195 it computes
    `comm = rMulComm[buildLinRAny[ar], buildLinRAny[br]]`. This is the suspected
    throw site — one of the two `buildLinRAny[...]` results is `fun`-typed where
    a `real` term is required.
  - `rMulComm[x_, y_] := specAll[HOL`Stdlib`Real`realMulCommThm, {x, y}]`
    (line ~522) — `specAll` instantiates `∀x y. x*y = y*x`; the instantiation
    goes through `mkComb`, which is where the `fun`/`real` mismatch is raised.
  - `buildLinRAny[rLin[c_, vs_, env_]]` (line ~759) → `summandTerm` (~736) /
    `scalarTerm` (~730) / `rAdd` (~98, `rAdd[a,b] := mkComb[mkComb[realAddConst[],a],b]`).
  - `realLinNormConv` (~1207) and `proveRealConclUnderFacts` (~1562) are the
    callers above `normLinRAux`.
- `mkComb` and `typeOf` live in `HOL`Terms``
  (`/Users/fft/Developments/mma-hol-acceptance/Terms.wl:44`):
  ```
  mkComb[f_, x_] := Module[{ft, xt, dom, rng},
    ft = typeOf[f]; xt = typeOf[x]; {dom, rng} = destFunType[ft];
    If[dom =!= xt, HOL`Error`holError["term", "mkComb: argument type mismatch",
      <|"fnType" -> ft, "argType" -> xt, "expected" -> dom|>]];
    comb[f, x]];
  ```
  A `fun`-typed argument = a `comb`/`const` chain that is missing one
  application (a curried constant not fully applied). That is almost certainly
  produced upstream by a rum eval divergence (WK loads mma-hol clean).
- rum is a Rust reimplementation of a Wolfram-kernel slice; this diagnostic
  runs the WL layer, it does not change rum.

## Reproduction (fast iteration base — seconds per run)

The authoritative checkpoints are the SIBLING dir
`/Users/fft/Developments/rum-snapshot-checkpoints-v5/` (NOT inside the repo).
Use `cp34` (Real/MinMax) and **re-extend RealArith with the current binary** —
`cp35` baked the OLD `Private`Nothing` into a stored DownValue and is tainted
(BLAST-RADIUS lesson, PROGRESS `2026-06-23`).

```
cd /Users/fft/Developments/rum
cargo build --release -p rum-cli           # ensure current binary
BIN="$PWD/target/release/rum-cli"
CKPT=/Users/fft/Developments/rum-snapshot-checkpoints-v5/cp34-stdlib_Real_MinMax_wl.mx

SCRATCH=$(mktemp -d)/mma-hol
rsync -a --exclude='.git/' --exclude='*.mx' \
  /Users/fft/Developments/mma-hol-acceptance/ "$SCRATCH"/
cp "$CKPT" "$SCRATCH"/bootstrap.mx

# re-extend RealArith with the CURRENT binary so Nothing/etc. resolve right
"$BIN" --eval '$ScriptCommandLine = {"tests/extend_snapshot.wls", "auto/RealArith.wl"}; Get["'"$SCRATCH"'/tests/extend_snapshot.wls"]; "ok"'

# reproduce the mkComb error (~4s)
"$BIN" --eval '$ScriptCommandLine = {"tests/extend_snapshot.wls", "stdlib/Real/Seq.wl"}; Get["'"$SCRATCH"'/tests/extend_snapshot.wls"]; "ok"'
```

Confirm the second command prints/throws `mkComb: argument type mismatch`
(tag `term`) before instrumenting. **Everything is in `$SCRATCH` — a throwaway
copy. Never touch `/Users/fft/Developments/mma-hol-acceptance`.**

## Diagnostic method

Instrument by **redefining the WL functions after restoring the snapshot,
before loading Seq** — do NOT cold-rebuild. Write a driver `*.wls` in `$SCRATCH`
(e.g. `$SCRATCH/tests/trace_mkcomb.wls`) that mirrors `extend_snapshot.wls`'s
preamble (`$HOLEncapsulationMode="Stable"`, `Get[bootstrap.mx]`, restore the
`HOL`*` public contexts onto `$ContextPath`), then:

1. **Trip-once mkComb dump.** Redefine `HOL`Terms`mkComb` so that when
   `dom =!= xt` it prints, exactly once (guard with a module-global flag), the
   verbatim `FullForm[f]`, `typeOf[f]`, `FullForm[x]`, `typeOf[x]`, `dom`, and a
   `Stack[]` (or `Throw` with a tagged marker) — then still raises the original
   `holError` so behavior is unchanged. (`Unprotect` it first if needed; it may
   be Protected after snapshot restore.)
2. **normLinRAux real-multiply capture.** Redefine the
   `normLinRAux[t_ /; realMulTermQ[t]]` branch (copy the canonical body from
   `auto/RealArith.wl:1181-1199`) with `Print`s, once, just before line ~1195,
   dumping: `FullForm[t]`, `typeOf[t]`; `ar`, `br` (the rLin records);
   `FullForm[buildLinRAny[ar]]`, `typeOf[buildLinRAny[ar]]`,
   `FullForm[buildLinRAny[br]]`, `typeOf[buildLinRAny[br]]`. Identify WHICH of
   the two is `fun`-typed.
3. Load `stdlib/Real/Seq.wl` under this driver with `$BIN`, capturing all
   output. The Print lines + the throw give the rum-side picture.

Then **(best-effort, secondary) the two-engine diff:** take the captured `t`
(its `FullForm`), and in a plain WolframKernel session that has the SAME
RealArith loaded — `wolframscript -file $SCRATCH/tests/build_snapshot.wls ...`
to the same point, OR simplest: load the libs up through RealArith under
`wolframscript` — reconstruct that exact `t` and evaluate `parseLinR[t]`,
`normLinRAux[a]`/`normLinRAux[b]`, `buildLinRAny[ar]`, `buildLinRAny[br]`, and
`typeOf` of each. Report the FIRST intermediate where WK differs from rum (e.g.
rum's `buildLinRAny[ar]` is `fun`-typed, WK's is `real`-typed). **Stop-loss:**
if reconstructing `t` under WK is awkward (huge term, serialization friction),
SKIP the WK diff and report only the full rum-side capture — that is still a
complete, useful deliverable; say you stopped there and why.

## Scope

- Files you MAY create/modify: ONLY files under the throwaway `$SCRATCH` copy
  (the driver `*.wls`; instrumentation redefinitions). These are scratch and
  not committed.
- Files/areas you MUST NOT touch:
  - the canonical `/Users/fft/Developments/mma-hol-acceptance` clone (read-only)
    or any other `mma-hol` clone — NEVER edit the acceptance standard;
  - ANY rum Rust source (`crates/**`) — this brief changes no rum code;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except this brief's own
    report section; `.git`, git config, `Cargo.lock`;
  - the `rum-snapshot-checkpoints-v5/` checkpoints (read-only — copy `cp34`,
    never overwrite it).
- No rum default-path cost question here (no Rust change).

## rum / Rust pitfalls (read twice)

1. **Re-extend RealArith from cp34, never start from cp35.** cp35 baked the old
   `Private`Nothing`; using it reintroduces a fixed bug and wastes a cycle.
2. **Instrument by redefinition after restore — do NOT cold-rebuild** (~7 min)
   and do NOT edit canonical `auto/RealArith.wl`. Copy the function body into
   the driver and redefine in the live session.
3. **Print, don't mutate semantics.** The instrumented `mkComb` /
   `normLinRAux` must still raise/return exactly what the originals do; only add
   `Print`. Guard each dump with a once-flag so the log isn't flooded
   (`mkComb` runs constantly).
4. **`FullForm` everything you report.** A pretty-printed HOL term hides the
   `comb`/`const`/`tyApp` structure that reveals the missing application.
   Always pair a term dump with its `typeOf`.
5. **Scratch is throwaway.** Put it under `$(mktemp -d)`; do not leave artifacts
   in the repo or in the canonical clone.
6. **Stop-loss.** If the reproduction does not show `mkComb: argument type
   mismatch` after the cp34 + re-extend recipe, STOP and report what error you
   got instead — do not chase a different failure.

## Verification (MANDATORY — you run it; the evidence IS the deliverable)

No source change ⇒ no `cargo` gate. Run the reproduction + the instrumented
driver and paste, VERBATIM:

1. The second reproduction command's output showing `mkComb: argument type
   mismatch` (confirms the frontier).
2. The trip-once `mkComb` dump: `FullForm[f]`, `typeOf[f]`, `FullForm[x]`,
   `typeOf[x]`, `expected` dom, and the `Stack[]`/marker.
3. The `normLinRAux` real-multiply dump: `t`, `typeOf[t]`, `ar`, `br`,
   `buildLinRAny[ar]` + its `typeOf`, `buildLinRAny[br]` + its `typeOf`, and
   which one is `fun`-typed.
4. (best-effort) the WK comparison rows for the same `t`, or an explicit note
   that you stopped before the WK diff and why.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave any in-repo
  artifact (only this brief's report) in the working tree; Claude gates.
- NEVER write to/edit any `mma-hol` clone or the v5 checkpoints. All edits live
  in the `$(mktemp -d)` scratch.
- Do NOT modify rum Rust source or anything outside Scope. If you think you
  must, STOP and say so.
- Full machine access, touch NOTHING outside the named scratch + repo report,
  no network. Commands: the release `rum-cli`, `wolframscript`, `rsync`/`cp`/
  `mktemp`, `python3`.
- This is diagnostic only — produce evidence, change no behavior. You MUST paste
  real verbatim output; if a run did not reach the error or evidence, say so.

## Report format (final message)

1. Repro confirmation — the verbatim `mkComb: argument type mismatch` line.
2. The rum-side capture — items 2 and 3 above, verbatim, with the `FullForm` +
   `typeOf` of every term, and a one-line statement of which `buildLinRAny`
   result is `fun`-typed.
3. The WK two-engine diff (or the explicit stop-loss note).
4. Your reading of the mechanism (one short paragraph) — where the `fun`-typed
   term came from, if the dumps make it visible. (Claude decides the fix.)
5. Open questions / where you stopped.
