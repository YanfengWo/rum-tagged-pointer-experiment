# Brief 061 — Structured (typed) HostValue

## Goal

`HostValue` is `{ text: String }` only — an embedder (Rust/Python/JS) gets the result as
a STRING and must re-parse it to do anything typed. When you are done, `HostValue` ALSO
carries a typed view of the result so an embedder can pattern-match an `i64` / rational /
symbol / string / list directly, with a clean fallback for anything else. Back-compat:
`HostValue.text` stays exactly as today (every existing reader keeps working); the typed
view is an ADDED field, not a replacement. Host layer only — zero rum-runtime/rum-core
change, no eval-hot-path cost (the conversion runs once when building the outcome).

This is the Tier-1 "structured value extraction" gap (`docs/host-embedding-and-hardening.md`).

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-1 ("Structured value
  extraction. `HostValue` is `{ text: String }` only … Embedders need typed access").
- The result `Expr` IS available to the host: `SessionEvalResponse.value: Expr`
  (`crates/rum-runtime/src/runtime/model.rs:614`), and `eval_success_from_runtime`
  (`crates/rum-host/src/model/outcome.rs:173`) currently does just
  `HostValue { text: response.value.to_string() }`. Build the typed view from
  `&response.value` there.
- rum is **rationals-only** (no machine floats — `[[rum-rationals-only-by-design]]`), so
  numbers are only `Integer` or `Rational`. Do NOT invent a float variant.
- Types (all reachable via `rum_runtime::*`, which rum-host already imports — confirm the
  exact import path and add what you need):
  - `Expr` = `Atom(Atom)` | `Call { head: SharedExpr, args: SharedArgs }` (`crates/rum-core/src/expr.rs:653`).
  - `Atom` = `Integer(IntValue)` | `Rational(Box<Rational>)` | `String(String)` | `Symbol(Symbol)` (:470).
  - `IntValue::as_i64() -> Option<i64>` (:286), `IntValue::to_bigint() -> BigInt` (:279).
  - `Rational::numerator() -> &BigInt` (:387), `Rational::denominator() -> &BigInt` (:391).
  - `Symbol`'s `Display`/`to_string()` gives the (context-qualified) name; `SharedArgs`
    iterates the call args (`.iter()` / `.as_ref()`).
  - For detecting a `List[...]` head: compare the head, when it is an `Atom(Symbol)`, to
    the `List` symbol — use the existing `system_expr("List")` / symbol-name helper rather
    than a raw string compare if one exists; a `System\`List` name match is the intent.

## Design (do this; deviate only with a reported reason)

1. Add a public enum (e.g. `HostTyped`), deriving `Clone, Debug, PartialEq, Eq`:
   ```
   pub enum HostTyped {
       Integer(i64),                                          // IntValue that fits i64
       BigInteger(String),                                    // decimal string otherwise
       Rational { numerator: String, denominator: String },   // decimal strings (BigInt-safe)
       Symbol(String),                                        // context-qualified name
       Str(String),                                           // a WL String
       List(Vec<HostTyped>),                                  // List[...] elements, recursively
       Other { head: String, text: String },                 // anything else: head name + its FullForm/text
   }
   ```
2. Extend `HostValue` to `{ text: String, typed: HostTyped }` (keep `text`; ADD `typed`).
   Update every `HostValue` construction (there is essentially one — `eval_success_from_runtime`;
   grep to be sure) to fill `typed` from the result `Expr`. Existing `.text` readers
   (e.g. `structured_outputs`) are unaffected.
3. The `Expr -> HostTyped` conversion (a private fn in outcome.rs), DEPTH-CAPPED:
   - `Atom::Integer(iv)` → `as_i64()` ? `Integer(i)` : `BigInteger(iv.to_bigint().to_string())`.
   - `Atom::Rational(r)` → `Rational { numerator: r.numerator().to_string(), denominator: r.denominator().to_string() }`.
   - `Atom::String(s)` → `Str(s.clone())`.
   - `Atom::Symbol(sym)` → `Symbol(sym.to_string())`.
   - `Call { head, args }` where head is the `List` symbol → `List(args.iter().map(convert_at_depth+1).collect())`.
   - any other `Call` (or a List that hit the depth cap) → `Other { head: <head: the symbol name if head is a symbol, else head.to_string()>, text: <this expr>.to_string() }`.
   - **DEPTH CAP**: thread a `depth: usize`; beyond a small constant (e.g. `MAX_TYPED_DEPTH = 64`)
     stop recursing and emit `Other { head, text }` for the whole subtree. This prevents a
     recursive-convert stack overflow on a pathologically deep result (same discipline as
     the parser/matcher depth guards — a deep `List[List[...]]` from `Get` of a crafted
     dump must not crash the host while extracting a value). Report the chosen cap.
4. Export `HostTyped` from `crates/rum-host/src/lib.rs`. If the Jupyter adapter reads
   `HostValue` it is unaffected (it reads `.text`); only touch rum-jupyter if it fails to
   compile.

## Scope

- MAY modify: `crates/rum-host/src/model/outcome.rs`, `crates/rum-host/src/model/mod.rs`
  (re-exports), `crates/rum-host/src/lib.rs` (export `HostTyped`), `crates/rum-host/src/tests.rs`
  (tests); ONLY touch `crates/rum-jupyter/**` if adding the field breaks its compile.
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core` / `crates/rum-runtime`
  SOURCE (host-layer only; the result `Expr` already flows out — if you think a runtime
  change is needed, STOP and report); `Expr`/serde shape; `AGENTS.md`/`PROGRESS.md`/
  `TODO.md`/`docs/**` except your own report; `.git`, `Cargo.lock`.
- **No new `unsafe`.** Not a hot-path change (conversion runs once per eval outcome) — but
  do NOT put the conversion anywhere per-eval-step.

## Deliverable

- `HostTyped` enum + `HostValue.typed`; the depth-capped `Expr -> HostTyped` conversion.
- rum-host tests asserting the typed view for: a small integer (`2 + 3` → `Integer(5)`),
  a big integer (e.g. `2^200` → `BigInteger("…")`), a rational (`1/3` → `Rational{…}`),
  a symbol (`Hold[x]`? no — use a bare evaluated symbol like `True` → `Symbol("True")` or
  a defined one), a string (`"hi"` → `Str("hi")`), a list (`{1, 2, 3}` →
  `List([Integer(1), Integer(2), Integer(3)])`), and a fallback (`f[1]` → `Other{head:"f", …}`).
  AND assert `.text` is unchanged for each (back-compat).

## rum / Rust pitfalls (read twice)

1. rationals-only — no float variant; decimals already lower to rationals.
2. Use decimal STRINGS for BigInteger/Rational components (never assume i64 fits — `2^200`
   does not). Integer(i64) only when `as_i64()` is `Some`.
3. DEPTH-CAP the recursive conversion (a deep `List[List[...]]` must not overflow the
   converter's stack) — emit `Other{text}` beyond the cap.
4. Back-compat: `text` MUST be byte-identical to today (`response.value.to_string()`);
   the typed view is additive. Do not change the rendered text.
5. Symbol names: keep them context-qualified as `to_string()` gives them (do not strip
   contexts — the embedder may need the full name).
6. Minimal diff; no drive-by churn.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace        # adding a field; confirm nothing downstream broke
```
Paste the workspace summary AND the new typed-value test output (the per-variant
assertions). No microbench needed (not a hot path) — state that the conversion is built
once in `eval_success_from_runtime`, not per eval step.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes in the tree for Claude's gate.
- Do NOT modify `rum-core`/`rum-runtime` SOURCE. Host layer only. If you think you must,
  STOP and report.
- No new `unsafe`. Never edit any `mma-hol` file. Minimal diff.
- Stop-loss: same failure twice → STOP and report.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim workspace summary + the per-variant typed-value test output;
   the chosen `MAX_TYPED_DEPTH`; confirmation `.text` is unchanged.
3. Finding — the typed view works for each Atom kind + List + the Other fallback.
4. Stop-loss / open questions (empty if none).
