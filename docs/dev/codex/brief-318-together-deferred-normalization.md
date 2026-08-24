# brief-318 — stop renormalizing Together on every addend

## Goal

`Together` normalizes the accumulated rational function after **every** addend,
so summing `n` fractions costs roughly O(n^2.5). Reduce that, without changing
any output and without making the shapes that are fast today any slower.

## Measured today (release CLI, verified by the director — do not re-derive)

`Together[Sum[1/(x + i), {i, 1, n}]]` — pairwise-coprime denominators:

| n | time |
|---|---|
| 10 | 0.0165 s |
| 20 | 0.0845 s |
| 40 | **0.5341 s** |

5.1–6.3× per doubling, i.e. about O(n^2.4–2.7).

## The mechanism

`crates/rum-stdlib/src/polynomials/rational_function.rs`:

- The `Plus` arm (`:26-33`) folds addends one at a time: `result = result.add(…)`.
- `add` (`:65-77`) cross-multiplies and then calls `.normalized()` — which runs
  `cancel_common_factors`, a full polynomial factorization pass — on every step.

So the factorization cost is paid `n` times, on operands that grow as the fold
proceeds.

## The trap: deferring normalization is not free

The obvious fix — accumulate raw numerator/denominator products and normalize
once at the end — **makes a currently-fast shape slow**. Measured:

| input | current time | current result |
|---|---|---|
| `Together[Sum[1/x, {i, 1, 20}]]` | 0.00028 s | `20 * x^-1` |
| `Together[Sum[1/x, {i, 1, 40}]]` | 0.00038 s | `40 * x^-1` |

Identical denominators stay tiny **because** each step normalizes: the
accumulator never leaves the form `k/x`. Defer it, and the same input builds a
degree-40 numerator over `x^40` before cancelling it all back down.

So this is a genuine tradeoff, not an oversight, and a fix that only measures
the coprime case is not a fix. Whatever you build must be measured on **both**
shapes, and neither may regress.

Possible directions, in no particular order — pick with evidence, do not assume:

- Normalize lazily but bound the growth: e.g. keep a cheap denominator-identity
  or gcd check so identical/sharing denominators still collapse immediately,
  while coprime ones accumulate.
- Replace the per-step full factorization with something cheaper that preserves
  the same final result.
- Fold in a different order (e.g. pairwise/binary rather than linear), which
  changes the operand sizes the factorization sees.

## Output that must not change — measured, not assumed

| input | result |
|---|---|
| `Together[1/x + 1/y]` | `x^-1 * y^-1 * (x + y)` |
| `Together[1/x + 1/x]` | `2 * x^-1` |
| `Together[1/(x*y) + 1/(x*z)]` | `x^-1 * y^-1 * z^-1 * (y + z)` |
| `Together[x/(x^2) + 1/x]` | `2 * x^-1` |

The third is the one to watch: the shared `x` is cancelled, so the denominator
is `x*y*z` and not `x^2*y*z`. Any deferred scheme must still reach that form,
not merely an equivalent unreduced one — `Together`'s contract is the reduced
form, so an algebraically equal but larger result is a defect.

Capture goldens from the **pre-change build** and, as in brief-310 and
brief-312, run them green against the unmodified code first so they cannot be
fitted to the new behaviour afterwards. Cover at minimum: coprime denominators,
identical denominators, shared-factor denominators, a reducible sum, nested
fractions, a sum whose result is a polynomial (denominator cancels entirely),
and rational coefficients.

## Acceptable outcomes

1. **Both shapes improve, or one improves and the other is unchanged.** Best.
2. **A measured argument that the tradeoff cannot be won**, with the numbers
   that show it. That is a real result — record it and stop.

Do not accept "coprime got faster, identical got slower" as a win.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/polynomials/rational_function.rs`
- the `rum-stdlib` test module for polynomial/rational-function behaviour

Out of scope: everything else — including `Polynomial` itself, `Factor`, and the
rest of `polynomials/`. Full polynomial factorization is deferred by standing
user decision; do not grow the linear-factor code to work around this.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The change, with both shapes measured before and after.
- The golden corpus, stated by size, run green against the old code first.
- If you reach outcome 2, the measurements that establish it.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (10, 20, 40):
    open("/tmp/b318-coprime-%d.rum" % n, "w").write(
        'Print["coprime n=%d: ", AbsoluteTiming[Head[Together[Sum[1/(x + i), {i, 1, %d}]]]]];\n' % (n, n))
for n in (20, 40):
    open("/tmp/b318-same-%d.rum" % n, "w").write(
        'Print["same n=%d: ", AbsoluteTiming[InputForm[Together[Sum[1/x, {i, 1, %d}]]]]];\n' % (n, n))
PY
for n in 10 20 40; do ./target/release/rum-cli /tmp/b318-coprime-$n.rum < /dev/null; done
for n in 20 40; do ./target/release/rum-cli /tmp/b318-same-$n.rum < /dev/null; done
```

The coprime series must grow more slowly than before; the identical-denominator
series must not get slower and must still print `20 * x^-1` / `40 * x^-1`.

No microbench A/B: `Together` is a symbolic builtin, not the evaluation hot
path. Say so if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses. Goldens captured from
  rum itself are fine.
- **No new `unsafe`.**
- **Leave no `TODO` or `FIXME` markers in code.** This repository has none, by
  policy — future work belongs in `TODO.md`, which is out of scope here, so
  report it in your final message instead.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If a golden differs, STOP and report both strings
  rather than adjusting the golden.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges.
2. Both shapes' before/after numbers, and why the identical-denominator case
   did not regress.
3. Verification — each command with verbatim output: test counts, corpus size,
   the golden equality, and the two scaling series.
4. Stop-loss / open questions, including anything you would have written as a
   TODO.
