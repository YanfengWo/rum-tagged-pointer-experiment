# Borrowed single binding for synchronous rule RHS substitution

This is an isolated safe Rust experiment. `candidate.patch` applies **after**
the primary `rewrite-shortcuts/matcher.patch` on pinned rum
`77029938f999b14dbe5b0755bebdf98e51517ae6`. It changes two source files in a
disposable copy; it does not change the default checkout or global
`SymbolExprMap`. Build and correctness records are documented in the README;
performance remains unmeasured for this candidate.

## Representation and entry boundary

`RuleBindingView` is a Copy, borrowed RHS lookup with two cases:
`Map(&SymbolExprMap)` and `One { name: Symbol, value: &Expr }`. It provides
only `get`, `contains_key`, and `is_empty`. The One lookup compares the full
Symbol identity using the existing integer equality, including generated
identity; it does not compare text or alter typed-blank head semantics.

The central scanner admits One only when a compiled matcher is exactly
Named/Blank and the rule's guards are empty. It calls the existing
`match_blank` predicate on the same normalized expression. It begins at the
same fresh-candidate boundary where the original scanner supplied an empty
binding map. The primary unnamed Blank shortcut continues to use its old
helper. All complex, guarded, or other patterns retain their original full
match result, error handling, cursor and suspension behavior.

For a successful Named/Blank with initial empty bindings, the original
binding map contains precisely `{name -> normalized_expr.clone()}`. One's
lookup returns the same expression at that symbol, returns None for every
other symbol, and is not empty. Therefore its three supported operations
agree with that singleton map. A predicate miss still joins the original
profile record and cursor continuation before any RHS substitution.

The central scanner retains the borrowed normalized expression through the
entire synchronous substitution call. It never stores a view in
`CompiledMatchOutcome`, a frame, a Session field, or a continuation. The
complex branch instead owns its original map until the common RHS exit
finishes. All output values are owned Expr values cloned or rebuilt using
the existing algorithm; no output expression contains a Rust borrow.

## Reuse of the original substitution algorithm

The two existing map entry points keep their names and signatures. Their
wrappers create Map views. One internal RHS-view entry and one private
top-level view helper share the original profile and empty-binding logic.
The binding parameter changes in eight private helper functions; their
walking, reconstruction, splicing and hygiene bodies remain the original
ones. Alpha-renaming environments continue to use ordinary SymbolExprMap.
There is no separate simplified RHS walker.

At a symbol, pattern bindings retain priority over alpha-renames. A bound
value is cloned directly into the result and is not recursively traversed
or alpha-renamed. Module, With, and explicit Function parameter processing
still use `alpha_renames_with_fresh_locals`; if a local is itself bound by
the pattern, the existing `contains_key` rule skips a new alpha name. With
initializer ordering and its evolving alpha environment remain unchanged.
This argument preserves rum's existing behavior, including its same-name
local-binding rules, rather than substituting a different hygiene model.

The same call walker still splices a Sequence only when it is substituted
for a directly bound symbol argument. A literal Sequence Call stays one
argument in its parent, even if its own arguments were reconstructed.
The same rebuild helper preserves unchanged siblings and SharedArgs /
SharedExpr nodes; output mutation continues to use their existing COW.
The retained capacity of output vectors is governed by the original
builders, not a new single-binding builder.

## Observable ordering and checks

The common successful exit preserves the original rule-applied trace and
returns the substituted expression without evaluating it early. Rule
priority, target evaluation, rule construction, RHS guard evaluation and
final traversal evaluation are not reordered. The One branch cannot call
user code or suspend; a guarded rule remains on the full continuation
path and may later continue to an unguarded One fallback candidate.

The original Named/Blank matcher branches and RHS substitution helper do
not themselves call evaluate, enforce a budget, or increment the orderless
match-depth guard. No explicit resource, sandbox, or cancellation check
is deleted. The outer evaluator's existing checks remain in place. This
does not prove identical outcomes at every timing, memory, or native-stack
threshold: allocation history, temporary clone counts, and execution time
are deliberately different. The proof assumes the supported entry shape,
empty initial bindings, retained original normalization, and no future
user-code callback inserted into this synchronous walker.

## Regression source and architecture assessment

`regression-tests.rs` contains six tests, using the real scanner instead
of calling a candidate-only helper. The same source is suitable for the
unchanged base, primary matcher, and this candidate. It checks distinct
generated identities with the same text; actual fresh local identities in
Module, With and Function; binding priority over local renaming; inserted
bound values outside alpha-renaming; direct versus literal Sequence;
shared child identities and mutation through COW; and queued input versus
two real suspensions with a declining guard and a narrow fallback rule.
The initial primary matcher regressions and full workspace checks remain
required. These are semantic tests, not timing workloads.

The touched substitution module was already about 27 KB and remains in
the 20-35 KB split-plan range. The scanner module crosses roughly 20 KB.
For this temporary experiment, keeping the exact existing walker together
makes the semantic change auditable. Before any default integration,
assess extracting the RHS lookup/entry boundary and scanner application
stage into coherent modules; do not mechanically split the hygiene code.
No claim of default adoption, full soundness proof, or speedup is made.
