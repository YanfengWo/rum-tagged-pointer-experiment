# Brief 192 — PowerMod negative-exponent modular inverse (unit-5 fix 4c)

Implementation brief. Executes part of unit-5 triage "cheap flexibility"
(tracker `docs/dev/usability-pass.md`, unit-5 section; evidence E3 in
`docs/dev/usability-pass/math-findings.md`). `PowerMod[a, e, m]`
currently rejects a NEGATIVE exponent (returns inert). WL defines
`PowerMod[a, -1, m]` = the modular inverse of `a` mod `m`, and
`PowerMod[a, -k, m]` = `(a^-1)^k mod m`. Add it via the extended
Euclidean algorithm — a standard, bounded, folk-cheap computation (the
natural completion of the modular exponentiation `PowerMod` already
does). No CAS, no new dependency.

All in `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs`
(`eval_power_mod` at `integer_ops.rs:90`; helpers below it).

## WL semantics (the acceptance spec)

```
PowerMod[2, -1, 5]   -> 3        (2*3 = 6 ≡ 1 mod 5)
PowerMod[3, -1, 7]   -> 5        (3*5 = 15 ≡ 1 mod 7)
PowerMod[2, -3, 5]   -> 2        ((2^-1)^3 = 3^3 = 27 ≡ 2 mod 5)
PowerMod[10, -1, 1]  -> 0        (everything ≡ 0 mod 1)
PowerMod[2, -1, 4]   -> PowerMod::ninv message + inert   (gcd(2,4)=2 ≠ 1)
PowerMod[2, 3, 5]    -> 3        (positive path UNCHANGED)
PowerMod[2, 3, 0]    -> inert    (zero-modulus singularity UNCHANGED)
PowerMod[2, -1, 0]   -> inert    (no inverse mod 0)
```

## Change — extend `eval_power_mod`

Today, after parsing `base`/`exponent`/`modulus` as exact integers,
`eval_power_mod` does `if exponent < 0 { return inert }`
(`integer_ops.rs:103`). Replace that rejection with negative-exponent
handling; the non-negative path and all inert fallbacks stay as they are:

- If `exponent >= 0`: UNCHANGED (existing `power_mod_integer` path).
- If `exponent < 0`:
  - if `modulus == 0` → return the inert call (no inverse mod 0; keep the
    singularity inert per the user decision).
  - compute `inv = mod_inverse(&base, &modulus)` (extended Euclid, below).
    - `None` (gcd(base, modulus) ≠ 1, i.e. not invertible) → emit
      `self.invalid_call_with_message("PowerMod", args, "ninv",
      format!("{base} is not invertible modulo {modulus}."))` (message +
      inert call).
    - `Some(inv)` → let `k = -exponent` (a positive `BigInt`); return
      `power_mod_integer(&inv, &k, &modulus).map(Expr::integer)
      .unwrap_or_else(|| inert)` — REUSE the existing positive-exponent
      helper to raise the inverse to `k`.

Add a hand-rolled extended-Euclid helper next to `bigint_gcd`
(`integer_ops.rs:125`), no new dependency:

```
// inverse of `base` in Z/|modulus|Z, or None if gcd(base, |modulus|) != 1
fn mod_inverse(base: &BigInt, modulus: &BigInt) -> Option<BigInt> {
    let m = modulus.abs();            // ring modulus (m != 0 by caller)
    if m == 1 { return Some(0); }     // everything ≡ 0 mod 1
    let a = base.rem_euclid-style reduce into [0, m);   // reuse mod_integer(base, &m)
    // iterative extended Euclid on (a, m): find x with a*x ≡ 1 (mod m)
    // track (old_r, r) and (old_s, s); if final gcd != 1 -> None
    // else return old_s reduced into [0, m)
}
```

Mirror the iterative style of the existing `bigint_gcd`. Use only
`BigInt` `+ - * / %` and comparisons (all already used in this file);
introduce NO new crate. Reduce the base with the existing
`mod_integer(base, &m)` so the sign convention matches the rest of the
file, and normalize the returned inverse into `[0, m)`.

## Tests

Add tests in `crates/rum-runtime/src/runtime/tests/arithmetic.rs` (where
the PowerMod/Mod tests live): every acceptance row above — the four
successful inverses, the `ninv` non-invertible case (assert the message
symbol/tag and the inert return), the `mod 1` case, the positive-path
unchanged case, and both zero-modulus inert cases. Keep existing PowerMod
tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{PowerMod[2,-1,5], PowerMod[3,-1,7], PowerMod[2,-3,5], PowerMod[10,-1,1], PowerMod[2,-1,4], PowerMod[2,3,5], PowerMod[2,3,0], PowerMod[2,-1,0]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
the probe returns `{3, 5, 2, 0, PowerMod[2,-1,4] (inert), 3,
PowerMod[2,3,0] (inert), PowerMod[2,-1,0] (inert)}` with a
`PowerMod::ninv` message printed for `PowerMod[2,-1,4]`.

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs`
  (`eval_power_mod` + the new `mod_inverse`/extended-Euclid helper) and
  `crates/rum-runtime/src/runtime/tests/arithmetic.rs` (new tests).
- MUST NOT touch: `Mod`/`Quotient`/`GCD`/`LCM` in the same file; the
  positive-exponent `power_mod_integer`; any other builtin;
  `Total`/`Commonest`/`Quantile` (other briefs); the guide,
  `docs/reference/builtins.md`, `TODO.md`, the tracker, findings files,
  `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, NO new dependency (hand
  roll extended Euclid with `BigInt` ops), no new `unsafe`.
- Compat: the non-negative-exponent path and all existing inert
  fallbacks (non-integer args, zero modulus) are byte-identical. Only a
  negative exponent with an invertible base newly computes a value; a
  non-invertible base emits `PowerMod::ninv` and stays inert.
- Folk scope: extended Euclid only — no factoring, no CRT, no other
  number theory.
- No A/B gate needed (not an eval/dispatch/matcher hot path).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — `eval_power_mod` negative branch + the helper, line
   ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
