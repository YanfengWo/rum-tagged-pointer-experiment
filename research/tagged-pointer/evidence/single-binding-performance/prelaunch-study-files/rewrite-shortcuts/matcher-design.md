# Rewrite leaf matcher prototype

This isolated patch changes only `apply_rules_once_from_list` in
`definitions/application/replace_all.rs`. It belongs to a disposable source
copy; the default runtime is unchanged. Do not combine it with the wide-copy
or Flat prototypes for its first comparison.

## Eligibility and existing implementation

After the existing shape prefilter accepts a rule, the candidate reuses
`Session::try_fast_definition_match_compiled` only when both conditions hold:

- `rule.guards` is empty.
- The compiled matcher is `Blank`, or `Named` whose immediate inner matcher
  is `Blank`.

`RewriteRule` does contain `guards: Box<[Expr]>` in `model/definitions.rs`.
Compilation through `compile_down_value_matcher` also wraps nonempty guards in
`MatcherNode::DefinitionCondition`; both representations are excluded. The
existing helper is visible from this sibling module without changing its
visibility. Unsupported matchers go through the original
`match_compiled_owned` call with the original cloned matcher, normalized value,
empty bindings, and suspension handling.

The reused helper creates a fresh binding map, calls the same `match_blank`
predicate, and for `Named` calls the same `bind_pattern_name` function as the
general matcher. It returns a successful or failed `CompiledMatchOutcome`
without a possible host request. For this exact grammar, the general matcher
would allocate binding snapshots for the outer `Named` and inner `Blank`
although the matcher neither evaluates code nor backtracks. The prototype
avoids those snapshots and the compiled matcher clone. It retains the binding
map allocation required for a named success.

## Preserved ordering and boundaries

The existing rule-plan cursor, shape prefilter and profile records stay in the
same order. Both paths join before the original outcome handling. A mismatch
continues to the next rule; a match uses the original RHS substitution, trace,
and return code. RHS evaluation is not moved into matching or advanced in the
Replace traversal. Target evaluation, rule compilation, candidate
normalization and later result evaluation are unchanged.

The fast leaf match reads only the already-normalized candidate. It performs
no definition lookup, capability operation, user predicate or guard. A typed
blank still uses the original `expr_has_head` rules, including their exact
symbol/context behavior. A single named blank starts with empty bindings, so
there is no previous binding to restore and no repeated-name ambiguity.
Composite/repeated bindings, conditions, pattern tests, alternatives, sequence
patterns, optional/default patterns and key-value patterns all use the full
matcher. Their original request frames and binding rollback remain intact.

Do not broaden this patch to arbitrary `MatcherNode::Call`. In the existing
definition fast matcher, that branch can reject an atomic candidate or unequal
arity before considering `OneIdentity`. The full matcher can promote a whole
candidate to a singleton or retry after a head mismatch. Thus a direct reuse
for all compiled matchers requires a separate semantic audit. This prototype
also does not cache attribute eligibility across rule applications; a previous
guard can change attributes before the next rule.

## Required validation

Check ordinary and typed blanks, named blanks binding both atoms and calls,
matching and nonmatching list elements, exact rules mixed with these leaves,
and first-rule precedence. Check guarded RHS decline/fallback, condition and
pattern-test side effects, repeated-name call patterns, `OneIdentity` heads,
sequence splicing and held parents through the original fallback. Existing
queued-input versus actual suspend/resume tests must remain equal, including
guarded replacement and nested RHS replacement. Run the whole workspace and
the unchanged 112-label microbenchmark before considering adoption. Retain
all per-label regressions; the avoided operation count alone is not a speed
result.

The next wider option, only if this narrow experiment is worthwhile, is a
leaf shortcut inside `match_compiled_owned` that accepts incoming bindings and
preserves their failure rollback. It would help guarded outer patterns as
well, but affects every matcher caller and therefore has a larger semantic
and performance surface. It is not included here.
