# Brief 187 — Math argument/arity validation (unit-5 fix 1)

Implementation brief. Executes unit-5 triage fix-batch 1 (tracker
`docs/dev/usability-pass.md`, unit-5 section; evidence E5/E7/E9 in
`docs/dev/usability-pass/math-findings.md`). Read the tracker's unit-5
triage FIRST — especially the USER singularity decision: arithmetic
singularities (`1/0`, `0^0`, `(-1)!`, zero divisor/modulus/multiple)
STAY INERT and are OUT of scope here. This brief is ONLY about
genuinely MALFORMED ARGUMENTS where WL clearly errors (arity, an
out-of-range base, a negative length, a non-variable derivative target)
and rum currently returns a SILENT inert call. The fix adds a message
and returns the SAME inert call (byte-identical value + a message) —
the `Prime::intpp` pattern already used in this area.

Message mechanism (mirror existing): `session.invalid_call_with_message(
"Head", args, "tag", format!("…"))` emits `Head::tag` and returns the
inert `Head[args]` (see `eval_d`'s arity path in
`crates/rum-stdlib/src/calculus.rs:17` and `eval_mod` in
`crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs`). Arity uses
`session.invalid_call_with_exact_arg_count("Head", args, n)` (see
`eval_binomial` in `crates/rum-stdlib/src/combinatorics.rs:12`).

## Conservative rule (applies to every part)

Message ONLY when the offending argument is a CONCRETE value that WL
rejects. When the argument is symbolic / non-integer / otherwise
uncertain, LEAVE the current inert behavior UNCHANGED — never guess a WL
error. Valid calls stay byte-identical.

## Change 1 — `Fibonacci` arity (combinatorics.rs)

`eval_fibonacci` (`crates/rum-stdlib/src/combinatorics.rs:26`) returns a
silent inert call on wrong arity (`Fibonacci[]`, `Fibonacci[1, 2]`);
`Binomial` two functions up already emits `Binomial::argx`. Mirror it:
on `args.len() != 1`, return
`session.invalid_call_with_exact_arg_count("Fibonacci", args, 1)`. (The
fn currently binds `_session`; use `session`.) The valid `Fibonacci[n]`
path — including negative `n`, which stays as today for brief-188 to
handle — is UNCHANGED. Do NOT touch the negative-index value question
here.

## Change 2 — integer-digits base/length domain (integer_digits.rs)

For `IntegerLength`, `IntegerDigits`, `IntegerReverse`, `DigitCount`:
when the BASE argument is present and is a CONCRETE exact integer < 2
(i.e. `expr_as_exact_integer(base_expr)` is `Some(b)` with `b < 2`),
emit a base-domain message (`ibase`, text in rum's style, e.g.
`format!("Base {b} is not an integer greater than 1 in {call}.")`) and
return inert, instead of the current silent inert
(`expr_as_integer_digits_base` returns `None`). A non-integer/symbolic
base stays inert (unchanged).

For `IntegerDigits[n, base, len]`: when `len` is a CONCRETE negative
exact integer, emit a nonnegative-length message (`intnn`) and return
inert, instead of the current silent inert
(`expr_as_non_negative_machine_integer` returns `None`). A
non-integer/symbolic length stays inert.

Cite the current sites: base handling at
`crates/rum-stdlib/src/integer_digits.rs:23/53/107/180`; length at
`:62`. Keep every valid call (`IntegerDigits[5, 2, 4]`,
`DigitCount[12321, 10, 2]`, base-10 defaults) byte-identical.

## Change 3 — `D` malformed spec (calculus.rs)

`eval_d` (`crates/rum-stdlib/src/calculus.rs:15`) returns a silent inert
call when `parse_derivative_spec` returns `None` (line 28-29). Classify
the None reason and message ONLY the clearly-invalid concrete cases;
leave everything else inert:
- the spec is a CONCRETE number that cannot be a variable (`D[x^2, 1]`,
  `D[x^2, 3/2]`) → emit `D::dvar`
  (`format!("{spec} is not a valid variable.")`);
- the spec is `{var, order}` with a CONCRETE NEGATIVE integer order
  (`D[x^2, {x, -1}]`) → emit a nonnegative-order message (`dvord` or the
  rum-style tag);
- ANYTHING ELSE stays inert unchanged — in particular the symbolic-order
  form `D[x^2, {x, a}]` is DEFERRED (not this brief), and a bare symbol
  variable, `{var, n}` with valid n, and multi-spec `D[f, x, y]` all keep
  working exactly as today.

Do NOT change `parse_derivative_spec` itself or any valid-derivative
path (`D[x^3, {x,2}]` → `6 x`, `D[f[x], x]` → `Derivative[1][f][x]`).

## Tests

Add rum-stdlib tests (co-locate with the existing math tests —
`crates/rum-stdlib/src/tests/` combinatorics / integer_digits / calculus
modules; confirm the paths):
- `Fibonacci[]` and `Fibonacci[1, 2]` emit `Fibonacci::argx` and return
  the inert call; `Fibonacci[10]` → `55` unchanged.
- `IntegerDigits[5, 1]` / `IntegerLength[10, 1]` / `IntegerReverse[10, 1]`
  / `DigitCount[12321, 1]` emit the base message + inert; a valid base
  (`IntegerDigits[5, 2]`) and base-10 default are unchanged;
  `IntegerDigits[5, 2, -1]` emits the length message + inert while
  `IntegerDigits[5, 2, 4]` is unchanged.
- `D[x^2, 1]` emits `D::dvar` + inert; `D[x^2, {x, -1}]` emits the order
  message + inert; `D[x^2, {x, a}]` STAYS inert with NO message (deferred);
  `D[x^3, {x, 2}]` → `6 x` and `D[f[x], x]` unchanged.
- Keep all existing math tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# malformed args now message; valid + symbolic/uncertain stay as today:
printf '' | cargo run -q -p rum-cli -- --eval '{Fibonacci[], Fibonacci[1,2], Fibonacci[10], IntegerDigits[5,1], IntegerLength[10,1], IntegerReverse[10,1], DigitCount[12321,1], IntegerDigits[5,2,-1], IntegerDigits[5,2,4], D[x^2,1], D[x^2,{x,-1}], D[x^2,{x,a}], D[x^3,{x,2}], D[f[x],x]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
the malformed args (`Fibonacci[]`, `Fibonacci[1,2]`, base-1 calls,
`IntegerDigits[5,2,-1]`, `D[x^2,1]`, `D[x^2,{x,-1}]`) each print a
message and return their inert call; `Fibonacci[10]`→55,
`IntegerDigits[5,2,4]`, `D[x^3,{x,2}]`→`6 x`, `D[f[x],x]`, and the
symbolic-order `D[x^2,{x,a}]` are UNCHANGED and message-free.

## Scope

- MAY modify: `crates/rum-stdlib/src/combinatorics.rs` (Fibonacci
  arity), `crates/rum-stdlib/src/integer_digits.rs` (base/length
  messages), `crates/rum-stdlib/src/calculus.rs` (D spec messages), and
  the math test files under `crates/rum-stdlib/src/tests/**` (new tests).
- MUST NOT touch: any arithmetic singularity (`Divide`/`Quotient`/
  `Mod`/`PowerMod`/`Round`/`Power`/`Factorial` — all stay inert per the
  user decision); `parse_derivative_spec`; the negative-index VALUE
  question (brief-188); `FromDigits`; the guide,
  `docs/reference/builtins.md`, `TODO.md`, the tracker, findings files,
  `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve any suspend/resume shape you touch.
- Message-then-inert only: the returned expression is the SAME inert
  call as today; you ONLY add a message. Valid calls byte-identical.
- Conservative: concrete WL-error arguments only; symbolic / non-integer
  / uncertain inputs keep today's inert behavior with NO message. When in
  doubt, leave it inert and note it.
- No A/B gate needed (error path, not eval/dispatch/matcher hot path).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file/function, line ranges, the message tag used
   and the precedent mirrored.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
