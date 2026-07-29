use super::super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssociationEntryForm {
    /// Association entry key expression.
    pub key: Expr,
    /// Association entry value expression.
    pub value: Expr,
    /// Whether the entry preserves a RuleDelayed head.
    pub delayed: bool,
}

pub fn association_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entries(
        parse_association_entries_deep(args)?,
    ))
}

pub fn association_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entry_forms(
        parse_association_entry_forms_deep(args)?,
    ))
}

pub(in crate::runtime) fn keyed_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(in crate::runtime) fn parse_association_entries_deep(
    items: &[Expr],
) -> Option<Vec<(Expr, Expr)>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_deep(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(expr) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(entries);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries_deep(args)
}

pub fn render_association(entries: Vec<(Expr, Expr)>) -> Expr {
    let entries = canonicalize_association_entries(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    )
}

pub fn render_association_entry_forms_preserving_heads(entries: Vec<AssociationEntryForm>) -> Expr {
    render_association_entry_forms(entries)
}

pub(in crate::runtime) fn association_value_by_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value)
}

pub(in crate::runtime) fn association_position_and_value_by_key(
    expr: &Expr,
    key: &Expr,
) -> Option<(isize, Expr)> {
    association_entries_from_expr(expr)?
        .into_iter()
        .enumerate()
        .find(|(_, (entry_key, _))| entry_key == key)
        .map(|(index, (_, value))| (index as isize + 1, value))
}

pub(in crate::runtime) fn normalize_key_value_pattern_spec(spec: &Expr) -> Option<Expr> {
    if let Some(entry) = parse_association_entry_form(spec) {
        return Some(Expr::call(
            system_expr("List"),
            vec![association_entry_form_expr(entry)],
        ));
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let entries = parse_association_entry_forms(args)?;
    Some(Expr::call(
        system_expr("List"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    ))
}

pub fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn canonicalize_association_entries(entries: Vec<(Expr, Expr)>) -> Vec<(Expr, Expr)> {
    let mut canonical = Vec::new();

    for (key, value) in entries {
        if let Some((_, existing_value)) = canonical
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            *existing_value = value;
        } else {
            canonical.push((key, value));
        }
    }

    canonical
}

pub(in crate::runtime) fn parse_association_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

pub fn parse_association_entry_form(expr: &Expr) -> Option<AssociationEntryForm> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some(AssociationEntryForm {
        key: args[0].clone(),
        value: args[1].clone(),
        delayed: is_system_symbol_name(head, "RuleDelayed"),
    })
}

fn rule_list_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms(args)
}

fn parse_association_entry_forms(items: &[Expr]) -> Option<Vec<AssociationEntryForm>> {
    items.iter().map(parse_association_entry_form).collect()
}

pub(in crate::runtime) fn parse_association_entry_forms_deep(
    items: &[Expr],
) -> Option<Vec<AssociationEntryForm>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_forms_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_forms_deep(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Association")
        || is_system_symbol_name(head, heads::structure::LIST)
    {
        return parse_association_entry_forms_deep(args);
    }

    None
}

fn render_association_entry_forms(entries: Vec<AssociationEntryForm>) -> Expr {
    let entries = canonicalize_association_entry_forms(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    )
}

pub(in crate::runtime) fn association_entry_form_expr(entry: AssociationEntryForm) -> Expr {
    Expr::call(
        system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
        vec![entry.key, entry.value],
    )
}

fn canonicalize_association_entry_forms(
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut canonical: Vec<AssociationEntryForm> = Vec::new();
    for entry in entries {
        if let Some(existing) = canonical
            .iter_mut()
            .find(|existing| existing.key == entry.key)
        {
            *existing = entry;
        } else {
            canonical.push(entry);
        }
    }

    canonical
}

pub(in crate::runtime) fn invalid_association_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association.")
}

pub(in crate::runtime) fn is_association_expr(expr: &Expr) -> bool {
    association_entries_from_expr(expr).is_some()
}

pub(in crate::runtime) fn apply_association_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    match args {
        [] => association_entries_from_expr(head).map(|_| head.clone()),
        [key] => lookup_association_key(head, key),
        _ => apply_association_to_key_path(head, args),
    }
}

fn lookup_association_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    let entries = association_entries_from_expr(expr)?;
    Some(
        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
    )
}

fn apply_association_to_key_path(head: &Expr, keys: &[Expr]) -> Option<Expr> {
    let mut current = head.clone();
    for (index, key) in keys.iter().enumerate() {
        let Some(value) = lookup_association_key(&current, key) else {
            return None;
        };

        if is_missing_expr(&value) || index + 1 == keys.len() {
            return Some(value);
        }

        if !is_association_expr(&value) {
            return None;
        }
        current = value;
    }

    Some(current)
}

pub fn missing_key_absent_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("KeyAbsent"), key])
}

pub fn is_missing_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Missing"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssociationSetLValueTarget {
    FunctionCall {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
    Part {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssociationSetUpdate {
    Updated(Expr),
    MissingKey(Expr),
    Unchanged,
}

impl AssociationSetLValueTarget {
    fn symbol(&self) -> &Symbol {
        match self {
            Self::FunctionCall { symbol, .. } | Self::Part { symbol, .. } => symbol,
        }
    }

    fn selectors(&self) -> &[Expr] {
        match self {
            Self::FunctionCall { selectors, .. } | Self::Part { selectors, .. } => selectors,
        }
    }

    fn is_direct_single_key_shape(&self) -> bool {
        match self {
            Self::FunctionCall { selectors, .. } => selectors.len() == 1,
            Self::Part { selectors, .. } => {
                matches!(selectors.as_slice(), [selector] if explicit_association_key_selector(selector).is_some())
            }
        }
    }

    fn direct_single_key(&self, evaluated_selectors: &[Expr]) -> Option<Expr> {
        let [selector] = evaluated_selectors else {
            return None;
        };

        match self {
            Self::FunctionCall { selectors, .. } if selectors.len() == 1 => Some(selector.clone()),
            Self::Part { selectors, .. } if matches!(selectors.as_slice(), [raw] if explicit_association_key_selector(raw).is_some()) => {
                explicit_association_key_selector(selector).cloned()
            }
            _ => None,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn try_set_association_lvalue(
        &mut self,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = self.normalize_association_set_lvalue_target(&lhs) else {
            return Ok(None);
        };

        let direct_preflight = target.is_direct_single_key_shape()
            && self.association_lvalue_has_valid_direct_proof(target.symbol());
        if !direct_preflight && self.association_lvalue_current(target.symbol()).is_none() {
            return Ok(None);
        }

        if !self.ensure_symbol_not_protected_for_write("Set", target.symbol()) {
            return Ok(Some(rhs));
        }

        let value =
            self.resume_association_set_lvalue_selector_evaluation(target, rhs, Vec::new(), 0)?;
        Ok(Some(value))
    }

    fn normalize_association_set_lvalue_target(
        &self,
        lhs: &Expr,
    ) -> Option<AssociationSetLValueTarget> {
        let Expr::Call { head, args } = lhs else {
            return None;
        };

        if is_system_symbol_name(head, "Part") && args.len() >= 2 {
            let symbol = self.normalize_symbol_target_expr(&args[0], true)?;
            return Some(AssociationSetLValueTarget::Part {
                symbol,
                selectors: args[1..].to_vec(),
            });
        }

        let symbol = expr_as_symbol(head)?.clone();
        (!args.is_empty()).then_some(AssociationSetLValueTarget::FunctionCall {
            symbol,
            selectors: args.to_vec(),
        })
    }

    fn association_lvalue_current(&self, symbol: &Symbol) -> Option<Expr> {
        let current = self
            .definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.clone())?;
        association_entry_forms_from_expr(&current)?;
        Some(current)
    }

    fn resume_association_set_lvalue_selector_evaluation(
        &mut self,
        target: AssociationSetLValueTarget,
        rhs: Expr,
        mut evaluated_selectors: Vec<Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= target.selectors().len() {
            return self.finish_association_set_lvalue(target, evaluated_selectors, rhs);
        }

        let selector = target.selectors()[index].clone();
        let selector_result = self.evaluate(selector);
        let selector = self.suspend_with_frame(selector_result, {
            let target = target.clone();
            let rhs = rhs.clone();
            let evaluated_selectors = evaluated_selectors.clone();
            move |session, selector| {
                let mut evaluated_selectors = evaluated_selectors;
                evaluated_selectors.push(selector);
                session.resume_association_set_lvalue_selector_evaluation(
                    target,
                    rhs,
                    evaluated_selectors,
                    index + 1,
                )
            }
        })?;

        evaluated_selectors.push(selector);
        self.resume_association_set_lvalue_selector_evaluation(
            target,
            rhs,
            evaluated_selectors,
            index + 1,
        )
    }

    fn finish_association_set_lvalue(
        &mut self,
        target: AssociationSetLValueTarget,
        evaluated_selectors: Vec<Expr>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some(key) = target.direct_single_key(&evaluated_selectors) {
            if self.try_direct_association_rule_update(target.symbol(), key, &rhs)? {
                return Ok(rhs);
            }
        }

        let Some(current) = self.association_lvalue_current(target.symbol()) else {
            return Ok(rhs);
        };

        let update = match target {
            AssociationSetLValueTarget::FunctionCall { .. } => {
                set_association_key_path(current, &evaluated_selectors, rhs.clone())
            }
            AssociationSetLValueTarget::Part { .. } => {
                set_association_part_path(current, &evaluated_selectors, rhs.clone())
            }
        };

        match update {
            AssociationSetUpdate::Updated(updated) => {
                let target_symbol = *target.symbol();
                let definition = self.definitions.entry(target_symbol).or_default();
                definition.own_value = Some(updated);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&target_symbol);
            }
            AssociationSetUpdate::MissingKey(key) => {
                self.push_message(
                    "Set",
                    "kval",
                    format!("The value for the key {key} does not exist."),
                );
            }
            AssociationSetUpdate::Unchanged => {}
        }

        Ok(rhs)
    }
}

fn set_association_part_path(current: Expr, selectors: &[Expr], rhs: Expr) -> AssociationSetUpdate {
    if let Some(keys) = association_part_key_path(selectors) {
        return set_association_key_path(current, &keys, rhs);
    }

    AssociationSetUpdate::Unchanged
}

fn association_part_key_path(selectors: &[Expr]) -> Option<Vec<Expr>> {
    selectors
        .iter()
        .map(association_part_key_selector)
        .collect()
}

fn explicit_association_key_selector(selector: &Expr) -> Option<&Expr> {
    match selector {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

fn association_part_key_selector(selector: &Expr) -> Option<Expr> {
    if matches!(selector, Expr::Atom(Atom::String(_))) {
        return Some(selector.clone());
    }

    match selector {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn set_association_key_path(current: Expr, selectors: &[Expr], rhs: Expr) -> AssociationSetUpdate {
    let Some((key, rest)) = selectors.split_first() else {
        return AssociationSetUpdate::Unchanged;
    };

    let Some(mut entries) = association_entry_forms_from_expr(&current) else {
        return AssociationSetUpdate::Unchanged;
    };

    if rest.is_empty() {
        upsert_immediate_association_entry(&mut entries, key.clone(), rhs);
        return AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(
            entries,
        ));
    }

    let Some(index) = entries.iter().position(|entry| entry.key == *key) else {
        return AssociationSetUpdate::MissingKey(key.clone());
    };

    match set_association_key_path(entries[index].value.clone(), rest, rhs) {
        AssociationSetUpdate::Updated(updated_child) => {
            entries[index].value = updated_child;
            entries[index].delayed = false;
            AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(entries))
        }
        other => other,
    }
}

fn upsert_immediate_association_entry(
    entries: &mut Vec<AssociationEntryForm>,
    key: Expr,
    rhs: Expr,
) {
    let updated = AssociationEntryForm {
        key: key.clone(),
        value: rhs,
        delayed: false,
    };

    if let Some(index) = entries.iter().position(|entry| entry.key == key) {
        entries[index] = updated;
    } else {
        entries.push(updated);
    }
}
