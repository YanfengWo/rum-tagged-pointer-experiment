use super::*;

struct LexicalBindings<'a> {
    map: &'a SymbolExprMap,
    single: Option<(&'a Symbol, &'a Expr)>,
}

impl<'a> LexicalBindings<'a> {
    fn new(map: &'a SymbolExprMap) -> Self {
        let single = if map.len() == 1 {
            map.iter().next()
        } else {
            None
        };
        Self { map, single }
    }

    #[inline]
    fn get(&self, symbol: &Symbol) -> Option<&'a Expr> {
        if let Some((bound_symbol, value)) = self.single {
            if bound_symbol == symbol {
                return Some(value);
            }
            return None;
        }

        self.map.get(symbol)
    }

    fn may_contain_binding_symbol(&self, expr: &Expr) -> bool {
        if let Some((symbol, _)) = self.single {
            return expr_may_contain_symbol(expr, symbol);
        }

        expr_may_contain_any_binding(expr, self.map)
    }
}

fn expr_may_contain_any_binding(expr: &Expr, bindings: &SymbolExprMap) -> bool {
    if lexical_profile_enabled() {
        let profile_start = if lexical_profile_detail_enabled() {
            lexical_profile_start()
        } else {
            None
        };
        let (result, nodes) = expr_may_contain_any_binding_counted(expr, bindings);
        record_profile_multi_presence_scan(profile_start, expr, result, nodes);
        return result;
    }

    expr_may_contain_any_binding_fast(expr, bindings)
}

fn expr_may_contain_any_binding_fast(expr: &Expr, bindings: &SymbolExprMap) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings.contains_key(symbol),
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_may_contain_any_binding_fast(head, bindings)
                || args
                    .iter()
                    .any(|arg| expr_may_contain_any_binding_fast(arg, bindings))
        }
    }
}

fn expr_may_contain_any_binding_counted(expr: &Expr, bindings: &SymbolExprMap) -> (bool, u64) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => (bindings.contains_key(symbol), 1),
        Expr::Atom(_) => (false, 1),
        Expr::Call { head, args } => {
            let (head_result, mut nodes) = expr_may_contain_any_binding_counted(head, bindings);
            if head_result {
                return (true, nodes);
            }

            for arg in args.iter() {
                let (arg_result, arg_nodes) = expr_may_contain_any_binding_counted(arg, bindings);
                nodes = nodes.saturating_add(arg_nodes);
                if arg_result {
                    return (true, nodes);
                }
            }

            (false, nodes)
        }
    }
}

fn expr_may_contain_symbol(expr: &Expr, target: &Symbol) -> bool {
    if lexical_profile_enabled() {
        let profile_start = if lexical_profile_detail_enabled() {
            lexical_profile_start()
        } else {
            None
        };
        let (result, nodes) = expr_may_contain_symbol_counted(expr, target);
        record_profile_single_presence_scan(profile_start, expr, target, result, nodes);
        return result;
    }

    expr_may_contain_symbol_fast(expr, target)
}

fn expr_may_contain_symbol_fast(expr: &Expr, target: &Symbol) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol == target,
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_may_contain_symbol_fast(head, target)
                || args
                    .iter()
                    .any(|arg| expr_may_contain_symbol_fast(arg, target))
        }
    }
}

fn expr_may_contain_symbol_counted(expr: &Expr, target: &Symbol) -> (bool, u64) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => (symbol == target, 1),
        Expr::Atom(_) => (false, 1),
        Expr::Call { head, args } => {
            let (head_result, mut nodes) = expr_may_contain_symbol_counted(head, target);
            if head_result {
                return (true, nodes);
            }

            for arg in args.iter() {
                let (arg_result, arg_nodes) = expr_may_contain_symbol_counted(arg, target);
                nodes = nodes.saturating_add(arg_nodes);
                if arg_result {
                    return (true, nodes);
                }
            }

            (false, nodes)
        }
    }
}

fn substitute_symbol_binding_changed(
    symbol: &Symbol,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let value = bindings.get(symbol)?;
    if shadowed.contains(symbol) {
        None
    } else {
        Some(value.clone())
    }
}

fn shadowed_with_pattern_locals(shadowed: &BTreeSet<Symbol>, expr: &Expr) -> BTreeSet<Symbol> {
    let mut locals = shadowed.clone();
    collect_pattern_locals(expr, &mut locals);
    locals
}

fn shadowed_with_binding_symbols(shadowed: &BTreeSet<Symbol>, expr: &Expr) -> BTreeSet<Symbol> {
    let mut locals = shadowed.clone();
    collect_with_binding_symbols(expr, &mut locals);
    locals
}

fn substitute_pattern_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("pattern_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_pattern = substitute_lexical_bindings_changed(&args[1], bindings, shadowed);

    if substituted_head.is_none() && substituted_pattern.is_none() {
        return None;
    }

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        vec![
            args[0].clone(),
            substituted_pattern.unwrap_or_else(|| args[1].clone()),
        ],
    ))
}
fn substitute_assignment_like_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("assignment_like_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_lhs = substitute_lexical_bindings_changed(&args[0], bindings, shadowed);
    let lhs_for_shadow = substituted_lhs.as_ref().unwrap_or(&args[0]);
    let rhs_shadowed = shadowed_with_pattern_locals(shadowed, lhs_for_shadow);
    let substituted_rhs = substitute_lexical_bindings_changed(&args[1], bindings, &rhs_shadowed);

    if substituted_head.is_none() && substituted_lhs.is_none() && substituted_rhs.is_none() {
        return None;
    }

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        vec![
            substituted_lhs.unwrap_or_else(|| args[0].clone()),
            substituted_rhs.unwrap_or_else(|| args[1].clone()),
        ],
    ))
}

fn substitute_condition_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("condition_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_lhs = substitute_lexical_bindings_changed(&args[0], bindings, shadowed);
    let lhs_for_shadow = substituted_lhs.as_ref().unwrap_or(&args[0]);
    let condition_shadowed = shadowed_with_pattern_locals(shadowed, lhs_for_shadow);
    let substituted_condition =
        substitute_lexical_bindings_changed(&args[1], bindings, &condition_shadowed);

    if substituted_head.is_none() && substituted_lhs.is_none() && substituted_condition.is_none() {
        return None;
    }

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        vec![
            substituted_lhs.unwrap_or_else(|| args[0].clone()),
            substituted_condition.unwrap_or_else(|| args[1].clone()),
        ],
    ))
}

fn substitute_with_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("with_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_bindings = substitute_with_binding_list_changed(&args[0], bindings, shadowed);
    let body_shadowed = shadowed_with_binding_symbols(shadowed, &args[0]);
    let substituted_body = substitute_lexical_bindings_changed(&args[1], bindings, &body_shadowed);

    if substituted_head.is_none() && substituted_bindings.is_none() && substituted_body.is_none() {
        return None;
    }

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        vec![
            substituted_bindings.unwrap_or_else(|| args[0].clone()),
            substituted_body.unwrap_or_else(|| args[1].clone()),
        ],
    ))
}

fn substitute_scoped_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("scoped_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_bindings = substitute_with_binding_list_changed(&args[0], bindings, shadowed);
    let body_shadowed = shadowed_with_binding_symbols(shadowed, &args[0]);
    let substituted_rest =
        substitute_arg_list_changed(&args[1..], bindings, &body_shadowed, head, 1);

    if substituted_head.is_none() && substituted_bindings.is_none() && substituted_rest.is_none() {
        return None;
    }

    let mut substituted_args =
        Vec::with_capacity(1 + substituted_rest.as_ref().map_or(args.len() - 1, Vec::len));
    substituted_args.push(substituted_bindings.unwrap_or_else(|| args[0].clone()));
    substituted_args.extend(substituted_rest.unwrap_or_else(|| args[1..].to_vec()));

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        substituted_args,
    ))
}

fn substitute_function_call_changed(
    expr: &Expr,
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("function_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    match parse_function(expr) {
        Some(FunctionSpec::SlotBased { body, .. }) => {
            let substituted_body = substitute_lexical_bindings_changed(body, bindings, shadowed);
            let substituted_rest =
                substitute_arg_list_changed(&args[1..], bindings, shadowed, head, 1);

            if substituted_head.is_none()
                && substituted_body.is_none()
                && substituted_rest.is_none()
            {
                return None;
            }

            let mut substituted_args =
                Vec::with_capacity(1 + substituted_rest.as_ref().map_or(args.len() - 1, Vec::len));
            substituted_args.push(substituted_body.unwrap_or_else(|| body.clone()));
            substituted_args.extend(substituted_rest.unwrap_or_else(|| args[1..].to_vec()));

            Some(Expr::call(
                substituted_head.unwrap_or_else(|| head.clone()),
                substituted_args,
            ))
        }
        Some(FunctionSpec::Explicit { params, body, .. }) => {
            let mut body_shadowed = shadowed.clone();
            body_shadowed.extend(params);

            let substituted_body =
                substitute_lexical_bindings_changed(body, bindings, &body_shadowed);
            let substituted_rest =
                substitute_arg_list_changed(&args[2..], bindings, shadowed, head, 2);

            if substituted_head.is_none()
                && substituted_body.is_none()
                && substituted_rest.is_none()
            {
                return None;
            }

            let mut substituted_args =
                Vec::with_capacity(2 + substituted_rest.as_ref().map_or(args.len() - 2, Vec::len));
            substituted_args.push(args[0].clone());
            substituted_args.push(substituted_body.unwrap_or_else(|| body.clone()));
            substituted_args.extend(substituted_rest.unwrap_or_else(|| args[2..].to_vec()));

            Some(Expr::call(
                substituted_head.unwrap_or_else(|| head.clone()),
                substituted_args,
            ))
        }
        None => substitute_generic_call_changed(head, args, bindings, shadowed),
    }
}

fn substitute_generic_call_changed(
    head: &Expr,
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("generic_call");
    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let substituted_args = substitute_arg_list_changed(args, bindings, shadowed, head, 0);

    match (substituted_head, substituted_args) {
        (None, None) => None,
        (Some(head), None) => Some(Expr::call_with_shared_args(
            head,
            SharedArgs::from(args.to_vec()),
        )),
        (None, Some(args)) => Some(Expr::call_shared(
            SharedExpr::from(head.clone()),
            SharedArgs::new(args),
        )),
        (Some(head), Some(args)) => Some(Expr::call(head, args)),
    }
}

fn substitute_arg_list_changed(
    args: &[Expr],
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
    parent_head: &Expr,
    index_offset: usize,
) -> Option<Vec<Expr>> {
    record_profile_arg_list_start(args.len());
    let profile_detail = lexical_profile_detail_enabled();
    let mut substituted_args: Option<Vec<Expr>> = None;

    for (index, arg) in args.iter().enumerate() {
        let maybe_contains_binding = {
            let _profile_site =
                lexical_profile_arg_prefilter_site(parent_head, index + index_offset);
            let _profile_context = lexical_profile_context("arg_prefilter");
            bindings.may_contain_binding_symbol(arg)
        };
        record_profile_arg_prefilter(maybe_contains_binding);
        if !maybe_contains_binding {
            if let Some(substituted_args) = &mut substituted_args {
                substituted_args.push(arg.clone());
            }
            continue;
        }

        let profile_start = if profile_detail {
            lexical_profile_start()
        } else {
            None
        };
        let substituted_arg = {
            let _profile_context = lexical_profile_context("arg_substitution");
            substitute_lexical_bindings_changed(arg, bindings, shadowed)
        };
        if profile_detail {
            record_profile_arg_substitution_attempt(profile_start, substituted_arg.is_some());
        }

        if let Some(substituted_arg) = substituted_arg {
            substituted_args
                .get_or_insert_with(|| args[..index].to_vec())
                .push(substituted_arg);
        } else if let Some(substituted_args) = &mut substituted_args {
            substituted_args.push(arg.clone());
        }
    }

    substituted_args
}

pub(super) fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &SymbolExprMap,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let profile_start = lexical_profile_start();
    record_profile_lexical_substitution_start(bindings.len(), shadowed.len());
    if bindings.is_empty() {
        record_profile_lexical_substitution_finish(profile_start, false);
        return expr.clone();
    }

    let bindings = LexicalBindings::new(bindings);
    let substituted = substitute_lexical_bindings_changed(expr, &bindings, shadowed);
    let changed = substituted.is_some();
    let result = substituted.unwrap_or_else(|| expr.clone());
    record_profile_lexical_substitution_finish(profile_start, changed);
    result
}

fn substitute_lexical_bindings_changed(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            substitute_symbol_binding_changed(symbol, bindings, shadowed)
        }
        Expr::Atom(_) => None,
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            substitute_pattern_call_changed(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            substitute_assignment_like_call_changed(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            substitute_condition_call_changed(head, args, bindings, shadowed)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            substitute_with_call_changed(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            substitute_scoped_call_changed(head, args, bindings, shadowed)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            substitute_function_call_changed(expr, head, args, bindings, shadowed)
        }
        Expr::Call { head, args } => {
            substitute_generic_call_changed(head, args, bindings, shadowed)
        }
    }
}

fn substitute_with_binding_list_changed(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("with_binding_list");
    let Expr::Call { head, args } = expr else {
        return substitute_lexical_bindings_changed(expr, bindings, shadowed);
    };

    if !is_system_symbol_name(head, "List") {
        return substitute_lexical_bindings_changed(expr, bindings, shadowed);
    }

    let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
    let mut substituted_items = None;

    for (index, item) in args.iter().enumerate() {
        if let Some(substituted_item) =
            substitute_with_binding_item_changed(item, bindings, shadowed)
        {
            substituted_items
                .get_or_insert_with(|| args[..index].to_vec())
                .push(substituted_item);
        } else if let Some(substituted_items) = &mut substituted_items {
            substituted_items.push(item.clone());
        }
    }

    match (substituted_head, substituted_items) {
        (None, None) => None,
        (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
        (None, Some(args)) => Some(Expr::call_shared(head.clone(), SharedArgs::new(args))),
        (Some(head), Some(args)) => Some(Expr::call(head, args)),
    }
}

fn substitute_with_binding_item_changed(
    expr: &Expr,
    bindings: &LexicalBindings<'_>,
    shadowed: &BTreeSet<Symbol>,
) -> Option<Expr> {
    let _profile_context = lexical_profile_context("with_binding_item");
    match expr {
        Expr::Atom(Atom::Symbol(_)) => None,
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            let substituted_head = substitute_lexical_bindings_changed(head, bindings, shadowed);
            let substituted_rhs = substitute_lexical_bindings_changed(&args[1], bindings, shadowed);

            if substituted_head.is_none() && substituted_rhs.is_none() {
                return None;
            }

            Some(Expr::call(
                substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                vec![
                    args[0].clone(),
                    substituted_rhs.unwrap_or_else(|| args[1].clone()),
                ],
            ))
        }
        _ => substitute_lexical_bindings_changed(expr, bindings, shadowed),
    }
}

pub(super) fn collect_pattern_locals(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                locals.insert(symbol.clone());
            }
            collect_pattern_locals(&args[1], locals);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_pattern_locals(arg, locals);
            }
        }
        Expr::Atom(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_rhs_substitution_respects_pattern_locals() {
        let x = Symbol::from_name("x");
        let y = Symbol::from_name("y");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x.clone(), Expr::integer(42_i64.into()));
        bindings.insert(y.clone(), Expr::integer(7_i64.into()));

        let expr = Expr::call(
            system_expr("Rule"),
            vec![
                Expr::call(
                    system_expr("Pattern"),
                    vec![Expr::Atom(Atom::Symbol(x.clone())), system_expr("Blank")],
                ),
                Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::Atom(Atom::Symbol(x.clone())),
                        Expr::Atom(Atom::Symbol(y.clone())),
                    ],
                ),
            ],
        );

        let substituted = substitute_lexical_bindings(&expr, &bindings, &BTreeSet::new());

        assert_eq!(
            substituted,
            Expr::call(
                system_expr("Rule"),
                vec![
                    Expr::call(
                        system_expr("Pattern"),
                        vec![Expr::Atom(Atom::Symbol(x.clone())), system_expr("Blank")],
                    ),
                    Expr::call(
                        system_expr("List"),
                        vec![Expr::Atom(Atom::Symbol(x)), Expr::integer(7_i64.into())],
                    ),
                ],
            )
        );
    }

    #[test]
    fn explicit_function_parameters_shadow_outer_bindings() {
        let x = Symbol::from_name("x");
        let y = Symbol::from_name("y");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x.clone(), Expr::integer(1_i64.into()));
        bindings.insert(y.clone(), Expr::integer(2_i64.into()));

        let expr = Expr::call(
            system_expr("Function"),
            vec![
                Expr::call(
                    system_expr("List"),
                    vec![Expr::Atom(Atom::Symbol(x.clone()))],
                ),
                Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::Atom(Atom::Symbol(x.clone())),
                        Expr::Atom(Atom::Symbol(y.clone())),
                    ],
                ),
            ],
        );

        let substituted = substitute_lexical_bindings(&expr, &bindings, &BTreeSet::new());

        assert_eq!(
            substituted,
            Expr::call(
                system_expr("Function"),
                vec![
                    Expr::call(
                        system_expr("List"),
                        vec![Expr::Atom(Atom::Symbol(x.clone()))]
                    ),
                    Expr::call(
                        system_expr("List"),
                        vec![Expr::Atom(Atom::Symbol(x)), Expr::integer(2_i64.into())],
                    ),
                ],
            )
        );
    }

    #[test]
    fn with_body_treats_binding_names_as_shadowed() {
        let x = Symbol::from_name("x");
        let y = Symbol::from_name("y");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x.clone(), Expr::integer(10_i64.into()));
        bindings.insert(y.clone(), Expr::integer(20_i64.into()));

        let expr = Expr::call(
            system_expr("With"),
            vec![
                Expr::call(
                    system_expr("List"),
                    vec![Expr::call(
                        system_expr("Rule"),
                        vec![
                            Expr::Atom(Atom::Symbol(x.clone())),
                            Expr::Atom(Atom::Symbol(y)),
                        ],
                    )],
                ),
                Expr::call(
                    system_expr("List"),
                    vec![Expr::Atom(Atom::Symbol(x.clone()))],
                ),
            ],
        );

        let substituted = substitute_lexical_bindings(&expr, &bindings, &BTreeSet::new());

        assert_eq!(
            substituted,
            Expr::call(
                system_expr("With"),
                vec![
                    Expr::call(
                        system_expr("List"),
                        vec![Expr::call(
                            system_expr("Rule"),
                            vec![
                                Expr::Atom(Atom::Symbol(x.clone())),
                                Expr::integer(20_i64.into()),
                            ],
                        )],
                    ),
                    Expr::call(system_expr("List"), vec![Expr::Atom(Atom::Symbol(x))],),
                ],
            )
        );
    }
}
