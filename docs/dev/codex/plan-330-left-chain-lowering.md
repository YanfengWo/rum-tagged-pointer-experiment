# Plan 330 — make left-chain lowering iterative without erasing binary semantics

This is a design-only result. No Rust source, test, script, generated reference,
or existing documentation was changed. The five follow-up briefs at the end are
raw drafts in `TEMPLATE.md` shape; the director must review and rewrite them
before dispatch.

One premise in brief 330 is false: `BinaryOp` has **48**, not 47, variants. The
enum occupies `crates/rum-syntax/src/ast.rs:197-246`, and the brief's measured
groups also add up to 16 + 24 + 8 = 48. This does not invalidate any measured
operator result.

## 1. Sixteen-operator classification

“Flat” below means that the source chain should lower to one n-ary call. “Nested”
means that lowering should walk and rebuild the chain iteratively but retain its
left-folded computation and shape. Confidence is about what the current rum
repository establishes; it is not based on another language or engine.

| Token (`BinaryOp`) | Decision | Result for three/four operands | Repository-owned reason | Confidence |
|---|---|---|---|---|
| `+` (`Add`) | **Flat** | `a+b+c` → `Plus[a,b,c]` | Binary `+` already targets `Plus` (`crates/rum-runtime/src/runtime/lowering.rs:387-390`), and `Plus` is explicitly `Flat` (`crates/rum-stdlib/src/catalog/arithmetic.rs:13-22`). | High |
| `-` (`Subtract`) | **Flat with `+`** | `a+b-c+d` → `Plus[a,b,Times[-1,c],d]` | `-` is lowering sugar for a `Plus` term whose rhs is multiplied by `-1` (`crates/rum-runtime/src/runtime/lowering.rs:391-400`); both `Plus` and `Times` are `Flat` (`crates/rum-stdlib/src/catalog/arithmetic.rs:13-32`). | High |
| `*` (`Times`) | **Flat** | `a*b*c` → `Times[a,b,c]` | Binary `*` targets `Times` (`crates/rum-runtime/src/runtime/lowering.rs:401-404`), and `Times` is explicitly `Flat` (`crates/rum-stdlib/src/catalog/arithmetic.rs:23-32`). | High |
| `/` (`Divide`) | **Flat with `*`**, preserving the literal-quotient fast path | `a*b/c*d` → `Times[a,b,Power[c,-1],d]` | Nonliteral division becomes multiplication by a reciprocal (`crates/rum-runtime/src/runtime/lowering.rs:410-424`), while two exact literal operands are combined immediately (`crates/rum-runtime/src/runtime/lowering.rs:159-163,410-413`). The chain builder must retain that distinction. | High |
| `.` (`Dot`) | **Flat, subject to a director ruling** | `a.b.c` → `Dot[a,b,c]` | `eval_dot` accepts any nonempty arity and left-folds over all remaining operands (`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:31-53`). However, `Dot` is not `Flat` (`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_a_e.rs:126-132`), unsupported operands return the original call (`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:47-49`), and the golden explicitly pins `Dot[Dot[a,b],c]` (`crates/rum-runtime/src/runtime/tests/lowering_golden.rs:29`). Flattening is the better long-chain representation, but it is an intentional observable contract change and needs the ruling called out below. | Ruling required |
| `**` (`NonCommutativeMultiply`) | **Nested** | `a**b**c` → `NonCommutativeMultiply[NonCommutativeMultiply[a,b],c]` | The repository establishes only binary lowering (`crates/rum-runtime/src/runtime/lowering.rs:406-409`) and left-associated syntax (`crates/rum-syntax/src/parser/tests/infix_operators.rs:53-67`). It registers neither a builtin nor attributes: the alphabetical attribute inventory moves from `NonPositive` directly to `NoneTrue` (`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_n_r.rs:12-15`). There is no rum-owned basis for declaring the head n-ary/associative. | High for preservation; future n-ary semantics would be a separate language ruling |
| `<>` (`StringJoin`) | **Flat** | `"a"<>"b"<>"c"` → `StringJoin["a","b","c"]` | `StringJoin` iterates over arbitrary argument slices (`crates/rum-stdlib/src/strings/text/builtins.rs:9-36`), and direct multi-argument calls are part of the test surface (`crates/rum-stdlib/src/tests/strings/search_replace.rs:82-89`). It is not attributed `Flat` (`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_s_z.rs:36-41`), so held and invalid residual shapes deliberately change; those are gate cases, not facts to hide. | Medium-high |
| `::` (`MessageName`) | **Nested** | `a::b::c` stays a left fold of two-argument `MessageName` forms | The evaluator requires exactly two arguments (`crates/rum-runtime/src/runtime/messages/builtins_messages.rs:31-47`), and message selectors likewise accept only a two-argument `MessageName` (`crates/rum-runtime/src/runtime/messages/message_selector_parsing.rs:83-104`). | High |
| `/.` (`ReplaceAll`) | **Nested** | `a/.r/.s` → `ReplaceAll[ReplaceAll[a,r],s]` | `ReplaceAll` requires exactly target + rules and evaluates/replaces the target before applying that link's rules (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:4-19,25-54`). A flattened call is invalid and cannot represent sequential rule application. | High |
| `//.` (`ReplaceRepeated`) | **Nested** | `a//.r//.s` → `ReplaceRepeated[ReplaceRepeated[a,r],s]` | Each call accepts target + rules, with only a recognized third option allowed (`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:4-17`). Combining chain links into arguments would either be invalid or reinterpret a link as an option. | High |
| `:` (`Pattern`) | **Nested** | `a:b:c` → `Optional[Pattern[a,b],c]` | The lowering choice is stateful per link: a symbol lhs yields binary `Pattern`, while any other lowered lhs yields binary `Optional` (`crates/rum-runtime/src/runtime/lowering.rs:350-358`). The matcher recognizes `Pattern` only at arity two and `Optional` only at arity one or two (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:162-169,206-214`). | High |
| `|` (`Alternatives`) | **Flat** | `a|b|c` → `Alternatives[a,b,c]` | The documented pattern surface demonstrates a three-way source chain (`docs/guide/patterns-and-rules.md:31-54`); the matcher consumes all arguments (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:246-251`) and compiles each in order (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:63-65`). | High |
| `?` (`PatternTest`) | **Nested** | `a?f?g` → `PatternTest[PatternTest[a,f],g]` | `PatternTest` is recognized only at arity two and applies one test to one inner pattern (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:183-191`); the compiled matcher preserves that nesting (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:39-43`). | High |
| `~~` (`StringExpression`) | **Flat** | `a~~b~~c` → `StringExpression[a,b,c]` | The guide defines `~~` as sequence syntax (`docs/guide/string-patterns.md:7-16`); the string-pattern parser accepts all nonempty arguments as an ordered sequence (`crates/rum-stdlib/src/strings/patterns.rs:428-434`), and the matcher advances through every sequence part in order (`crates/rum-stdlib/src/strings/patterns/matcher/engine.rs:191-208,306-328`). | High |
| `//` (`PostfixApply`) | **Nested** | `x//f//g` → `g[f[x]]` | Each link is directly lowered as `rhs[lhs]` (`crates/rum-runtime/src/runtime/lowering.rs:373-375`), and the exact three-term shape is already golden (`crates/rum-runtime/src/runtime/tests/lowering_golden.rs:22-23`). There is no n-ary head to emit. | High |
| `/*` (`RightComposition`) | **Flat** | `f/*g/*h` → `RightComposition[f,g,h]` | `RightComposition` is explicitly `Flat` and `OneIdentity` (`crates/rum-stdlib/src/catalog/functional.rs:380-387`); application walks its arbitrary function list from left to right (`crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:23-27,49-58`). | High |

The recommended table therefore has nine flat operators (`+`, `-`, `*`, `/`,
`.`, `<>`, `|`, `~~`, `/*`) and seven nested operators (`**`, `::`, `/.`,
`//.`, `:`, `?`, `//`). Only Dot's existing golden-vs-n-ary policy needs a
director ruling before its shape-changing step is dispatched. The conservative
decision for undocumented `**` is to retain the only shape rum currently
establishes; adding n-ary `NonCommutativeMultiply` semantics is outside this fix.

## 2. Step-by-step implementation plan

### Step 1 — isolate binary lowering and remove comparison recursion

- **Files/functions:** move the binary-specific block from
  `crates/rum-runtime/src/runtime/lowering.rs:273-577` behind a new
  `crates/rum-runtime/src/runtime/lowering/binary.rs` coordinator; put comparison
  traversal/output in `crates/rum-runtime/src/runtime/lowering/binary/comparison.rs`;
  keep the entry dispatch at `crates/rum-runtime/src/runtime/lowering.rs:33-35`.
  Replace `collect_comparison_chain` with an
  explicit in-order work stack used by `lower_comparison_chain`.
- **Why first:** comparisons already have their own n-ary/mixed-chain contract
  (`crates/rum-runtime/src/runtime/lowering.rs:501-553`) and their collector is an
  independent recursion defect (`crates/rum-runtime/src/runtime/lowering.rs:555-568`).
  Separating it makes the output-preserving safety change independently
  reviewable before any of the sixteen change shape.
- **Iteration:** seed a LIFO stack with `Visit(rhs), EmitOp(root_op), Visit(lhs)`.
  A comparison `Visit` pushes its rhs, operator marker, and lhs in reverse
  execution order; a leaf appends one operand. This reproduces the current
  in-order vectors without a native recursive call.
- **Organization:** `lowering.rs` is 22,671 characters today, inside the
  20k–35k split-plan range; the guidelines require a split assessment on touch
  (`docs/code-organization-guidelines.md:70-81`). Binary lowering is a real
  syntax-vs-other-lowering responsibility boundary, not an arbitrary size cut.
- **Gate:** all existing comparison goldens remain byte-identical, including
  uniform and mixed chains (`crates/rum-runtime/src/runtime/tests/lowering_golden.rs:42-48`);
  add parenthesized-left/right subtree cases and a long uniform/mixed-chain test
  inspected without recursive `to_string`. Run the focused runtime tests, then
  the workspace. A bounded release `Head[Hold[...]]` comparison probe must exit
  normally at a length beyond the current recursive collector's failure case;
  if later `SyntaxExpr` destruction or rendering is the next failure, report it
  rather than misattribute it.

### Step 2 — one all-operator lhs spine, binary-preserving rebuild

- **Files/functions:** create `lowering/binary/left_chain.rs` and keep only its
  dispatch hook in `lowering/binary.rs`. In `left_chain.rs`, add a borrowed link record
  `(BinaryOp, &SyntaxExpr rhs)`, an `is_iterative_left_chain_op` predicate for
  all sixteen measured operators, and a `collect_left_spine` loop. Route all
  sixteen through one iterative fold while still emitting their current binary
  shapes. Add focused tests in a new
  `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs` and register it
  in `crates/rum-runtime/src/runtime/tests/mod.rs`.
- **Collection rule:** initialize the link vector with the outer `(op, rhs)`,
  then repeatedly inspect the lhs. While it is any of the sixteen affected
  binary operators, push its `(op, rhs)` and advance to its lhs; reverse once at
  the end. Do not stop at an operator-family transition. This covers ordinary
  mixed families such as `Add`/`Subtract` and `ReplaceAll`/`ReplaceRepeated`, and
  also prevents an adversarial direct `SyntaxExpr` from alternating unrelated
  affected operators to recover recursive depth.
- **Rebuild rule:** lower the innermost/base expression once, then iterate links
  in source order and update an `Expr` accumulator with exactly the current
  binary transformation. In this step every operator remains binary, including
  the nine later selected for flattening. `:` chooses `Pattern` versus
  `Optional` from the current accumulator; `//` uses `rhs[accumulator]`.
  `MessageName` is the deliberate exception to a simple “lower each rhs once”
  rule: the current dispatcher first lowers its rhs and, for a non-symbol lhs
  plus a non-symbol/string tag, `lower_message_tag_expr` lowers that raw rhs a
  second time (`crates/rum-runtime/src/runtime/lowering.rs:290-305,579-590`).
  Preserve that resolver call count/order in this output-preserving step rather
  than silently folding an unrelated cleanup into the stack fix.
- **Invariant:** all short-chain output stays byte-identical and symbol resolver
  calls retain their count and left-to-right order. The latter is observable to
  the generic `FnMut` resolver contract (`crates/rum-runtime/src/runtime/lowering.rs:9-15`),
  even though the session resolver itself only borrows session state
  (`crates/rum-runtime/src/runtime/resolution.rs:66-69`).
- **Gate:** exact three-link goldens for all sixteen, alternating same-family
  chains, a synthetic cross-family direct AST, resolver call-count/order tests
  including the `MessageName` exception, and ordinary-worker deep tests.
  Bounded release probes for all sixteen measured operators use
  `Head[Hold[chain]]`, stdin from `/dev/null`, and must pass at 20,000 links.
  This proves the recursive defect is removed before Step 5 deliberately imposes
  a lower resource ceiling.

### Step 3 — flatten the `Flat`-backed arithmetic/composition families

- **Files/functions:** extend the Step-2 spine collector in
  `lowering/binary/left_chain.rs` for additive (`Add|Subtract`), multiplicative
  (`Times|Divide`), and `RightComposition` families. Update
  `crates/rum-runtime/src/runtime/tests/lowering_golden.rs`, the focused long-chain tests,
  `docs/lowering-canonical-forms.md`, and `docs/guide/syntax.md`.
- **Additive builder:** start a syntax-chain-owned argument vector with the
  lowered base. Append an ordinary rhs for `+`; append
  `Times[-1, lowered_rhs]` for `-`. Emit one `Plus`. Do not inspect and absorb an
  explicitly written direct `Plus[...]` base: direct canonical calls must be
  preserved by lowering (`docs/lowering-canonical-forms.md:22-30,52-60`).
- **Multiplicative builder:** use a small state that distinguishes an opaque
  current expression from a chain-owned factor vector. `*` appends a factor;
  `/` first applies the existing exact-literal quotient rule when the entire
  current accumulator and rhs are numeric atoms, otherwise appends
  `Power[rhs,-1]`. This preserves `8/2/4`'s current literal folding while making
  `a*b/c*d` one `Times` call. Do not flatten a direct `Times[...]` base.
- **Composition builder:** lower functions in source order and emit one
  `RightComposition` call. Its existing `Flat` attribute and callable parser are
  the semantic authority (`crates/rum-stdlib/src/catalog/functional.rs:380-387`;
  `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:15-27`).
- **Family transitions:** make the fold accumulator an explicit state. Consecutive
  links in the active flat family append to its owned vector; a different family
  first materializes that vector to one `Expr`, then applies the next link. This
  retains Step 2's protection for arbitrary mixed direct ASTs without merging
  across a source boundary.
- **Gate:** small shape goldens for homogeneous and mixed arithmetic; numeric
  literal cases (`8/2/4`, `8*2/4`, `8/2*x`); direct-call opacity; held `SameQ`
  observability; valid evaluated results; and bounded 20,000-link release probes
  for all five tokens. Generated builtin reference output must not change because
  no catalog spec changes.

### Step 4 — flatten rum's variadic non-`Flat` heads

- **Prerequisite ruling:** the director must explicitly approve changing the Dot
  golden from nested to n-ary. If the ruling is “preserve Dot,” route `.` through
  Step 2's nested fold instead; the other three operators remain independently
  implementable.
- **Files/functions:** extend the spine collector for `.`, `<>`, `|`, and `~~` in
  `lowering/binary/left_chain.rs`; update the golden and focused tests plus the same two
  lowering/syntax docs. Do **not** add `Flat` attributes: that would change direct
  calls throughout evaluation and matching, broader than syntax lowering.
- **Builder:** lower the base and rhs operands in source order into one
  chain-owned vector, then emit `Dot`, `StringJoin`, `Alternatives`, or
  `StringExpression`. An explicitly written direct canonical call at the base
  remains one opaque operand.
- **Observable negative cases:** pin the intentional changes for held values and
  unsupported symbolic `Dot`/`StringJoin` calls. Pin typical pattern/string
  behavior unchanged. The testing strategy explicitly includes held inputs and
  inert residual shape as contract axes (`docs/testing-strategy.md:39-55`).
- **Gate:** the old Dot golden is deliberately updated; n-ary valid-domain
  results remain unchanged; explicit nested canonical calls remain untouched;
  `Verbatim` matching demonstrates the documented held-shape change; bounded
  20,000-link release probes for all four tokens exit 0.

### Step 5 — add a fixed 4,096-link source/lowering ceiling

- **Recommendation:** `MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS = 4_096`, counted as
  binary links. Exactly 4,096 is admitted and link 4,097 is rejected at its
  operator token. This is 32 times the existing 128-link native-recursion guard,
  but remains well below the measured danger zone of a repository-owned
  consumer. With a 2 MiB process stack, the existing release formatter passed
  8,000 links and aborted by 12,000; exact 4,096 and 4,097 probes both passed
  before the cap existed. The proposed boundary is therefore about 2.9 times
  below the first observed failing point and has an exact positive probe. The
  director's 20,000-link cases remain Step 2–4 stress tests proving iteration;
  after this policy step they must return the dedicated error, never abort.
- **Files/functions:** define one shared public constant in `rum-syntax` so both
  the parser and `rum-runtime` collector use the same value; count every
  left-associative link in `parser/front.rs`, including implicit `Times`; add a
  dedicated `SyntaxError::LeftAssociativeChainTooLong { max, span }` and its
  `span()` arm; enforce the same total in runtime collection for programmatically
  constructed `SyntaxExpr`. Update parser/runtime/formatter tests,
  `docs/guide/resource-limits-and-sandbox.md`, and the parse-limit table at
  `docs/dev/publication-audit/capability-consistency-findings.md:714-716`.
- **Why not reuse 128:** `MAX_OPERATOR_CHAIN_DEPTH` is a native-stack guard for
  recursive right-associative parsing (`crates/rum-syntax/src/parser/front.rs:11-20,115-130`).
  The new loop has no per-link native frame; its ceiling is an allocation/work
  policy, so it needs its own name, error, and much larger value.
- **Counting:** one local parser counter owns one precedence-climbing lhs spine;
  independent nested/right-hand spines start fresh. Capture the offending
  operator's token span before advancing and report exactly
  `left-associative chain exceeded limit 4096 at bytes {span}`. The lowerer
  bounds the all-sixteen lhs vector and the total comparison-chain operator
  vector, including comparison subtrees reached through explicit parentheses.
- **Gate:** exact limit and one-past tests, a 20,000-link graceful-rejection
  regression, homogeneous and mixed chains,
  implicit multiplication, independent spines, `is_incomplete() == false`,
  diagnostic span/caret, direct-AST lowering, and formatter behavior. Run the
  accepted boundary and rejection path on an ordinary test worker, a 2 MiB
  thread, and the release CLI. If 4,096 overflows in recursive `SyntaxExpr` drop,
  lowering, or formatter code on the 2 MiB embedding stack after the change,
  stop and return to the director; do not ship a nominal limit whose own
  accepted boundary aborts.

## 3. Answers to the six design questions

### 3.1 Per operator: flatten or keep nested

The table in section 1 is the decision. Five operators flatten because their
target head is explicitly `Flat` (`+`, `-`, `*`, `/`, `/*`); four flatten because
rum's own evaluator/matchers define ordered n-ary behavior (`.`, `<>`, `|`,
`~~`); seven stay nested because they are exact-binary transformations or have
no n-ary contract (`**`, `::`, `/.`, `//.`, `:`, `?`, `//`). Dot is the one
shape policy that must be affirmed because a checked-in golden and symbolic
residual behavior currently expose nesting.

### 3.2 Iteration shape for both classes

One borrowed **lhs-spine link vector** serves all sixteen classes. Collection is
a loop down `lhs` across every affected operator, not a same-token or
same-family walk; reversal restores source order. Step 2 first rebuilds every
link through an `Expr` accumulator with byte-identical binary behavior. Steps 3
and 4 turn that accumulator into an explicit state: an ordinary `Expr`, or a
chain-owned argument vector tagged with its active flat family. A family
transition materializes the vector before applying the next link. The
chain-owned marker is important: flattening based merely on the lowered head
would absorb explicitly written canonical calls, contradicting the direct-call
preservation rule (`docs/lowering-canonical-forms.md:22-30`).

Comparisons need a second collection mechanism: their current helper traverses
the entire comparison subtree in order, both lhs and rhs
(`crates/rum-runtime/src/runtime/lowering.rs:555-568`), rather than only one lhs
spine. An explicit `Visit`/`EmitOp` stack preserves that exact behavior. Sharing
the link record/rebuild utilities is fine; forcing these different traversals
through one collector would obscure the invariant.

### 3.3 Mixed `Subtract` and `Divide` chains

The shared spine collection crosses every affected operator; **coalescing is by
lowering family**, not by one token. `Add` and `Subtract` belong to one additive
family, so `a+b-c+d` produces the ordered terms
`[a,b,Times[-1,c],d]` and one `Plus`. `Times` and `Divide` similarly share one
multiplicative family, with each division rhs converted to `Power[rhs,-1]`.
This avoids the operator-at-a-time failure where every `-` or `/` transition
would recursively re-enter lowering.

Division has an extra invariant: `divided_numeric_literals` currently combines
an exact numeric lhs/rhs during lowering (`crates/rum-runtime/src/runtime/lowering.rs:159-163,410-413`).
A streaming opaque-expression/owned-factors state preserves that rule. It must
not mechanically turn every literal division into three `Times`/`Power` nodes.

### 3.4 Comparison path

It joins this work as **Step 1**, but stays independently gateable. Its output is
already the desired flat same-head call or `And` of adjacent mixed comparisons
(`crates/rum-runtime/src/runtime/lowering.rs:516-552`); only collection changes.
Doing it first removes the known higher-threshold recursion without coupling its
unchanged semantics to the sixteen-operator shape decisions.

### 3.5 Remaining ceiling

Yes: use a fixed **4,096 links per left-associative chain** in both normal source
parsing and direct-AST lowering. Iteration removes stack growth, not linear AST,
link-vector, lowered-argument, or output allocation: every binary syntax node
owns boxed children (`crates/rum-syntax/src/ast.rs:70-78,156-160`). The normal
parse path also lexes the entire source before parsing (`crates/rum-syntax/src/parser/front.rs:29-32`),
so this is not a substitute for a future host input-byte ceiling; it specifically
bounds additional chain-shaped AST/lowering work.

The initial 20,000 recommendation was rejected by the required stack evidence.
The existing formatter recursively walks `SyntaxExpr` children while building
group plans (`crates/rum-syntax/src/formatter/planning.rs:317-381`). It passed
20,000 links on the release main thread, but the same input aborted on a 2 MiB
stack. A six-process bracket passed at 8,000 and failed at 12,000; exact 4,096
and 4,097 both passed. Choosing 4,096 gives an exact positive check, is 32 times
the existing recursive parser limit, and sits about 2.9 times below the first
observed failing point. The final implementation must still prove parse, lower,
format, and drop at that exact boundary on a dedicated 2 MiB test thread.

### 3.6 Blast radius

- **Lowering tests/goldens:**
  `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` must change for
  flattened source chains, especially its existing Dot row at line 29. A new
  behavior-focused `left_chain_lowering.rs` should own long-chain, mixed-family,
  source-order, opacity, held-shape, and direct-AST cases rather than swelling
  `lowering_and_context.rs`.
- **Parser/error tests:** only Step 5 changes parser acceptance. Add a dedicated
  parser test module, the exhaustive `SyntaxError::span()` ripple
  (`crates/rum-syntax/src/error.rs:50-62`), and a false `is_incomplete()` pin
  (`crates/rum-syntax/src/error.rs:39-48`). Parser AST associativity remains
  unchanged (`crates/rum-syntax/src/ast.rs:383-410`).
- **Docs:** chain canonical forms change in
  `docs/lowering-canonical-forms.md:79-97` and `docs/guide/syntax.md:133-144`;
  Step 5 adds a fourth parser limit beside the three documented at
  `docs/guide/resource-limits-and-sandbox.md:111-140`. No builtin catalog or
  attribute changes are planned, so `docs/reference/builtins.md` should remain
  generated-identical.
- **Formatter round trip:** lowering-shape changes do not affect formatting.
  `format_source` parses to `ParsedSource` and passes that syntax document to the
  formatter (`crates/rum-syntax/src/formatter/mod.rs:41-59`); `ParsedSource`
  stores `SyntaxExpr` directly (`crates/rum-syntax/src/document.rs:43-48,326-358`).
  It never consumes lowered `Expr`. Step 5's cap does affect over-limit formatter
  input because formatting parses first, and the formatter has its own recursive
  syntax walk; this is why its boundary is an explicit gate.
- **Held expressions:** lowering precedes evaluation, while `Hold` and
  `HoldComplete` prevent argument evaluation (`crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs:120-155`).
  Therefore every flat decision changes held `Expr` shape and `SameQ`/rendered
  full form even when ordinary evaluation would normalize the head later.
- **Pattern matching held expressions:** yes, it can observe changes for heads
  without `Flat`. `Verbatim` compiles to an exact matcher
  (`crates/rum-runtime/src/runtime/matcher_and_symbols.rs:3-8`), and the release
  probe below distinguishes nested versus flat held `Dot`, `StringJoin`,
  `Alternatives`, and `StringExpression`. For `Plus`/`Times`/`RightComposition`,
  ordinary match normalization recursively applies `Flat` attributes
  (`crates/rum-runtime/src/runtime/resolution.rs:9-25,47-57`), so both groupings
  can match even though `SameQ` still distinguishes their held trees.

## 4. Open questions for the director

1. **Dot shape ruling (blocking Step 4 for `.`):** approve the recommended
   `Dot[a,b,c]` canonical source-chain form and update the existing nested golden,
   accepting that held and unsupported symbolic residual shapes change; or keep
   Dot nested while still using the iterative fold. Both preserve valid tensor
   results because `eval_dot` already left-folds n-ary arguments
   (`crates/rum-stdlib/src/functional_ops/arrays/mod.rs:31-53`).
2. **Ceiling policy:** accept the recommended 4,096-link public ceiling, under
   which the measured 20,000-link crash inputs become clean syntax errors; or
   require 20,000 links to remain accepted. The latter choice needs a separate
   expansion of scope to make the formatter and possibly recursive
   `SyntaxExpr` destruction safe on a 2 MiB stack. The existing formatter has
   already shown that publishing 20,000 as safe without that work would be
   false.

No repository evidence supports flattening `NonCommutativeMultiply`; the plan's
nested decision is not blocked. A future decision to give that head `Flat` or
n-ary evaluator semantics belongs in a separate language-contract brief.

## 5. Release-CLI probe transcript

All probes used the existing `target/release/rum-cli`, used only rum's own
behavior, made no network calls, and fed stdin from `/dev/null`. The first small
shape probe was below; every small semantic probe in this section exited 0.

```text
$ ./target/release/rum-cli --eval 'Hold[a+b-c+d]' < /dev/null
Hold[Plus[Plus[Plus[a, b], Times[-1, c]], d]]
```

Current chain shapes:

```text
$ ./target/release/rum-cli --eval 'FullForm[HoldComplete[{a+b-c+d, a*b/c*d, a.b.c, a**b**c, "a"<>"b"<>"c", a::b::c, a/.r/.s, a//.r//.s, a:b:c, a|b|c, a?f?g, a~~b~~c, x//f//g, f/*g/*h}]]' < /dev/null
"HoldComplete[List[Plus[Plus[Plus[a, b], Times[-1, c]], d], Times[Times[Times[a, b], Power[c, -1]], d], Dot[Dot[a, b], c], NonCommutativeMultiply[NonCommutativeMultiply[a, b], c], StringJoin[StringJoin[\"a\", \"b\"], \"c\"], a::b::c, ReplaceAll[ReplaceAll[a, r], s], ReplaceRepeated[ReplaceRepeated[a, r], s], Optional[Pattern[a, b], c], Alternatives[Alternatives[a, b], c], PatternTest[PatternTest[a, f], g], StringExpression[StringExpression[a, b], c], g[f[x]], RightComposition[RightComposition[f, g], h]]]"
```

The message-name renderer prints nested two-argument forms as `a::b::c`; the
exact-two-argument implementation cited above establishes the structure.

Attributes:

```text
$ ./target/release/rum-cli --eval '{Attributes[Plus], Attributes[Times], Attributes[Dot], Attributes[NonCommutativeMultiply], Attributes[StringJoin], Attributes[Alternatives], Attributes[StringExpression], Attributes[RightComposition]}' < /dev/null
List[List[Listable, Orderless, Flat, OneIdentity, Protected], List[Listable, Orderless, Flat, OneIdentity, Protected], List[Protected], List[], List[Protected], List[Protected], List[Protected], List[Flat, OneIdentity, Protected]]
```

Variadic valid-domain behavior:

```text
$ ./target/release/rum-cli --eval '{Dot[{{1,2},{3,4}},{{1,0},{0,1}},{{2},{3}}], StringJoin["a","b","c"], (f/*g/*h)[x], MatchQ[2,1|2|3], StringMatchQ["abc","a"~~"b"~~"c"]}' < /dev/null
List[List[List[8], List[18]], "abc", h[g[f[x]]], True, True]
```

Flat-head match normalization versus structural identity:

```text
$ ./target/release/rum-cli --eval '{MatchQ[HoldComplete[a+b+c], Verbatim[HoldComplete[Plus[Plus[a,b],c]]]], MatchQ[HoldComplete[a+b+c], Verbatim[HoldComplete[Plus[a,b,c]]]], SameQ[HoldComplete[a+b+c], HoldComplete[Plus[Plus[a,b],c]]], SameQ[HoldComplete[a+b+c], HoldComplete[Plus[a,b,c]]]}' < /dev/null
List[True, True, True, False]
```

Non-`Flat` held-shape matching:

```text
$ ./target/release/rum-cli --eval '{MatchQ[HoldComplete[a.b.c], Verbatim[HoldComplete[Dot[Dot[a,b],c]]]], MatchQ[HoldComplete[a.b.c], Verbatim[HoldComplete[Dot[a,b,c]]]], MatchQ[HoldComplete["a"<>"b"<>"c"], Verbatim[HoldComplete[StringJoin[StringJoin["a","b"],"c"]]]], MatchQ[HoldComplete["a"<>"b"<>"c"], Verbatim[HoldComplete[StringJoin["a","b","c"]]]], MatchQ[HoldComplete[a|b|c], Verbatim[HoldComplete[Alternatives[Alternatives[a,b],c]]]], MatchQ[HoldComplete[a|b|c], Verbatim[HoldComplete[Alternatives[a,b,c]]]], MatchQ[HoldComplete[a~~b~~c], Verbatim[HoldComplete[StringExpression[StringExpression[a,b],c]]]], MatchQ[HoldComplete[a~~b~~c], Verbatim[HoldComplete[StringExpression[a,b,c]]]]}' < /dev/null
List[True, False, True, False, True, False, True, False]
```

Unsupported/inert residual shape:

```text
$ ./target/release/rum-cli --eval 'FullForm[{Dot[Dot[x,y],z], Dot[x,y,z], StringJoin[StringJoin["a",x],"b"], StringJoin["a",x,"b"]}]' < /dev/null
"List[Dot[Dot[x, y], z], Dot[x, y, z], StringJoin[StringJoin[\"a\", x], \"b\"], StringJoin[\"a\", x, \"b\"]]"
```

Binary-preserving forms:

```text
$ ./target/release/rum-cli --eval 'FullForm[HoldComplete[{a::b::c,a/.r/.s,a//.r//.s,a:b:c,a?f?g,x//f//g}]]' < /dev/null
"HoldComplete[List[a::b::c, ReplaceAll[ReplaceAll[a, r], s], ReplaceRepeated[ReplaceRepeated[a, r], s], Optional[Pattern[a, b], c], PatternTest[PatternTest[a, f], g], g[f[x]]]]"
```

The two bounded formatter probes discarded the enormous formatted stdout and
reported only process status:

```text
$ scripts/run-bounded.py --timeout 30 -- python3 -c 'import subprocess; source="+".join(f"x{i}" for i in range(20001)); result=subprocess.run(["./target/release/rum-cli", "--format", source], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True); print(f"exit={result.returncode}"); print(result.stderr, end="")'
exit=0

$ scripts/run-bounded.py --timeout 30 -- python3 -c 'import subprocess; source="+".join(f"x{i}" for i in range(100001)); result=subprocess.run(["./target/release/rum-cli", "--format", source], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True); print(f"exit={result.returncode}"); print(result.stderr, end="")'
exit=-6

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```

The same 20,000-link formatter probe with a 2 MiB process stack did not pass:

```text
$ scripts/run-bounded.py --timeout 30 -- zsh -c 'ulimit -s 2048; python3 -c '\''import subprocess; source="+".join(f"x{i}" for i in range(20001)); result=subprocess.run(["./target/release/rum-cli", "--format", source], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True); print(f"exit={result.returncode}"); print(result.stderr, end="")'\'''
exit=-6

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```

A declared six-process 2 MiB bracket located a conservative policy region:

```text
$ scripts/run-bounded.py --timeout 30 -- zsh -c 'ulimit -s 2048; python3 -c '\''import subprocess
for links in (1000, 2000, 4000, 8000, 12000, 16000):
 source="+".join(f"x{i}" for i in range(links+1))
 result=subprocess.run(["./target/release/rum-cli", "--format", source], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
 first=result.stderr.strip().splitlines()[0] if result.stderr.strip() else ""
 print(f"links={links} exit={result.returncode} stderr={first!r}")'\'''
links=1000 exit=0 stderr=''
links=2000 exit=0 stderr=''
links=4000 exit=0 stderr=''
links=8000 exit=0 stderr=''
links=12000 exit=-6 stderr="thread 'main' has overflowed its stack"
links=16000 exit=-6 stderr="thread 'main' has overflowed its stack"
```

The final declared two-process validation checked the proposed boundary and its
future rejection point before any cap exists:

```text
$ scripts/run-bounded.py --timeout 30 -- zsh -c 'ulimit -s 2048; python3 -c '\''import subprocess
for links in (4096, 4097):
 source="+".join(f"x{i}" for i in range(links+1))
 result=subprocess.run(["./target/release/rum-cli", "--format", source], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
 first=result.stderr.strip().splitlines()[0] if result.stderr.strip() else ""
 print(f"links={links} exit={result.returncode} stderr={first!r}")'\'''
links=4096 exit=0 stderr=''
links=4097 exit=0 stderr=''
```

## 6. Draft follow-up implementation briefs

These drafts intentionally follow the five plan steps. Each is independently
gateable after its predecessors, and each leaves the tree uncommitted.

### Draft brief 330A — iterative comparison-chain lowering and binary module split

#### Goal

Move binary-sugar lowering into a coherent submodule and replace recursive
comparison collection with an explicit in-order work stack. Uniform and mixed
comparison chains must lower exactly as before, while comparison-chain length no
longer consumes native stack in the collector.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/lowering-canonical-forms.md`, and the reviewed Plan 330.
- Entry recursion and binary dispatch:
  `crates/rum-runtime/src/runtime/lowering.rs:9-40,273-446`.
- Current comparison output and recursive collector:
  `crates/rum-runtime/src/runtime/lowering.rs:487-568`.
- Existing comparison goldens:
  `crates/rum-runtime/src/runtime/tests/lowering_golden.rs:42-48`.
- File-size split rule: `docs/code-organization-guidelines.md:64-81`.
- Acceptance is output identity plus bounded depth evidence, not another
  engine's behavior.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/lowering.rs` — module declaration/import and
    removal of the moved binary block only;
  - `crates/rum-runtime/src/runtime/tests/mod.rs` — one test-module declaration;
  - `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` — comparison
    identity cases only.
- Files you MAY create:
  - `crates/rum-runtime/src/runtime/lowering/binary.rs`;
  - `crates/rum-runtime/src/runtime/lowering/binary/comparison.rs`;
  - `crates/rum-runtime/src/runtime/tests/comparison_chain_lowering.rs`.
- Files you MUST NOT touch: all other Rust, all docs, scripts, Cargo files,
  `.git`, and git config.
- The move must not change visibility outside `runtime` or alter any public type.

#### Deliverable

1. Binary-sugar lowering moved behind `lowering/binary.rs`, with comparison
   traversal/output isolated in `lowering/binary/comparison.rs`.
2. An iterative in-order comparison collector with no self-recursive call.
3. Byte-identical two-argument, homogeneous n-ary, and mixed `And` output.
4. Focused tests for all eight comparison operators, mixed order, comparison
   subtrees on lhs/rhs, operand source order, and a deep chain.
5. A split assessment showing the resulting sizes of `lowering.rs`,
   `lowering/binary.rs`, and `lowering/binary/comparison.rs`.

#### rum / Rust pitfalls (read twice)

1. Preserve the current traversal of comparison nodes on both sides; a lhs-only
   walk is not equivalent to `collect_comparison_chain`.
2. For mixed comparisons, the middle operand remains shared into adjacent
   two-argument calls in the same order
   (`crates/rum-runtime/src/runtime/lowering.rs:537-552`).
3. Lower each operand exactly once and in source order.
4. Do not change comparison attributes, evaluator logic, or rendering.
5. Use safe Rust only; an explicit enum/tuple work stack is sufficient.
6. This is an output-preserving input-path change. Do not add profiling or
   touch `Expr`/`SharedExpr` implementation; if scope reaches an evaluation hot
   path, stop and require the normal rum-vs-rum A/B gate.

#### Verification (MANDATORY — run and report verbatim)

Use `scripts/run-bounded.py` for Cargo and release probes, with the timeout
recorded from the most recent same-shape clean run and at least enough headroom
for the baseline build:

```text
cargo fmt --check
cargo test -p rum-runtime comparison_chain_lowering
cargo test -p rum-runtime lowering_golden
cargo test -p rum-runtime
cargo test --workspace
git diff --check
```

Build `rum-cli --release`, generate `.rum` sources under `/tmp`, and run uniform
and mixed comparison chains through `Head[Hold[...]]` with stdin from
`/dev/null`. Include one case beyond the current collector's measured failure
size. Paste commands, output, exit codes, input link counts, and timeout
headroom. An exit `-6`/134 is failure. If another recursive consumer now fails,
stop and identify it with `path:line`; do not claim comparison collection is the
remaining frame.

No microbench A/B is required unless the implementation exceeds this input-only
scope; explain that in the report.

#### Hard rules

- Do not commit, branch, push, or touch git config.
- Modify only Scope files; no network; touch nothing outside the repo except
  explicitly named `/tmp/*.rum` probes.
- Tests assert rum's documented comparison lowering only. No external oracle,
  differential harness, compatibility table, or third-party output.
- No `unsafe`, no enlarged-stack test helper, no recursive replacement helper.
- The module move must be mechanical outside the comparison collector.
- Preserve all existing golden strings byte-for-byte.
- Use bounded commands and paste real output; never hide a nonzero exit.

#### Report format

1. Files moved/changed and resulting character counts.
2. Work-stack invariant and proof of in-order identity, with `path:line`.
3. Focused/workspace test output verbatim.
4. Long uniform/mixed probe commands, link counts, output, exits, and bounds.
5. Confirmation that comparison goldens are unchanged.
6. `git status --short`, showing only Scope files.
7. Stop-loss/open questions.

### Draft brief 330B — iterative shape-preserving left folds for all sixteen

#### Goal

Make all sixteen measured left-associative `BinaryOp` lhs spines lower through
one iterative collector/fold while preserving every current short-chain `Expr`
shape and resolver call count/order. This step removes the recursive defect
before later briefs make any approved canonical-shape change.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, the reviewed Plan 330, and the
  final 330A report.
- Binary dispatcher: `crates/rum-runtime/src/runtime/lowering/binary.rs` after
  330A (pre-split source is
  `crates/rum-runtime/src/runtime/lowering.rs:273-446`).
- Exact consumer contracts:
  `crates/rum-runtime/src/runtime/messages/builtins_messages.rs:31-47`,
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/all.rs:4-54`,
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:4-17`,
  and `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:162-214`.
- Existing postfix-application golden:
  `crates/rum-runtime/src/runtime/tests/lowering_golden.rs:22-23`.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/lowering/binary.rs`;
  - `crates/rum-runtime/src/runtime/tests/mod.rs`;
  - `crates/rum-runtime/src/runtime/tests/lowering_golden.rs` — add
    shape-preservation cases only.
- Files you MAY create:
  - `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`;
  - `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`.
- Files you MUST NOT touch: parser/syntax crate, evaluator/matcher/message/rewrite
  implementations, docs, scripts, Cargo files, rum-core, `.git`, and git config.

#### Deliverable

1. A borrowed left-spine link representation and iterative collector that walks
   across any of the sixteen affected operators, including family transitions.
2. One source-order `Expr` fold dispatching the current binary transformation
   for every link; no shape flattening in this brief.
3. Raw message-tag handling, including the current conditional second lowering
   of a non-symbol/string rhs, and `Pattern`/`Optional` selection preserved.
4. Tests for exact small shapes, resolver visitation count/order, direct-call
   opacity, same-family mixes, a synthetic cross-family direct AST, and deep
   homogeneous/mixed chains.
5. Bounded 20,000-link release evidence for all sixteen tokens before the later
   resource-policy brief introduces a 4,096-link ceiling.
6. Character counts for all touched lowering modules and a split assessment
   against `docs/code-organization-guidelines.md`.

#### rum / Rust pitfalls (read twice)

1. The vector is collected outer-to-inner and must be reversed before folding.
   Continue across all sixteen affected ops; a same-token predicate is not
   sufficient for adversarial direct AST input.
2. The first lowered value is the innermost lhs; then process rhs operands in
   source order. Do not lower an rhs during collection.
3. `MessageName` needs both its initially lowered rhs and raw syntax for tag
   conversion. Current code conditionally lowers a non-symbol/string rhs again
   when the accumulated lhs is not a symbol. Preserve that odd call count/order;
   cleanup is outside this output-preserving brief.
4. `a:b:c` is not one n-ary pattern: the second link observes a non-symbol lhs
   and becomes `Optional[Pattern[a,b],c]`.
5. `x//f//g` must remain `g[f[x]]`; there is no canonical `PostfixApply` head.
6. Do not infer n-ary semantics for any head in this step. In particular, do
   not infer them for `NonCommutativeMultiply` from its name.
7. Do not add a cap in this brief; Step 330E owns one shared limit and error.

#### Verification (MANDATORY — run and report verbatim)

```text
cargo fmt --check
cargo test -p rum-runtime left_chain_lowering
cargo test -p rum-runtime lowering_golden
cargo test -p rum-runtime
cargo test --workspace
git diff --check
```

Run each through the bounded runner with a data-derived timeout. With a fresh
release CLI, generate sixteen 20,000-link `.rum` probes under `/tmp`, each shaped
as `Head[Hold[chain]]`, and feed stdin from `/dev/null`. Add one alternating
`/.`/`//.` probe and one short full-form table. Paste all outputs and exit codes;
every long result must be `Hold` and exit 0. State an eighteen-process release
probe cap before running.

No A/B is required for this lowering-only call-site change. If any shared
constructor implementation or evaluation path changes, stop and add the
required core-language base/candidate gate.

#### Hard rules

- Do not commit, branch, push, or touch git config.
- Modify only Scope files; `/tmp` probe files use the `.rum` suffix; no network.
- No external engine, oracle, compatibility claim, or third-party output.
- Safe Rust only. No recursive collector/fold and no large-stack test wrapper.
- Short-chain output and resolver call count/order are immutable in this brief.
- Do not flatten any of the sixteen heads or add attributes.
- Use bounded commands and paste real output/exit codes.

#### Report format

1. All-sixteen collector/fold implementation and target predicate, with
   `path:line`.
2. Exact before/after small-shape table for all sixteen (all rows unchanged).
3. Resolver count/order, mixed-rewrite, and cross-family direct-AST results.
4. Cargo verification output verbatim.
5. Eighteen release probe outputs/exits and timeout headroom.
6. Scope-only `git status --short`.
7. Stop-loss/open questions.

### Draft brief 330C — flatten additive, multiplicative, and right-composition chains

#### Goal

Use 330B's iterative spine infrastructure to lower `+`/`-` chains into one
`Plus`, `*`/`/` chains into one `Times`, and `/*` chains into one
`RightComposition`, without absorbing direct canonical calls or changing exact
literal-division folding.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/lowering-canonical-forms.md`, the reviewed Plan 330, and final 330A/B
  reports.
- Current arithmetic lowering and literal quotient:
  pre-split `crates/rum-runtime/src/runtime/lowering.rs:143-163,387-424`.
- `Flat` authorities:
  `crates/rum-stdlib/src/catalog/arithmetic.rs:13-32` and
  `crates/rum-stdlib/src/catalog/functional.rs:380-387`.
- Direct-call preservation contract:
  `docs/lowering-canonical-forms.md:22-30,52-60`.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/lowering/binary.rs`;
  - `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`;
  - `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`;
  - `crates/rum-runtime/src/runtime/tests/lowering_golden.rs`;
  - `docs/lowering-canonical-forms.md`;
  - `docs/guide/syntax.md`.
- Files you MUST NOT touch: parser/syntax implementation, rum-core, arithmetic
  evaluator, functional evaluator, catalogs/attributes, generated reference,
  scripts, Cargo files, other docs, `.git`, and git config.

#### Deliverable

1. One flat additive chain builder handling mixed `Add`/`Subtract`.
2. One flat multiplicative builder handling mixed `Times`/`Divide` and retaining
   the exact-literal quotient behavior.
3. One flat `RightComposition` builder.
4. Direct `Plus[...]`, `Times[...]`, and `RightComposition[...]` bases kept
   opaque to syntax-chain flattening.
5. Updated canonical-form docs and goldens for homogeneous/mixed chains.
6. Tests for held structural change, ordinary evaluated equivalence, numeric
   edge cases, source-order resolution, and 20,000-link safety.
7. Character counts and a split assessment for touched lowering modules.

#### rum / Rust pitfalls (read twice)

1. Do not call runtime `flatten_flat_call_args` on the final lowered tree; it
   cannot distinguish syntax-chain-owned nodes from direct canonical calls.
2. `a+b-c+d` is one family. A same-token-only collector reintroduces recursion
   on alternating links.
3. Preserve `8/2/4` and related exact-literal results by tracking whether the
   whole current accumulator is still one numeric atom.
4. Binary `/` still emits `Power[rhs,-1]` only when literal division did not
   succeed.
5. Flattening changes held `SameQ` and full-form results. Document and pin it;
   do not call it output-preserving.
6. Do not change `Plus`, `Times`, or `RightComposition` attributes/evaluators.
7. Do not add the chain ceiling here; 330E owns it.

#### Verification (MANDATORY — run and report verbatim)

```text
cargo fmt --check
cargo test -p rum-runtime left_chain_lowering
cargo test -p rum-runtime lowering_golden
cargo test -p rum-runtime arithmetic
cargo test -p rum-stdlib functional_ops
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
git diff --check
```

Use bounded commands. Run a fresh release CLI on five 20,000-link
`Head[Hold[...]]` `.rum` probes (`+`, `-`, `*`, `/`, `/*`) plus a short full-form
and numeric table; stdin is `/dev/null`. Paste output and exits. Inspect generated
builtin-reference output or run its drift test and confirm no reference diff.

This changes the number and shape of `Expr::call` allocations during lowering,
not the evaluator or constructor implementation. No core-language timing A/B is
required under that exact scope; if implementation touches `Expr`/`SharedArgs`
or evaluation dispatch, stop and apply the AGENTS.md hot-path A/B rule.

#### Hard rules

- Do not commit, branch, push, or touch git config.
- Modify only Scope files; no network; only `.rum` probes under `/tmp`.
- Rum's documented semantics only; no external oracle or compatibility claim.
- Safe Rust only; no recursive chain walk or rebuild.
- Preserve direct-call opacity and literal-division behavior exactly.
- Do not hand-edit generated reference or change any attribute.
- Documentation and golden changes land with the shape change.
- Paste real bounded verification output and nonzero exits.

#### Report format

1. Per-family builder and ownership/provenance invariant, with `path:line`.
2. Old/new lowering table for every new golden.
3. Numeric-literal and direct-call-opacity results.
4. Cargo verification output verbatim.
5. Release probe outputs/exits and bounds.
6. Documentation diff summary and confirmation of zero reference drift.
7. Scope-only `git status --short`.
8. Stop-loss/open questions.

### Draft brief 330D — flatten variadic Dot, StringJoin, Alternatives, and StringExpression chains

#### Goal

After the director settles Dot's shape, use the iterative spine infrastructure
to emit one n-ary `Dot`, `StringJoin`, `Alternatives`, or `StringExpression` for
each source chain. Preserve documented valid-domain results and direct canonical
calls, while explicitly documenting and testing the held/inert shape changes.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, the reviewed Plan 330, the Dot
  ruling, and final 330A–C reports.
- N-ary consumers:
  `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:31-53`,
  `crates/rum-stdlib/src/strings/text/builtins.rs:9-36`,
  `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:63-65,246-251`, and
  `crates/rum-stdlib/src/strings/patterns.rs:428-446`.
- Existing Dot golden:
  `crates/rum-runtime/src/runtime/tests/lowering_golden.rs:29`.
- These heads are not attributed `Flat`:
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_a_e.rs:18,131`
  and
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes/specs_s_z.rs:36-40`.

#### Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/lowering/binary.rs`;
  - `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`;
  - `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`;
  - `crates/rum-runtime/src/runtime/tests/lowering_golden.rs`;
  - `docs/lowering-canonical-forms.md`;
  - `docs/guide/syntax.md`;
  - `docs/guide/patterns-and-rules.md` — only if needed to state the n-ary
    Alternatives canonical form;
  - `docs/guide/string-patterns.md` — only if needed to state the n-ary
    StringExpression canonical form.
- Files you MUST NOT touch: stdlib/runtime evaluator and matcher
  implementations, attribute catalogs, generated reference, parser, rum-core,
  scripts, Cargo files, other docs, `.git`, and git config.

#### Deliverable

1. N-ary chain builders for the four named tokens, using only chain-owned links.
2. Updated Dot golden according to the director's ruling.
3. Canonical-form docs for each source chain.
4. Valid tensor/string/pattern/string-pattern evaluation tests unchanged in
   result.
5. Explicit held `SameQ`/`Verbatim` and unsupported symbolic residual-shape
   tests showing the intentional change.
6. Direct canonical nested calls remain untouched.
7. Four bounded 20,000-link release probes.
8. Character counts and a split assessment for touched lowering modules.

#### rum / Rust pitfalls (read twice)

1. Do not add `Flat` attributes. This brief changes syntax lowering only.
2. `Dot[Dot[x,y],z]` and `Dot[x,y,z]` remain distinct for unsupported symbolic
   operands today; update the contract knowingly.
3. Invalid `StringJoin` input similarly returns its original call shape.
4. Typical source patterns lower on both value/pattern sides, but explicit held
   canonical patterns can distinguish the old/new tree. Keep a `Verbatim` test.
5. Preserve order: Dot and string sequences are not orderless.
6. Do not absorb a direct canonical call used as the source chain's base.
7. If the Dot ruling is “nested,” implement only the other three here and add
   Dot to 330B's iterative nested family; do not leave `.` recursive.

#### Verification (MANDATORY — run and report verbatim)

```text
cargo fmt --check
cargo test -p rum-runtime left_chain_lowering
cargo test -p rum-runtime lowering_golden
cargo test -p rum-runtime definitions_and_rules
cargo test -p rum-stdlib strings
cargo test -p rum-stdlib functional_ops
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
git diff --check
```

Run all commands bounded. With a fresh release CLI, run four 20,000-link
`Head[Hold[...]]` `.rum` probes and the exact short valid/held/inert table from
Plan 330, stdin from `/dev/null`. Paste every output and exit. Confirm generated
reference has no diff. No A/B is required unless evaluator/matcher/constructor
implementation leaves the stated scope.

#### Hard rules

- Do not dispatch until the Dot shape ruling is written into the final brief.
- Do not commit, branch, push, or touch git config.
- Modify only Scope files; no network; `/tmp` sources end in `.rum`.
- No external oracle, differential testing, compatibility claim, or third-party
  output.
- Safe Rust only and no recursive chain collection/rebuild.
- No attribute or evaluator changes.
- Treat held and inert residual shape as observable contract, with docs/tests in
  the same change.
- Paste real bounded outputs and exits.

#### Report format

1. Director ruling and final per-token decision.
2. Old/new canonical and held/inert shapes.
3. Valid-domain behavior table.
4. Cargo/reference verification output verbatim.
5. Four long release probes with exits/bounds.
6. Documentation/golden diff summary.
7. Scope-only `git status --short`.
8. Stop-loss/open questions.

### Draft brief 330E — enforce the 4,096-link left-chain ceiling

#### Goal

Add one fixed, documented 4,096-link limit for left-associative source chains
and direct-AST lowering. Exactly 4,096 links must parse/lower/format on the
supported stack configurations; link 4,097 and the original 20,000-link crash
cases must fail with a dedicated, non-incomplete syntax diagnostic instead of
allocating without bound or aborting.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, the reviewed Plan 330, and final
  330A–D reports.
- Parser counters and precedence loop:
  `crates/rum-syntax/src/parser/front.rs:4-27,115-130,203-325`.
- Syntax error classifications:
  `crates/rum-syntax/src/error.rs:4-80`.
- Formatter consumes `ParsedSource.syntax`:
  `crates/rum-syntax/src/formatter/mod.rs:41-59` and
  `crates/rum-syntax/src/document.rs:43-48,326-358`.
- Existing resource-limit docs/table:
  `docs/guide/resource-limits-and-sandbox.md:111-140` and
  `docs/dev/publication-audit/capability-consistency-findings.md:714-716`.
- Plan evidence: the existing formatter passed 8,000 and failed at 12,000 links
  on a 2 MiB process stack; exact 4,096 and 4,097 both passed before the cap.

#### Scope

- Files you MAY modify:
  - `crates/rum-syntax/src/lib.rs` — export the single shared constant only;
  - `crates/rum-syntax/src/parser/front.rs`;
  - `crates/rum-syntax/src/error.rs`;
  - `crates/rum-syntax/src/parser/tests/mod.rs`;
  - `crates/rum-syntax/src/formatter/tests.rs`;
  - `crates/rum-runtime/src/runtime/lowering/binary/left_chain.rs`;
  - `crates/rum-runtime/src/runtime/lowering/binary/comparison.rs`;
  - `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`;
  - `docs/guide/resource-limits-and-sandbox.md`;
  - `docs/dev/publication-audit/capability-consistency-findings.md`.
- File you MAY create:
  - `crates/rum-syntax/src/parser/tests/left_chain_limits.rs`.
- Files you MUST NOT touch: AST representation/Drop, evaluator, rum-core,
  catalogs/generated reference, scripts, Cargo files, other docs, `.git`, and
  git config.

#### Deliverable

1. One shared `MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS: usize = 4_096` constant.
2. Parser counting for explicit left-associative operators and implicit `Times`,
   scoped to one owned lhs spine.
3. `SyntaxError::LeftAssociativeChainTooLong { max, span }`, rendered exactly as
   `left-associative chain exceeded limit {max} at bytes {span}`, included in
   `span()`, and excluded from `is_incomplete()`.
4. The same bound in the all-sixteen runtime lhs collector and comparison
   collector for direct AST input.
5. Exact-limit/one-past, mixed-family, implicit-times, independent-spine,
   diagnostic, direct-AST, formatter, and script-splitting tests.
6. Updated guide and publication-audit limit rows.
7. Ordinary-worker, 2 MiB-thread, and release-process proof that the accepted
   boundary and rejected boundary do not overflow.

#### rum / Rust pitfalls (read twice)

1. Count **links**, not operands: 4,096 operators means 4,097 operands.
2. Increment for implicit multiplication as well as explicit tokens.
3. Do not count right-associative links here; their native-recursion guard stays
   `MAX_OPERATOR_CHAIN_DEPTH = 128`.
4. Independent rhs/nested spines must not accumulate into one counter.
5. Comparison lowering can traverse comparison subtrees on both sides; enforce
   its total collected operator count even when parser-local spine counters are
   individually below the limit.
6. The new error is terminal, not incomplete. Otherwise script-form splitting
   can swallow later lines, as the existing statement-cap regression explains
   (`crates/rum-runtime/src/runtime/tests/lowering_and_context.rs:148-171`).
7. A cap whose accepted or rejected boundary overflows during recursive
   `SyntaxExpr` drop/formatting is not a fix. Stop on that evidence.
8. Do not introduce a host/session setting: parsing precedes session evaluation,
   and this brief specifies one language-source ceiling.
9. Capture the offending explicit operator token span before `bump()`; implicit
   multiplication uses the first token span of the operand that would exceed
   the limit. Direct-AST enforcement uses the rejected binary node's span.

#### Verification (MANDATORY — run and report verbatim)

```text
cargo fmt --check
cargo test -p rum-syntax left_chain
cargo test -p rum-syntax formatter
cargo test -p rum-runtime left_chain_lowering
cargo test -p rum-runtime lowering_and_context
cargo test --workspace
git diff --check
```

Run with bounded commands and data-derived timeouts. Build a fresh release CLI
and probe, with stdin `/dev/null`:

- each of the sixteen homogeneous operators at exactly 4,096 links through
  `Head[Hold[...]]` (exit 0, output `Hold`);
- representative mixed additive, multiplicative, rewrite, and comparison chains
  at the limit;
- 4,097 links and 20,000 links (exit 1, the same dedicated diagnostic, no
  abort);
- `--format` at 4,096 (exit 0) and 4,097 (same syntax diagnostic);
- the accepted and rejected cases inside a dedicated 2 MiB-stack test process.

Cap the release probe panel at exactly 24 processes and state that cap before
running: sixteen homogeneous, four mixed, two evaluation rejections, and two
formatter boundaries.
Paste all output/exit rows, the exact error text/span, and timeout headroom. An
exit `-6`/134 or a missing later script form is failure. No timing A/B is
required; no evaluation hot path is touched.

#### Hard rules

- Do not commit, branch, push, or touch git config.
- Modify only Scope files; no network; all `/tmp` rum sources use `.rum`.
- Rum-owned tests only; no other engine/oracle, compatibility matrix, or
  third-party output.
- Safe Rust only. Do not change `SyntaxExpr` representation/Drop or use a
  big-stack wrapper to hide a failing boundary.
- The constant is exactly 4,096 unless the director issues a new ruling after
  a stop-loss report.
- Preserve the existing three parser limits and their messages.
- Documentation and publication-audit rows must match final names/lines.
- Paste real bounded verification output and every nonzero exit.

#### Report format

1. Constant, counting scope, and diagnostic text with `path:line`.
2. Boundary/mixed/implicit/independent-spine test table.
3. `is_incomplete`, span/caret, and script-splitting results.
4. Ordinary-worker and 2 MiB-stack results.
5. Sixteen-operator and formatter release probe table with exits/bounds.
6. Cargo/workspace output verbatim.
7. Docs/audit diff summary.
8. Scope-only `git status --short`.
9. Stop-loss/open questions.
