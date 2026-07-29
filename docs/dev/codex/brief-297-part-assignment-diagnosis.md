# Brief 297 — DESIGN-ONLY: find out why `lst[[i]] = v` is O(m), then plan the fix

**Design-only. `Scope: NONE` for source. You produce a diagnosis, a plan, and
brief drafts; you write no Rust.**

## Goal

After briefs 292–296, `AppendTo` (flat and through a Part spine) and
Association key assignment are all amortized O(1) per operation. But plain
positional assignment — `lst[[i]] = v` — is still linear in the **container
length**, and nobody has established why.

First **find the actual cause**, then plan the fix if one is warranted. A
well-argued "this should not be changed" is a valid and valuable outcome; do
not assume the conclusion is a patch.

## What is already measured (verify all of it)

**M1 — cost scales with container length, not just with the number of
operations.** Fixed at 100 assignments, varying only the list length m:

| m | `L[[1]] = i` x100 | relative | `AppendTo[A, i]` x100 (already optimized) |
|---|---|---|---|
| 100 | 0.609 ms | 1.00x | 0.143 ms |
| 200 | 0.996 ms | 1.64x | — |
| 400 | 1.633 ms | 2.68x | — |
| 800 | 3.071 ms | 5.04x | **0.169 ms (+18%)** |

Both operate on the same `Arc<Vec<Expr>>`. The optimized `AppendTo` is
essentially flat in m; positional assignment is linear in m. **This is why
Claude concluded the cost is an implementation gap rather than an inherent
cost of the immutable representation — check that reasoning.**

**M2 — accumulation is quadratic.** `qL = Table[0,{n}]; Do[qL[[i]] = i, {i,1,n}]`:
n=200 → 1.667 ms, n=400 → 5.869 ms, ratio **3.52x**.

**M3 — counters.** Same probe at n=200, `RUM_PROFILE_REPR=1`:

```
into_vec_total=0 into_vec_clone=0 shared_args_alloc=427 shared_expr_alloc=425
```

Two facts to explain: `shared_args_alloc` ~2 per operation is consistent with
allocating one fresh `Arc<Vec<Expr>>` per assignment (the Atoms inside are
memcpy'd and are not counted), **but `into_vec_total=0` means
`SharedArgs::into_vec` is never called** — and `into_vec` is precisely the
entry point that carries the `Arc::try_unwrap` unique-ownership optimization.
So this path appears to copy without ever attempting exclusivity.

### Where Claude was WRONG — do not repeat this

Claude first assumed the path was
`kernel_data/structure_data/exact_paths.rs:173 replace_exact_path`, which does
`unwrap_shared_args` → mutate one slot → `SharedArgs::new`. **That assumption
is refuted by M3**: `unwrap_shared_args` calls `into_vec()`, whose *both*
branches call `record_into_vec`, so a counter of 0 proves this path is not
taken. `replace_exact_path` has only three call sites
(`structure_data/replace/replace_at.rs:121`,
`definitions/definition_updates.rs:451`, and a `kernel_api` re-export used by
`rum-stdlib`), none of which is plain `Set[Part[sym, i], v]`.

**So the first question is simply: what code actually runs?** Claude did not
find it. Do not guess — trace it.

## Deliverable

A plan document at `docs/dev/codex/plan-297-part-assignment.md` containing:

1. **Verification of M1–M3**, with your own numbers. Say plainly where Claude
   was wrong.
2. **The actual execution path** for `Set[Part[sym, i], v]`, from the
   evaluator entry down to whatever copies the argument vector, with
   `path:line` at every hop. Explain the `into_vec_total=0` observation: what
   copies the Vec, if not `into_vec`?
3. **Why the Step 3/4 mechanism does not already cover this.** Brief-295
   extended the fast path through a Part *spine* for `AppendTo[lst[[i]], x]`.
   Positional *assignment* to the same shape is not covered. Is that a
   deliberate boundary, an oversight, or a different path entirely? This is
   the single most useful thing you can determine, because if the two are
   close, the fix may be small.
4. **Is it actually reachable?** Enumerate the common WL/rum idioms that hit
   this path: array fill (`Do[a[[i]] = f[i], ...]`), element swap, in-place
   matrix update, and anything in `rum-stdlib` or the test suite that does it
   internally. **If real code rarely takes this path, say so** — that changes
   the decision, and "rarely reached, leave it alone" is a legitimate finding.
5. **Observability boundary.** For `AppendTo` the rule was: only a plain
   `List` head with no interfering definitions is safe, because appending onto
   a non-`List` head evaluates that head's definitions. What is the equivalent
   boundary for positional assignment? Does `Set` on a Part target ever run
   user code before or after the mutation?
6. **The plan**, if warranted: steps, files, invariants, gates, expected win —
   same shape as plan-291 §5. Reuse the existing primitives
   (`SharedArgs::make_mut`, the mutation-proof facility, the predicate
   framework in `definitions/incremental_append.rs`) rather than inventing
   new ones, or explain why they do not fit.
7. **Drafts of the follow-up implementation briefs**, in `TEMPLATE.md` shape,
   with complete MAY-modify lists including mechanical ripple.
8. **Risks and open questions.**

## Scope

- File you MAY create: **`docs/dev/codex/plan-297-part-assignment.md`** only.
- Files you MUST NOT modify: **every `.rs` file**, plus all other docs,
  `scripts/**`, `Cargo.*`, `.git`.
- You MAY run read-only commands and write scratch probes under `/tmp`.

## Verification

Reproduce M1–M3 yourself. Timing notes for this repo:

- Use `AbsoluteTiming` (monotonic); the suite's `AbsoluteTime[]` differencing
  was removed in `3e42c09`.
- This suite has a per-run whole-run offset of a few percent that moves ALL
  labels together, so **a lopsided direction split is not evidence**. Judge by
  within-run quotients, or normalize against specific untouched labels —
  **never against the suite median**, which drifts cumulatively in an
  optimization lane (see `cc86cdd` and `a904935`).
- Counters are immune to load and are the preferred evidence. Prefer them.
- Check that any timing scales with the parameter you are varying; a number
  that does not scale means the probe is measuring something else.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **Write no Rust.** Code excerpts inside the plan document are encouraged.
- Do NOT modify any file outside the single plan document.
- Touch nothing outside this repo except `/tmp`. No network.
- **No new `unsafe`** in anything you propose.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Any proposal must preserve value semantics: after `x = {1,2,3}; y = x;
  y[[1]] = 9`, `x` is unchanged.
- Do not propose changing the `Expr` representation or the builtin `&[Expr]`
  ABI. That was examined in plan-291 finding D and is out of scope here.

## Report format (your final message)

1. **The execution path**, in 3–5 sentences, with `path:line` — this is the
   headline.
2. Verification verdicts for M1–M3, and where Claude was wrong.
3. Whether the Step 3/4 mechanism nearly covers this, or not at all.
4. Reachability: which real idioms hit this path.
5. Your recommendation: fix it, or leave it — with the reason.
6. Step list if you recommend fixing.
7. Confirm `git status` shows exactly one new file.
