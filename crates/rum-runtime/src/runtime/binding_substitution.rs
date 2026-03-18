use super::*;

pub(super) fn bind_pattern_name(
    name: &Symbol,
    value: Expr,
    bindings: &mut BTreeMap<Symbol, Expr>,
) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == &value,
        None => {
            bindings.insert(name.clone(), value);
            true
        }
    }
}

pub(super) fn sequence_expr(values: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Sequence"), values)
}

pub(super) fn substitute_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_bindings(head, bindings),
            args.iter()
                .map(|arg| substitute_bindings(arg, bindings))
                .collect(),
        ),
    }
}

pub(super) fn substitute_rule_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => {
            let substituted_head = substitute_rule_bindings(head, bindings);
            let mut substituted_args = Vec::new();

            for arg in args {
                let substituted_arg = substitute_rule_bindings(arg, bindings);

                if let Some(sequence_items) = sequence_items(&substituted_arg) {
                    substituted_args.extend(sequence_items.iter().cloned());
                } else {
                    substituted_args.push(substituted_arg);
                }
            }

            Expr::call(substituted_head, substituted_args)
        }
    }
}
