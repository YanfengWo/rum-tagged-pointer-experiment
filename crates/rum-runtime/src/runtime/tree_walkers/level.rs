use super::*;

impl Session {
    pub(in crate::runtime) fn eval_level(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Level", args, 2, 4));
        }

        let default_include_heads = self.system_option_bool("Level", "Heads", false)?;
        let Some(levels) = parse_level_level_spec(&args[1]) else {
            return Ok(self.invalid_level_spec_call("Level", args, &args[1]));
        };

        let (apply_head, include_heads) = match args {
            [_, _, third] => match classify_heads_option_expr(third) {
                HeadsOptionClassification::Value(include_heads) => {
                    (None, include_heads.unwrap_or(default_include_heads))
                }
                HeadsOptionClassification::NotOptionSyntax => {
                    (Some(third.clone()), default_include_heads)
                }
                classification => {
                    return Ok(self.invalid_heads_option_call(
                        "Level",
                        args,
                        third,
                        2,
                        classification,
                        "expected a Heads -> True/False option or apply head as the third argument",
                    ));
                }
            },
            [_, _, third, fourth] => {
                let classification = classify_heads_option_expr(fourth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Ok(self.invalid_heads_option_call(
                        "Level",
                        args,
                        fourth,
                        3,
                        classification,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };

                (
                    Some(third.clone()),
                    include_heads.unwrap_or(default_include_heads),
                )
            }
            _ => (None, default_include_heads),
        };

        let traversal = CasesTraversalConfig {
            levels,
            max_matches: None,
            include_heads,
        };
        let parts = collect_level_parts(&args[0], traversal, 0).parts;
        Ok(Expr::call(
            apply_head.unwrap_or_else(|| system_expr("List")),
            parts,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LevelCollection {
    parts: Vec<Expr>,
    depth: usize,
}

fn collect_level_parts(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> LevelCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if traversal.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth: 1 }
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut parts = Vec::new();
                let mut max_child_depth = 1usize;

                if traversal.include_heads {
                    let head_result = collect_level_parts(head, traversal, level + 1);
                    parts.extend(head_result.parts);
                }

                for (_, value) in entries {
                    let value_result = collect_level_parts(&value, traversal, level + 1);
                    max_child_depth = max_child_depth.max(value_result.depth);
                    parts.extend(value_result.parts);
                }

                let depth = max_child_depth.saturating_add(1);
                if traversal.levels.matches(level, depth) {
                    parts.push(expr.clone());
                }

                LevelCollection { parts, depth }
            } else {
                collect_level_parts_in_standard_call(expr, traversal, level)
            }
        }
        Expr::Call { .. } => collect_level_parts_in_standard_call(expr, traversal, level),
    }
}

fn collect_level_parts_in_standard_call(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> LevelCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if traversal.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth: 1 }
        }
        Expr::Call { head, args } => {
            let mut parts = Vec::new();
            let mut max_child_depth = 1;

            if traversal.include_heads {
                let head_result = collect_level_parts(head, traversal, level + 1);
                parts.extend(head_result.parts);
            }

            for arg in args {
                let arg_result = collect_level_parts(arg, traversal, level + 1);
                max_child_depth = max_child_depth.max(arg_result.depth);
                parts.extend(arg_result.parts);
            }

            let depth = max_child_depth.saturating_add(1);
            if traversal.levels.matches(level, depth) {
                parts.push(expr.clone());
            }

            LevelCollection { parts, depth }
        }
    }
}

pub fn collect_level_parts_owned(
    expr: &Expr,
    traversal: CasesTraversalConfig,
    level: usize,
) -> Vec<Expr> {
    collect_level_parts(expr, traversal, level).parts
}

fn parse_level_level_spec(expr: &Expr) -> Option<CasesLevelSpec> {
    if expr_as_signed_isize(expr) == Some(0) {
        return Some(CasesLevelSpec::range(1, Some(0)));
    }

    parse_standard_level_spec(expr)
}
