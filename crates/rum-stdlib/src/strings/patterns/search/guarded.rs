use super::super::matcher::pattern_match_text;
use super::*;

mod cases;
mod replace;

pub(super) use cases::{
    collect_string_cases_values_with_guarded_rules_expr, collect_string_cases_with_guarded_rules,
};
pub(super) use replace::{
    replace_string_value_with_guarded_rules, replace_string_values_with_guarded_rules_expr,
};

fn evaluate_string_rule_guard_result(
    session: &mut Session,
    guard: &StringRuleGuard,
    chars: &[char],
    start: usize,
    end: usize,
    bindings: &SymbolExprMap,
) -> Result<bool, EvalError> {
    let candidate = Expr::string(pattern_match_text(&chars[start..end]));
    evaluate_definition_rhs_guards_for_match(
        session,
        &guard.lhs,
        &candidate,
        &guard.guards,
        bindings,
    )
}
