use super::*;

pub(super) fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if shadowed.contains(symbol) {
                Expr::Atom(Atom::Symbol(symbol.clone()))
            } else {
                bindings
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone())))
            }
        }
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut rhs_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut rhs_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &rhs_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut condition_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut condition_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &condition_shadowed),
                ],
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_bindings,
                    substitute_lexical_bindings(&args[1], bindings, &body_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);
            let mut substituted_args = vec![substituted_bindings];

            substituted_args.extend(
                args[1..]
                    .iter()
                    .map(|arg| substitute_lexical_bindings(arg, bindings, &body_shadowed)),
            );

            Expr::call(substituted_head, substituted_args)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);

            match parse_function(expr) {
                Some(FunctionSpec::SlotBased { body, .. }) => {
                    let mut substituted_args =
                        vec![substitute_lexical_bindings(body, bindings, shadowed)];
                    substituted_args.extend(
                        args[1..]
                            .iter()
                            .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed)),
                    );
                    Expr::call(substituted_head, substituted_args)
                }
                Some(FunctionSpec::Explicit { params, body, .. }) => {
                    let mut body_shadowed = shadowed.clone();
                    body_shadowed.extend(params);

                    let mut substituted_args = vec![args[0].clone()];
                    substituted_args.push(substitute_lexical_bindings(
                        body,
                        bindings,
                        &body_shadowed,
                    ));
                    substituted_args.extend(
                        args[2..]
                            .iter()
                            .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed)),
                    );

                    Expr::call(substituted_head, substituted_args)
                }
                None => Expr::call(
                    substituted_head,
                    args.iter()
                        .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
                        .collect(),
                ),
            }
        }
        Expr::Call { head, args } => Expr::call(
            substitute_lexical_bindings(head, bindings, shadowed),
            args.iter()
                .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
                .collect(),
        ),
    }
}

pub(super) fn substitute_with_binding_list(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let Expr::Call { head, args } = expr else {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    };

    if !is_system_symbol_name(head, "List") {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    }

    Expr::call(
        substitute_lexical_bindings(head, bindings, shadowed),
        args.iter()
            .map(|item| substitute_with_binding_item(item, bindings, shadowed))
            .collect(),
    )
}

pub(super) fn substitute_with_binding_item(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => expr.clone(),
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        _ => substitute_lexical_bindings(expr, bindings, shadowed),
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
