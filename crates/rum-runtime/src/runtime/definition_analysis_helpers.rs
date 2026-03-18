use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TaggedDefinitionTarget {
    OwnValue(Symbol),
    DownValue(Symbol),
    SubValue(Symbol),
    UpValue(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TagDefinitionError {
    TagNotFound,
    TagPositionInvalid,
}

pub(super) fn collect_with_binding_symbols(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    let Expr::Call { head, args } = expr else {
        return;
    };

    if !is_system_symbol_name(head, "List") {
        return;
    }

    for item in args {
        if let Some((symbol, _)) = parse_module_binding_item(item) {
            locals.insert(symbol);
        }
    }
}

pub(super) fn classify_definition_lhs(lhs: &Expr) -> Option<TaggedDefinitionTarget> {
    if let Some(symbol) = expr_as_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::OwnValue(symbol));
    }

    if let Some(symbol) = subvalue_base_symbol(lhs).cloned() {
        return Some(TaggedDefinitionTarget::SubValue(symbol));
    }

    downvalue_head_symbol(lhs)
        .cloned()
        .map(TaggedDefinitionTarget::DownValue)
}

pub(super) fn classify_tagged_definition(
    tag: &Symbol,
    lhs: &Expr,
) -> Result<TaggedDefinitionTarget, TagDefinitionError> {
    if let Expr::Call { head, args } = strip_hold_pattern(lhs) {
        if is_system_symbol_name(head, "Condition") && !args.is_empty() {
            return classify_tagged_definition(tag, &args[0]);
        }
    }

    if expr_as_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::OwnValue(tag.clone()));
    }

    if subvalue_base_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::SubValue(tag.clone()));
    }

    if downvalue_head_symbol(lhs).is_some_and(|symbol| symbol == tag) {
        return Ok(TaggedDefinitionTarget::DownValue(tag.clone()));
    }

    if matches!(strip_hold_pattern(lhs), Expr::Call { args, .. } if args.iter().any(|arg| valid_upvalue_tag_position(arg, tag)))
    {
        return Ok(TaggedDefinitionTarget::UpValue(tag.clone()));
    }

    if expr_contains_symbol(lhs, tag) {
        Err(TagDefinitionError::TagPositionInvalid)
    } else {
        Err(TagDefinitionError::TagNotFound)
    }
}

pub(super) fn valid_upvalue_tag_position(expr: &Expr, tag: &Symbol) -> bool {
    expr_as_symbol(expr).is_some_and(|symbol| symbol == tag)
        || match expr {
            Expr::Call { head, .. } => expr_as_symbol(head).is_some_and(|symbol| symbol == tag),
            _ => false,
        }
}

pub(super) fn expr_contains_symbol(expr: &Expr, target: &Symbol) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol == target,
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_symbol(head, target)
                || args.iter().any(|arg| expr_contains_symbol(arg, target))
        }
    }
}

pub(super) fn collect_upvalue_candidate_symbols(args: &[Expr]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for arg in args {
        collect_expr_symbols_unique(arg, &mut symbols);
    }

    symbols
}

pub(super) fn collect_expr_symbols_unique(expr: &Expr, symbols: &mut Vec<Symbol>) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if !symbols.iter().any(|existing| existing == symbol) {
                symbols.push(symbol.clone());
            }
        }
        Expr::Atom(_) => {}
        Expr::Call { head, args } => {
            collect_expr_symbols_unique(head, symbols);

            for arg in args {
                collect_expr_symbols_unique(arg, symbols);
            }
        }
    }
}

pub(super) fn lhs_patterns_equivalent(lhs: &Expr, rhs: &Expr) -> bool {
    canonicalize_pattern_names(lhs) == canonicalize_pattern_names(rhs)
}

pub(super) fn canonicalize_pattern_names(expr: &Expr) -> Expr {
    let mut mapping = BTreeMap::new();
    let mut next_id = 0_u64;
    collect_pattern_name_mapping(expr, &mut mapping, &mut next_id);
    substitute_pattern_name_mapping(expr, &mapping)
}

pub(super) fn collect_pattern_name_mapping(
    expr: &Expr,
    mapping: &mut BTreeMap<Symbol, Symbol>,
    next_id: &mut u64,
) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                mapping.entry(symbol.clone()).or_insert_with(|| {
                    *next_id += 1;
                    Symbol::new("Pattern`", format!("p{next_id}"))
                });
            }

            collect_pattern_name_mapping(&args[1], mapping, next_id);
        }
        Expr::Call { head, args } => {
            collect_pattern_name_mapping(head, mapping, next_id);

            for arg in args {
                collect_pattern_name_mapping(arg, mapping, next_id);
            }
        }
        Expr::Atom(_) => {}
    }
}

pub(super) fn substitute_pattern_name_mapping(
    expr: &Expr,
    mapping: &BTreeMap<Symbol, Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => mapping
            .get(symbol)
            .cloned()
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_pattern_name_mapping(head, mapping),
            args.iter()
                .map(|arg| substitute_pattern_name_mapping(arg, mapping))
                .collect(),
        ),
    }
}

pub(super) fn downvalue_head_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            downvalue_head_symbol(&args[0])
        }
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    }
}

pub(super) fn subvalue_base_symbol(expr: &Expr) -> Option<&Symbol> {
    match strip_hold_pattern(expr) {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            subvalue_base_symbol(&args[0])
        }
        Expr::Call { head, .. } => base_head_symbol_of_nested_call(head),
        _ => None,
    }
}

pub(super) fn base_head_symbol_of_nested_call(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Call { head, .. } => match &**head {
            Expr::Call { .. } => base_head_symbol_of_nested_call(head),
            _ => expr_as_symbol(head),
        },
        _ => None,
    }
}

pub(super) fn normalize_rewrite_rules(expr: &Expr) -> Option<Vec<RewriteRule>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut rules = Vec::new();

            for item in args {
                let rule = normalize_single_rewrite_rule(item)?;
                rules.push(rule);
            }

            Some(rules)
        }
        _ => normalize_single_rewrite_rule(expr).map(|rule| vec![rule]),
    }
}

pub(super) fn normalize_single_rewrite_rule(expr: &Expr) -> Option<RewriteRule> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let delayed = is_system_symbol_name(head, "RuleDelayed");

            Some(RewriteRule {
                lhs: args[0].clone(),
                rhs: args[1].clone(),
                delayed,
                matcher: compile_matcher(&args[0]),
            })
        }
        _ => None,
    }
}
