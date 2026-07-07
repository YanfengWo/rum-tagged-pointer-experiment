# Control + Logic + Rewrite Usability Audit Findings

Coverage statement: 65 findings-table rows cover 63 distinct catalog names: 35
from `control.rs`, 21 from `logic.rs`, and 7 from `rewrite.rs`. The coverage
check printed `COVERAGE-OK 63/63`.

## Top 10 Gaps By Script-User Value

1. `Exit` is documented as `Capability: Pure` even though it requests host-level termination.
2. `Implies[False, rhs]` evaluates `rhs`, so it lacks the short-circuit safety users expect from implication.
3. `ReplaceAll[rules][expr]` is still missing, blocking the common pipeline form noted in earlier unit spillover.
4. `ReplaceRepeated` lacks a `MaxIterations` option, leaving only the session-wide resource limit.
5. `Cases[patt][expr]` is missing even though `MatchQ[patt][expr]` and `ReplaceList[rules][expr]` work.
6. `DeleteCases[patt][expr]` is missing, forcing wrappers in filtering pipelines.
7. `Replace[rules][expr]` is missing despite `Replace` already supporting direct levelspec calls.
8. `ReplaceRepeated[rules][expr]` is missing, forcing wrappers for repeated-rule pipelines.
9. `Sum` and `Product` leave malformed iterator specs inert without a diagnostic while `Do` and `Table` emit messages.
10. `Replace` cannot traverse heads with `Heads -> True`; `Cases` and `DeleteCases` already expose that option.

## Findings Table

### Control

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| CompoundExpression | WL | OK | | | | |
| If | WL | OK | | | | |
| Which | WL | OK | | | | |
| Switch | WL | OK | | | | |
| Do | WL | OK | | | | |
| Product | WL | gap | safety | Malformed iterator specs return inert `Product[...]` without a message, unlike the nearby `Do`/`Table` diagnostics (Evidence E4; `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:85`). | Emit the same `argform`-style diagnostic used by `Do`/`Table` while leaving valid aggregate iteration unchanged. | S |
| Sum | WL | gap | safety | Malformed iterator specs return inert `Sum[...]` without a message, unlike the nearby `Do`/`Table` diagnostics (Evidence E4; `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:85`). | Emit the same `argform`-style diagnostic used by `Do`/`Table` while leaving valid aggregate iteration unchanged. | S |
| While | WL | OK | | | | |
| Until | WL | OK | | | | |
| For | WL | OK | | | | |
| Table | WL | OK | | | | |
| Abort | WL | OK | | | | |
| Exit | WL | gap | safety | The reference exposes `Exit` as `Capability: Pure` even though its implementation raises `EvalError::Exit` for host-level termination (Evidence E9; `docs/reference/builtins.md:1217`; `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:47`). | Put `Exit` behind an explicit host/termination capability or host policy gate; default trusted CLI behavior can remain unchanged. | M |
| CheckAbort | WL | OK | | | | |
| AbortProtect | WL | OK | | | | |
| WithCleanup | WL | OK | | | | |
| Failsafe | WL | OK | | | | |
| Assert | WL | OK | | | | |
| $AssertFunction | WL | OK | | | | |
| PropagateAborts | WL | OK | | | | |
| Enclose | WL | OK | | | | |
| Confirm | WL | OK | | | | |
| ConfirmBy | WL | OK | | | | |
| ConfirmMatch | WL | OK | | | | |
| ConfirmQuiet | WL | OK | | | | |
| ConfirmAssert | WL | OK | | | | |
| Return | WL | OK | | | | |
| Break | WL | OK | | | | |
| Continue | WL | OK | | | | |
| Throw | WL | OK | | | | |
| Catch | WL | OK | | | | |
| Failure | WL | OK | | | | |
| FailureQ | WL | OK | | | | |
| Success | WL | OK | | | | |
| TerminatedEvaluation | WL | OK | | | | |

### Logic

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| And | WL | OK | | | | |
| Or | WL | OK | | | | |
| Nand | WL | OK | | | | |
| Nor | WL | OK | | | | |
| Xor | WL | OK | | | | |
| Equivalent | WL | gap | flexibility | `Equivalent` has only `Orderless`, unlike the associative logical surface users get from `Xor`; nested equivalences remain nested instead of flattening (Evidence E1, E5; `crates/rum-stdlib/src/catalog/logic.rs:39`). | Add the missing associative identity attributes if that matches the documented WL surface; default truth-table reductions unchanged. | S |
| Implies | WL | gap | safety | `Implies[False, rhs]` evaluates `rhs` before returning `True`, so a false antecedent does not short-circuit side effects (Evidence E5; `crates/rum-stdlib/src/catalog/logic.rs:45`). | Give `Implies` held-rest planning and evaluate the consequent only when the antecedent requires it; keep exact binary return forms unchanged. | M |
| Not | WL | OK | | | | |
| Equal | WL | OK | | | | |
| Unequal | WL | OK | | | | |
| Less | WL | OK | | | | |
| Greater | WL | OK | | | | |
| LessEqual | WL | OK | | | | |
| GreaterEqual | WL | OK | | | | |
| LessThan | WL | OK | | | | |
| GreaterThan | WL | OK | | | | |
| LessEqualThan | WL | OK | | | | |
| GreaterEqualThan | WL | OK | | | | |
| EqualTo | WL | OK | | | | |
| UnequalTo | WL | OK | | | | |
| SameAs | WL | OK | | | | |

### Rewrite

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| MatchQ | WL | OK | | | | |
| Cases | WL | gap | flexibility | The unary operator form `Cases[patt][expr]` emits an arity message, while the direct form and `Heads`/levelspec support work (Evidence E6, E7; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:4`). | Add the unambiguous `Cases[spec]` operator form using the existing wrapper machinery. | S |
| DeleteCases | WL | gap | flexibility | The unary operator form `DeleteCases[patt][expr]` emits an arity message, while direct levelspec and `Heads` forms work (Evidence E6, E8; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:4`). | Add `DeleteCases[patt]` as an operator form; default direct-call behavior unchanged. | S |
| Replace | WL | gap | flexibility | The unary operator form `Replace[rules][expr]` emits an arity message (Evidence E6; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:4`). | Add `Replace[rules]` as an operator form, and preserve direct two- and three-argument calls. | S |
| Replace | WL | gap | flexibility | `Replace[..., levels, Heads -> True]` is rejected, and head replacement is unreachable through the current three-argument path (Evidence E8; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:4`). | Register and honor `Heads -> False` for `Replace`; keep the current non-head traversal default. | M |
| ReplaceList | WL | OK | | | | |
| ReplaceAll | WL | gap | flexibility | The unary operator form `ReplaceAll[rules][expr]` emits an arity message (Evidence E6; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:4`). | Add the WL-documented `ReplaceAll[rules]` operator form; the arity is unambiguous and does not collide with a data form. | S |
| ReplaceRepeated | WL | gap | flexibility | The unary operator form `ReplaceRepeated[rules][expr]` emits an arity message (Evidence E6; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:4`). | Add `ReplaceRepeated[rules]` as an operator form. | S |
| ReplaceRepeated | WL | gap | safety | `MaxIterations -> n` is rejected as a third argument, `Options[ReplaceRepeated]` is empty, and users cannot lower the per-call runaway-rewrite budget (Evidence E7; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:60`). | Add `MaxIterations -> Automatic` option support that defaults to the session resource limit and permits per-call lower limits. | M |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin to `cargo run`.

### E1: Attribute Survey

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Attributes /@ {CompoundExpression, If, Which, Switch, Do, Product, Sum, While, Until, For, Table, CheckAbort, AbortProtect, WithCleanup, Assert, Enclose, Confirm, ConfirmBy, ConfirmMatch, ConfirmQuiet, ConfirmAssert, Catch}, Attributes /@ {And, Or, Nand, Nor, Xor, Equivalent, Implies, Not, Equal, Unequal, Less, Greater, LessEqual, GreaterEqual}, Attributes /@ {MatchQ, Cases, DeleteCases, Replace, ReplaceList, ReplaceAll, ReplaceRepeated}, Attributes /@ {$AssertFunction, PropagateAborts}}'
List[List[List[HoldAll], List[HoldRest], List[HoldAll], List[HoldRest], List[HoldAll], List[HoldAll, Protected, ReadProtected], List[HoldAll, Protected, ReadProtected], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll]], List[List[HoldAll], List[HoldAll], List[HoldAll], List[HoldAll], List[Orderless, Flat, OneIdentity], List[Orderless], List[], List[], List[], List[], List[], List[], List[], List[]], List[List[], List[], List[], List[], List[], List[], List[]], List[List[], List[]]]
```

### E2: Held Branch And Logic Short-Circuit Spot Checks

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'x = 0; {If[True, x = 1, x = 2], x, If[False, x = 3, x = 4], x, If[u, x = 5, x = 6, x = 7], x, Which[False, x = 8, True, x = 9, True, x = 10], x, Switch[1 + 1, 1, x = 11, 2, x = 12], x}'
List[1, 1, 4, 4, 7, 7, 9, 9, 12, 12]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'c = 0; {And[False, c = c + 1], c, Or[True, c = c + 1], c, Nand[False, c = c + 1], c, Nor[True, c = c + 1], c, And[True, c = c + 1], c, Or[False, c = c + 1], c}'
List[False, 0, True, 0, True, 0, False, 0, 1, 1, 2, 2]
```

### E3: Iteration Success Paths

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'c = 0; {Do[c = c + i, {i, 3}], c, Table[i^2, {i, 3}], Sum[i, {i, 1, 3}], Product[i, {i, 1, 4}], For[j = 1, j <= 3, j = j + 1, c = c + j]; c}'
List[Null, 6, List[1, 4, 9], 6, 24, 12]
```

### E4: Malformed Iteration Specs

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Do[1, {i, 1, 3, 0}], Table[i, {i, 1, 3, 0}], Sum[i, {i, 1, 3, 0}], Product[i, {i, 1, 3, 0}], Do[1, bad], Table[1, bad], Sum[1, bad], Product[1, bad]}'
! Do::argform: expected a supported repetition count or iterator specification
! Table::argform: expected a supported repetition count or iterator specification
! Do::argform: expected a supported repetition count or iterator specification
! Table::argform: expected a supported repetition count or iterator specification
List[Do[1, List[i, 1, 3, 0]], Table[i, List[i, 1, 3, 0]], Sum[i, List[i, 1, 3, 0]], Product[i, List[i, 1, 3, 0]], Do[1, bad], Table[1, bad], Sum[1, bad], Product[1, bad]]
```

### E5: Logic Reduction And Wrapper Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'c = 0; {Implies[False, c = c + 1], c, Implies[True, c = c + 1], c, Xor[True, False, a, a, b], Equivalent[True, a, b], Equivalent[Equivalent[a, b], c], Not[Not[a]], Less[1, 2, 3], Less[1, x, 3], LessThan[3][2], GreaterEqualThan[3][3], EqualTo[2][2], UnequalTo[2][3], SameAs[a][a]}'
List[True, 1, 2, 2, Not[b], And[a, b], Equivalent[2, Equivalent[a, b]], a, True, Less[1, x, 3], True, True, True, True, True]
```

### E6: Rewrite Operator Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{MatchQ[_Integer][1], Cases[_Integer][{1, a, 2}], DeleteCases[_Integer][{1, a, 2}], Replace[x_ :> f[x]][a], ReplaceAll[x_ :> f[x]][a], ReplaceRepeated[x_ :> f[x]][a], ReplaceList[x_ :> f[x]][a]}'
! Cases::argtu: expected between 2 and 5 argument(s), got 1
! DeleteCases::argtu: expected between 2 and 5 argument(s), got 1
! Replace::argtu: expected between 2 and 3 argument(s), got 1
! ReplaceAll::argx: expected exactly 2 argument(s), got 1
! ReplaceRepeated::argx: expected exactly 2 argument(s), got 1
! Cases::argtu: expected between 2 and 5 argument(s), got 1
! DeleteCases::argtu: expected between 2 and 5 argument(s), got 1
! Replace::argtu: expected between 2 and 3 argument(s), got 1
! ReplaceAll::argx: expected exactly 2 argument(s), got 1
! ReplaceRepeated::argx: expected exactly 2 argument(s), got 1
List[True, Cases[Blank[Integer]][List[1, a, 2]], DeleteCases[Blank[Integer]][List[1, a, 2]], Replace[RuleDelayed[Pattern[x, Blank[]], f[x]]][a], ReplaceAll[RuleDelayed[Pattern[x, Blank[]], f[x]]][a], ReplaceRepeated[RuleDelayed[Pattern[x, Blank[]], f[x]]][a], List[f[a]]]
```

### E7: Rewrite Options, Levels, And `MaxIterations`

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Options /@ {Cases, DeleteCases, Replace, ReplaceAll, ReplaceList, ReplaceRepeated, CheckAbort}, Cases[{1, f[2]}, _Integer, Infinity], Cases[{1, f[2]}, _Integer, Infinity, 1], Cases[f[1], _Symbol, {0, Infinity}, Heads -> True], DeleteCases[{1, f[2]}, _Integer, Infinity, 1], Replace[{1, f[2]}, x_Integer :> z, {0, Infinity}], ReplaceRepeated[a, {a -> b, b -> c}], ReplaceRepeated[a, a -> a + 1, MaxIterations -> 2]}'
! ReplaceRepeated::argx: expected exactly 2 argument(s), got 3
! ReplaceRepeated::argx: expected exactly 2 argument(s), got 3
List[List[List[Rule[Heads, False]], List[Rule[Heads, False]], List[], List[], List[], List[], List[Rule[PropagateAborts, Automatic]]], List[1, 2], List[1], List[f], List[f[2]], List[z, f[z]], c, ReplaceRepeated[a, Rule[a, Plus[a, 1]], Rule[MaxIterations, 2]]]
```

### E8: `Replace` Heads Traversal

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Replace[f[1], f -> g, {0, Infinity}], Replace[f[1], f -> g, {0, Infinity}, Heads -> True], Cases[f[1], f, {0, Infinity}, Heads -> True], DeleteCases[f[1], f, {0, Infinity}, Heads -> True]}'
! Replace::argtu: expected between 2 and 3 argument(s), got 4
! Replace::argtu: expected between 2 and 3 argument(s), got 4
List[f[1], Replace[f[1], Rule[f, g], List[0, Infinity], Rule[Heads, True]], List[f], 1]
```

### E9: Control, Cleanup, Confirmation, And Status Spot Checks

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Catch[Throw[1]], Catch[Throw[1, tag], tag], Catch[Throw[1, tag], _, f], CheckAbort[Abort[], fallback], AbortProtect[1 + 1], CheckAbort[AbortProtect[Abort[]], fallback]}'
List[1, 1, f[1, tag], fallback, 2, fallback]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'CheckAbort[AbortProtect[Abort[]], fallback, PropagateAborts -> True]'
$Aborted
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'x = 0; {WithCleanup[x = 1, x = 2]; x, CheckAbort[WithCleanup[Abort[], x = 3], caught]; x, Enclose[Confirm[$Failed]], Enclose[ConfirmBy[3, EvenQ]], Enclose[ConfirmMatch[3, _String]], Enclose[ConfirmQuiet[Message[General::stop]; 1]], Enclose[ConfirmAssert[False]]}'
List[2, 3, Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "`` encountered."], RuleDelayed["MessageParameters", List[$Failed]], Rule["ConfirmationType", "Confirm"], RuleDelayed["Expression", $Failed], Rule["Information", Null]]], Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "``[``] did not return True."], RuleDelayed["MessageParameters", List[EvenQ, 3]], Rule["ConfirmationType", "ConfirmBy"], RuleDelayed["Expression", 3], Rule["Information", Null], Rule["Function", EvenQ]]], Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "`` does not match ``."], RuleDelayed["MessageParameters", List[3, Blank[String]]], Rule["ConfirmationType", "ConfirmMatch"], RuleDelayed["Expression", 3], Rule["Information", Null], Rule["Pattern", Blank[String]]]], Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "`` produced a message on evaluation."], RuleDelayed["MessageParameters", List[HoldForm[CompoundExpression[Message[General::stop], 1]]]], Rule["ConfirmationType", "ConfirmQuiet"], RuleDelayed["Expression", Hold[CompoundExpression[Message[General::stop], 1]]], Rule["Information", Null], Rule["HeldExpression", Hold[CompoundExpression[Message[General::stop], 1]]], Rule["HeldMessage", Hold[General::stop]], Rule["HeldMessageName", Hold[General::stop]]]], Failure["ConfirmationFailed", Association[Rule["MessageTemplate", "Assertion `` failed."], RuleDelayed["MessageParameters", List[HoldForm[False]]], Rule["ConfirmationType", "ConfirmAssert"], RuleDelayed["Expression", False], Rule["Information", Null], RuleDelayed["Test", False], Rule["HeldTest", Hold[False]]]]]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval 'seen = none; {Assert[False], seen, $AssertFunction = Function[x, seen = x]; Assert[False, "ctx"]; Head[seen], seen}'
List[Null, none, HoldComplete, HoldComplete[Assert[False, "ctx"]]]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Failure["tag", <|"MessageTemplate" -> "bad `x`", "MessageParameters" -> <|"x" -> 1|>|>]["Message"], FailureQ[Failure["tag", <||>]], Success["ok", <|"Value" -> 1|>]["Properties"], Failsafe[f][Failure["tag", <||>]], Failsafe[f][1], Failsafe[f, Positive][1], Failsafe[f, Positive][-1]}'
List["bad 1", True, List["Value"], Failure["tag", Association[]], f[1], f[1], Failure["FailsafeFailed", Association[Rule["MessageTemplate", "The test failed on the arguments `1`."], RuleDelayed["MessageParameters", List[HoldForm[-1]]], Rule["Arguments", Hold[-1]]]]]
```

### E10: Source Citations

- `Exit` is generated in the reference as host termination with `Capability: Pure` at `docs/reference/builtins.md:1217`; the implementation returns `EvalError::Exit` at `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:47`.
- `Sum`/`Product` invalid specs fall through to `Expr::call(...)` without a message at `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:85`.
- `Equivalent` is registered with only `Orderless` at `crates/rum-stdlib/src/catalog/logic.rs:39`; `Implies` is registered as an ordinary non-held builtin at `crates/rum-stdlib/src/catalog/logic.rs:45`.
- Rewrite direct arity gates are visible at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:4`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:4`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:4`, `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:4`, and `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:4`; `ReplaceList` is the contrast case with a unary operator path at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:10`.
- `ReplaceRepeated` uses only the session `replace_repeated_limit` in its loop guard at `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:60`; no default option is registered for it in `crates/rum-runtime/src/runtime/options/defaults.rs:22`.

## Triage Sketch

1. **Host termination and implication safety**: gate `Exit` behind an explicit host capability/policy and give `Implies` held-rest short-circuit behavior. `Implies` touches call planning/hold attributes, so it needs the A/B microbench gate.
2. **Rewrite operator-form parity**: add `Cases[spec]`, `DeleteCases[patt]`, `Replace[rules]`, `ReplaceAll[rules]`, and `ReplaceRepeated[rules]`. This is a call-planning batch and needs the A/B microbench gate.
3. **Per-call rewrite runaway control**: register and honor `MaxIterations -> Automatic` for `ReplaceRepeated`, defaulting to the session limit. This is mostly options plus loop plumbing, not call planning.
4. **Replace traversal option**: register and honor `Heads -> False` for `Replace` levelspec traversal, matching `Cases`/`DeleteCases` discovery. This is traversal behavior, not call planning.
5. **Low-risk diagnostics and attribute parity**: make `Sum`/`Product` malformed iterator specs message like `Do`/`Table`, and decide whether `Equivalent` should gain `Flat`/`OneIdentity`. The `Equivalent` attribute change touches attribute-driven planning and should use the A/B gate.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/control.rs
35
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/logic.rs
21
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/rewrite.rs
7
```

```text
$ names=$(rg -o '"[$A-Za-z][A-Za-z0-9$]*"' crates/rum-stdlib/src/catalog/control.rs crates/rum-stdlib/src/catalog/logic.rs crates/rum-stdlib/src/catalog/rewrite.rs | sed 's/.*"\([^"]*\)"/\1/' | sort -u); rows=$(awk -F'|' '/^\| [$A-Za-z][A-Za-z0-9$]* \|/ {gsub(/ /,"",$2); if ($2 != "Builtin") print $2}' docs/dev/usability-pass/control-logic-rewrite-findings.md | sort -u); missing=$(comm -23 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); extra=$(comm -13 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); count=$(printf '%s\n' "$rows" | sed '/^$/d' | wc -l | tr -d ' '); if [ -z "$missing$extra" ] && [ "$count" = "63" ]; then echo "COVERAGE-OK 63/63"; else echo "COVERAGE-FAIL $count/63"; [ -z "$missing" ] || printf 'missing:\n%s\n' "$missing"; [ -z "$extra" ] || printf 'extra:\n%s\n' "$extra"; fi
COVERAGE-OK 63/63
```

```text
$ git status --short
?? docs/dev/usability-pass/control-logic-rewrite-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

- None.
