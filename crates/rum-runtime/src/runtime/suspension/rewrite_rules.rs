use super::*;

fn rewrite_rule_info_expr(rule: RewriteRuleInfo) -> Expr {
    Expr::call(
        system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
        vec![rule.lhs, rule.rhs],
    )
}

fn rewrite_rule_info_from_expr(expr: Expr) -> RewriteRuleInfo {
    match expr {
        Expr::Call { head, mut args }
            if (is_system_symbol_name(&head, "Rule")
                || is_system_symbol_name(&head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let rhs = args.pop().expect("rewrite-rule continuation rhs exists");
            let lhs = args.pop().expect("rewrite-rule continuation lhs exists");
            RewriteRuleInfo {
                lhs,
                rhs,
                delayed: is_system_symbol_name(&head, "RuleDelayed"),
            }
        }
        other => RewriteRuleInfo {
            lhs: other,
            rhs: system_expr("Null"),
            delayed: false,
        },
    }
}

impl ContinuationValue for Option<Vec<RewriteRuleInfo>> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(values) => Expr::call(
                system_expr("InternalContinuationSomeRewriteRuleInfos"),
                vec![Expr::call(
                    system_expr("List"),
                    values.into_iter().map(rewrite_rule_info_expr).collect(),
                )],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, mut args }
                if is_system_symbol_name(&head, "InternalContinuationSomeRewriteRuleInfos") =>
            {
                let Some(list_expr) = args.pop() else {
                    return Some(Vec::new());
                };
                debug_assert!(
                    args.is_empty(),
                    "rewrite-rule continuation payloads use exactly one list slot"
                );

                match list_expr {
                    Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                        Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
                    }
                    other => Some(vec![rewrite_rule_info_from_expr(other)]),
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
            }
            other => Some(vec![rewrite_rule_info_from_expr(other)]),
        }
    }
}
