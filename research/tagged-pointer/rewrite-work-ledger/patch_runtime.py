"""Exact diagnostic instrumentation for a disposable pinned rum archive.

The parent prepares rum_core::rewrite_work_ledger. No production checkout is
edited by this module. Counters describe logical work, not time or allocation
bytes. Lexical scopes cover synchronous calls only; suspended probes must be
reported separately rather than treated as complete operation ledgers.
"""
from pathlib import Path


def instrument(root: Path) -> None:
    runtime = root / "crates/rum-runtime/src/runtime"
    changed: dict[Path, str] = {}

    def replace(relative: str, old: str, new: str) -> None:
        path = runtime / relative
        source = changed.get(path)
        if source is None:
            source = path.read_text(encoding="utf-8")
        count = source.count(old)
        if count != 1:
            raise ValueError(f"expected one diagnostic site in {relative}, got {count}: {old!r}")
        changed[path] = source.replace(old, new, 1)

    ledger = "rum_core::rewrite_work_ledger"
    once = "kernel_data/rewrite_data/replace/once.rs"
    replace(once,
        "    ) -> Result<Expr, EvalError> {\n        let args_slice = args.as_ref();",
        f'    ) -> Result<Expr, EvalError> {{\n        let _ledger_replace = {ledger}::scope("replace");\n        let args_slice = args.as_ref();')
    replace(once,
        "        let target_result = self.evaluate(args_slice[0].clone());",
        f'''        let target_result = {{
            let _ledger_target = {ledger}::scope("target-evaluation");
            self.evaluate(args_slice[0].clone())
        }};''')
    replace(once,
        "        let replaced_result = self.replace_owned(target, rule_plan, levels, 0, include_heads);",
        f'''        let replaced_result = {{
            let _ledger_traversal = {ledger}::scope("traversal");
            self.replace_owned(target, rule_plan, levels, 0, include_heads)
        }};''')
    replace(once,
        "                    let evaluated_result = session.evaluate(outcome.expr);",
        f'''                    let evaluated_result = {{
                        let _ledger_result = {ledger}::scope("result-evaluation");
                        session.evaluate(outcome.expr)
                    }};''')
    replace(once,
        "    ) -> Result<ReplaceOutcome, EvalError> {\n        let depth = if include_heads {",
        f'''    ) -> Result<ReplaceOutcome, EvalError> {{
        {ledger}::event("replace-node", "entry", 1);
        let depth = if include_heads {{''')
    replace(once,
        "        if levels.matches(level, depth) {\n            let replaced_result",
        f'''        if levels.matches(level, depth) {{
            {ledger}::event("replace-level-selected", "entry", 1);
            let replaced_result''')
    replace(once,
        "fn replace_expr_depth(expr: &Expr) -> usize {\n    match expr {",
        f'''fn replace_expr_depth(expr: &Expr) -> usize {{
    {ledger}::event("replace-depth-node", "without-heads", 1);
    match expr {{''')
    replace(once,
        "fn replace_expr_depth_with_heads(expr: &Expr) -> usize {\n    match expr {",
        f'''fn replace_expr_depth_with_heads(expr: &Expr) -> usize {{
    {ledger}::event("replace-depth-node", "with-heads", 1);
    match expr {{''')

    application = "definitions/application/replace_all.rs"
    replace(application,
        "    ) -> Result<Option<Expr>, EvalError> {\n        let normalized_expr = self.normalize_match_expr(&expr);",
        f'''    ) -> Result<Option<Expr>, EvalError> {{
        let _ledger_apply = {ledger}::scope("rule-apply");
        {ledger}::event("rule-apply", "request", 1);
        let normalized_expr = self.normalize_match_expr(&expr);''')
    replace(application,
        "            record_profile_rewrite_rule_prefilter_candidate(would_reject);",
        f'''            {ledger}::event("rule-prefilter", if would_reject {{ "reject" }} else {{ "pass" }}, 1);
            record_profile_rewrite_rule_prefilter_candidate(would_reject);''')
    replace(application,
        "            record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);",
        f'''            {ledger}::event("rule-match", if outcome.matched {{ "hit" }} else {{ "miss" }}, 1);
            record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);''')

    matcher = "pattern_matching/core.rs"
    replace(matcher,
        "    ) -> Result<CompiledMatchOutcome, EvalError> {\n        let snapshot = BindingSnapshot::capture(&bindings, &matcher);",
        f'''    ) -> Result<CompiledMatchOutcome, EvalError> {{
        let _ledger_matcher = {ledger}::scope("matcher");
        {ledger}::event("matcher", "entry", 1);
        let snapshot = BindingSnapshot::capture(&bindings, &matcher);''')
    replace(matcher,
        "        collect_matcher_binding_names(matcher, &mut names);\n        let entries = names",
        f'''        collect_matcher_binding_names(matcher, &mut names);
        {ledger}::event("binding-snapshot", "capture", 1);
        {ledger}::event("binding-snapshot", "names", names.len());
        let entries: Vec<_> = names''')
    replace(matcher,
        "            .collect();\n        Self { entries }",
        f'''            .collect();
        {ledger}::event("binding-snapshot", "entries", entries.len());
        Self {{ entries }}''')

    substitution = "substitution/binding_substitution.rs"
    replace(substitution,
        "    _may_splice_without_bindings: bool,\n) -> Expr {\n    let start = rule_rhs_timing_start();",
        f'''    _may_splice_without_bindings: bool,
) -> Expr {{
    let _ledger_rhs = {ledger}::scope("rhs");
    {ledger}::event("rhs-substitution", "request", 1);
    {ledger}::event("rhs-substitution", "bindings", bindings.len());
    let start = rule_rhs_timing_start();''')

    preparation = "call_preparation.rs"
    replace(preparation,
        "            normalized_args.sort_by(compare_expr_canonical);",
        f'''            let ledger_head = expr_as_symbol(head).map(|symbol| symbol.name()).unwrap_or("<compound>");
            {ledger}::event("orderless-sort", ledger_head, 1);
            {ledger}::event("orderless-sort-input-slots", ledger_head, normalized_args.len());
            normalized_args.sort_by(|left, right| {{
                {ledger}::event("orderless-sort-comparison", ledger_head, 1);
                compare_expr_canonical(left, right)
            }});''')
    replace(preparation,
        "    ) -> SharedArgs {\n        if attribute_normalization_is_noop(head, args.as_ref(), attributes) {\n            return args;\n        }",
        f'''    ) -> SharedArgs {{
        let _ledger_normalization = {ledger}::scope("normalization");
        let ledger_head = expr_as_symbol(head).map(|symbol| symbol.name()).unwrap_or("<compound>");
        {ledger}::event("normalization-request", ledger_head, 1);
        if attribute_normalization_is_noop(head, args.as_ref(), attributes) {{
            {ledger}::event("normalization-reuse", ledger_head, 1);
            {ledger}::event("normalization-reuse-slots", ledger_head, args.len());
            return args;
        }}
        {ledger}::event("normalization-rebuild", ledger_head, 1);
        {ledger}::event("normalization-rebuild-slots", ledger_head, args.len());''')

    helpers = "expr_runtime_helpers.rs"
    replace(helpers,
        "pub(super) fn flatten_flat_call_args(head: &Expr, args: Vec<Expr>) -> Vec<Expr> {",
        f'''pub(super) fn flatten_flat_call_args(head: &Expr, args: Vec<Expr>) -> Vec<Expr> {{
    let ledger_head = expr_as_symbol(head).map(|symbol| symbol.name()).unwrap_or("<compound>");
    {ledger}::event("flatten-call", ledger_head, 1);
    {ledger}::event("flatten-input-slots", ledger_head, args.len());''')
    replace(helpers,
        "            } if nested_head.as_ref() == head => flattened.extend(nested_args),",
        f'''            }} if nested_head.as_ref() == head => {{
                {ledger}::event("flatten-nested-calls", ledger_head, 1);
                {ledger}::event("flatten-nested-slots", ledger_head, nested_args.len());
                flattened.extend(nested_args);
            }}''')
    replace(helpers,
        "    flattened\n}\n\npub fn compare_expr_canonical",
        f'''    {ledger}::event("flatten-output-slots", ledger_head, flattened.len());
    {ledger}::event("flatten-output-capacity-slots", ledger_head, flattened.capacity());
    flattened
}}

pub fn compare_expr_canonical''')

    arithmetic = "arithmetic/canonical.rs"
    replace(arithmetic,
        "    pub(super) fn eval_plus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {\n        if let Some(value) = fast_integer_plus(args) {\n            return Ok(value);\n        }",
        f'''    pub(super) fn eval_plus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {{
        {ledger}::event("plus-builtin", "calls", 1);
        {ledger}::event("plus-builtin", "input-slots", args.len());
        if let Some(value) = fast_integer_plus(args) {{
            {ledger}::event("plus-fast-integer", "success", 1);
            {ledger}::event("plus-fast-integer", "success-input-slots", args.len());
            return Ok(value);
        }}
        {ledger}::event("plus-fast-integer", "fallback", 1);''')

    # Validate every expected site before writing any source file.
    for path, source in changed.items():
        path.write_text(source, encoding="utf-8", newline="\n")
