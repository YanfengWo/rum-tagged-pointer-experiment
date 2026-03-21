use super::*;

fn substitute_symbol_binding(
    symbol: &Symbol,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    if shadowed.contains(symbol) {
        Expr::Atom(Atom::Symbol(symbol.clone()))
    } else {
        bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone())))
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

fn substitute_pattern_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    Expr::call(
        substitute_lexical_bindings(head, bindings, shadowed),
        vec![
            args[0].clone(),
            substitute_lexical_bindings(&args[1], bindings, shadowed),
        ],
    )
}

fn substitute_assignment_like_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
    let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
    let rhs_shadowed = shadowed_with_pattern_locals(shadowed, &substituted_lhs);

    Expr::call(
        substituted_head,
        vec![
            substituted_lhs,
            substitute_lexical_bindings(&args[1], bindings, &rhs_shadowed),
        ],
    )
}

fn substitute_condition_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
    let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
    let condition_shadowed = shadowed_with_pattern_locals(shadowed, &substituted_lhs);

    Expr::call(
        substituted_head,
        vec![
            substituted_lhs,
            substitute_lexical_bindings(&args[1], bindings, &condition_shadowed),
        ],
    )
}

fn substitute_with_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
    let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
    let body_shadowed = shadowed_with_binding_symbols(shadowed, &args[0]);

    Expr::call(
        substituted_head,
        vec![
            substituted_bindings,
            substitute_lexical_bindings(&args[1], bindings, &body_shadowed),
        ],
    )
}

fn substitute_scoped_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
    let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
    let body_shadowed = shadowed_with_binding_symbols(shadowed, &args[0]);
    let mut substituted_args = vec![substituted_bindings];

    substituted_args.extend(
        args[1..]
            .iter()
            .map(|arg| substitute_lexical_bindings(arg, bindings, &body_shadowed)),
    );

    Expr::call(substituted_head, substituted_args)
}

fn substitute_function_call(
    expr: &Expr,
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);

    match parse_function(expr) {
        Some(FunctionSpec::SlotBased { body, .. }) => {
            let mut substituted_args = vec![substitute_lexical_bindings(body, bindings, shadowed)];
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
            substituted_args.push(substitute_lexical_bindings(body, bindings, &body_shadowed));
            substituted_args.extend(
                args[2..]
                    .iter()
                    .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed)),
            );

            Expr::call(substituted_head, substituted_args)
        }
        None => substitute_generic_call(head, args, bindings, shadowed),
    }
}

fn substitute_generic_call(
    head: &Expr,
    args: &[Expr],
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    Expr::call(
        substitute_lexical_bindings(head, bindings, shadowed),
        args.iter()
            .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
            .collect(),
    )
}

pub(super) fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => substitute_symbol_binding(symbol, bindings, shadowed),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            substitute_pattern_call(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            substitute_assignment_like_call(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            substitute_condition_call(head, args, bindings, shadowed)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            substitute_with_call(head, args, bindings, shadowed)
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            substitute_scoped_call(head, args, bindings, shadowed)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            substitute_function_call(expr, head, args, bindings, shadowed)
        }
        Expr::Call { head, args } => substitute_generic_call(head, args, bindings, shadowed),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_rhs_substitution_respects_pattern_locals() {
        let x = Symbol::from_name("x");
        let y = Symbol::from_name("y");
        let mut bindings = BTreeMap::new();
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
        let mut bindings = BTreeMap::new();
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
        let mut bindings = BTreeMap::new();
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
