use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PartSpanSpec {
    pub(super) start: SpanEndpoint,
    pub(super) end: SpanEndpoint,
    pub(super) step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpanEndpoint {
    Index(isize),
    UpTo(Option<isize>),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PartSelector {
    Single(AtomicPartSpec),
    Key(Expr),
    Multi(Vec<PartSelector>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
    AssociationBranch(Vec<(Expr, SelectedPartShape)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtractPositionSpec {
    position_specs: Vec<Vec<PartSelector>>,
    multiple: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartSelectionFailure {
    PartW { part: Expr, target: Expr },
    PartD { spec: Option<Expr> },
    Take { span: PartSpanSpec, target: Expr },
    Pspec1 { spec: Expr },
}

impl Session {
    pub(in crate::runtime) fn eval_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Part",
                args,
                "argm",
                "Part called with 0 arguments; 1 or more arguments are expected.",
            ));
        }

        let Some(selectors) = parse_part_selectors_for_part(&args[1..]) else {
            if let Some(offender) = args[1..]
                .iter()
                .find(|selector| parse_part_selector_for_part(selector).is_none())
            {
                if is_invalid_span_expr(offender) {
                    self.push_invalid_span_selector_messages("Part", offender);
                } else {
                    self.push_message(
                        "Part",
                        "pkspec1",
                        format!(
                            "The expression {} cannot be used as a part specification.",
                            offender
                        ),
                    );
                }
            }
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if association_entries_from_expr(&args[0]).is_some()
            && selectors.iter().any(selector_mixes_key_and_non_key)
        {
            self.push_message(
                "Part",
                "pmix",
                "Integer and Key specifications cannot be mixed in the same part specification.",
            );
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let original = Expr::call(head.clone(), args.to_vec());
        let Some(value) = apply_part_selectors(&args[0], &selectors) else {
            if let Some(failure) = diagnose_part_selection_failure(&args[0], &selectors) {
                self.push_part_selection_failure("Part", failure, &original);
            }
            return Ok(original);
        };

        Ok(value)
    }

    pub(in crate::runtime) fn eval_extract(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if let [position_spec] = args {
            return Ok(extract_operator_form(position_spec.clone()));
        }

        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Extract",
                args,
                "argt",
                format!(
                    "Extract called with {} arguments; 2 or 3 arguments are expected.",
                    args.len()
                ),
            ));
        }

        let Some(position_spec) = parse_extract_position_spec(&args[1]) else {
            if let Some(span_expr) = first_invalid_span_expr(&args[1]) {
                self.push_invalid_span_selector_messages("Extract", span_expr);
            } else {
                self.push_message(
                    "Extract",
                    "psl1",
                    format!(
                        "Position specification {} in {} is not applicable.",
                        args[1],
                        Expr::call(head.clone(), args.to_vec())
                    ),
                );
            }
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let extracted = if position_spec.multiple {
            let mut items = Vec::new();
            for selectors in &position_spec.position_specs {
                let Some(value) = apply_part_selectors(&args[0], selectors) else {
                    if let Some(failure) = diagnose_part_selection_failure(&args[0], selectors) {
                        let original = Expr::call(head.clone(), args.to_vec());
                        self.push_part_selection_failure("Extract", failure, &original);
                    }
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                self.push_extract_missing_key_messages(&args[0], selectors);
                items.push(wrap_extracted_value(value, args.get(2)));
            }
            Expr::call(system_expr("List"), items)
        } else {
            let Some(value) = apply_part_selectors(&args[0], &position_spec.position_specs[0])
            else {
                if let Some(failure) =
                    diagnose_part_selection_failure(&args[0], &position_spec.position_specs[0])
                {
                    let original = Expr::call(head.clone(), args.to_vec());
                    self.push_part_selection_failure("Extract", failure, &original);
                }
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            self.push_extract_missing_key_messages(&args[0], &position_spec.position_specs[0]);
            wrap_extracted_value(value, args.get(2))
        };

        Ok(extracted)
    }

    fn push_extract_missing_key_messages(&mut self, expr: &Expr, selectors: &[PartSelector]) {
        let mut missing_keys = Vec::new();
        collect_missing_key_selections(expr, selectors, &mut missing_keys);
        for (key, target) in missing_keys {
            self.push_message(
                "Extract",
                "keyw",
                format!("Key {key} does not exist in {target}."),
            );
        }
    }

    fn push_part_selection_failure(
        &mut self,
        symbol: &str,
        failure: PartSelectionFailure,
        original: &Expr,
    ) {
        match failure {
            PartSelectionFailure::PartW { part, target } => self.push_message(
                symbol,
                "partw",
                format!("Part {} of {} does not exist.", part, target),
            ),
            PartSelectionFailure::PartD { spec } => {
                let rendered_spec = if symbol == "Extract" {
                    spec.as_ref().unwrap_or(original)
                } else {
                    original
                };
                self.push_message(
                    symbol,
                    "partd",
                    format!(
                        "Part specification {} is longer than depth of object.",
                        rendered_spec
                    ),
                );
            }
            PartSelectionFailure::Take { span, target } => {
                self.push_take_selection_failure(symbol, span, target);
            }
            PartSelectionFailure::Pspec1 { spec } => self.push_message(
                symbol,
                "pspec1",
                format!("Part specification {} is not applicable.", spec),
            ),
        }
    }

    fn push_take_selection_failure(&mut self, symbol: &str, span: PartSpanSpec, target: Expr) {
        let (start, end) = span_take_message_range(span);
        let text = format!(
            "Cannot take positions {} through {} in {}.",
            start, end, target
        );
        if self.messages.last().is_some_and(|message| {
            message.symbol == symbol && message.tag == "take" && message.text == text
        }) {
            return;
        }

        self.push_message(symbol, "take", text);
    }

    fn push_invalid_span_selector_messages(&mut self, symbol: &str, span_expr: &Expr) {
        self.push_invalid_upto_messages_in_span(span_expr);
        let text = format!(
            "{} is not a valid Span specification. A Span specification should be 1, 2 or 3 machine-sized integers separated by ;;. (Any of the integers can be omitted or replaced with All.)",
            span_expr
        );
        if self.has_message(symbol, "span", &text) {
            return;
        }

        self.push_message(symbol, "span", text);
    }

    fn has_message(&self, symbol: &str, tag: &str, text: &str) -> bool {
        self.messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag && message.text == text)
    }

    fn push_invalid_upto_message_once(&mut self, tag: &str, text: String) {
        if self.has_message("UpTo", tag, &text) {
            return;
        }

        self.push_message("UpTo", tag, text);
    }

    fn push_invalid_upto_messages_in_span(&mut self, expr: &Expr) {
        let Expr::Call { head, args } = expr else {
            return;
        };

        if is_system_symbol_name(head, "UpTo") {
            match args.as_slice() {
                [limit] if is_invalid_numeric_upto_limit(limit) => self
                    .push_invalid_upto_message_once(
                        "innf",
                        format!(
                            "Non-negative integer or Infinity expected at position 1 in {expr}."
                        ),
                    ),
                [_] => {}
                _ => self.push_invalid_upto_message_once(
                    "argx",
                    format!(
                        "UpTo called with {} arguments; 1 argument is expected.",
                        args.len()
                    ),
                ),
            }
            return;
        }

        for arg in args {
            self.push_invalid_upto_messages_in_span(arg);
        }
    }
}

pub(super) fn parse_part_selectors(exprs: &[Expr]) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector).collect()
}

pub(super) fn parse_part_selectors_for_part(exprs: &[Expr]) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector_for_part).collect()
}

pub(super) fn invalid_part_selector_for_part(exprs: &[Expr]) -> Option<&Expr> {
    exprs
        .iter()
        .find(|selector| parse_part_selector_for_part(selector).is_none())
}

fn parse_part_selector_for_part(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_list_item_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn parse_part_list_item_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(PartSelector::Single(AtomicPartSpec::Index(index)));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

fn parse_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn parse_scalar_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(PartSelector::Single(spec));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

fn parse_key_part_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

fn parse_atomic_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(AtomicPartSpec::Index(index));
    }

    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(AtomicPartSpec::All);
    }

    parse_span_part_spec(expr)
}

fn parse_span_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::SPAN) {
        return None;
    }

    let spec = match args.as_slice() {
        [value] => {
            let endpoint = parse_span_endpoint(value)?;
            PartSpanSpec {
                start: endpoint,
                end: endpoint,
                step: 1,
            }
        }
        [start, end] => PartSpanSpec {
            start: parse_span_endpoint(start)?,
            end: parse_span_endpoint(end)?,
            step: 1,
        },
        [start, end, step] => {
            let step = expr_as_signed_isize(step)?;
            PartSpanSpec {
                start: parse_span_endpoint(start)?,
                end: parse_span_endpoint(end)?,
                step,
            }
        }
        _ => return None,
    };

    Some(AtomicPartSpec::Span(spec))
}

fn parse_span_endpoint(expr: &Expr) -> Option<SpanEndpoint> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(SpanEndpoint::All);
    }

    if let Some(limit) = parse_upto_span_endpoint(expr) {
        return Some(SpanEndpoint::UpTo(limit));
    }

    expr_as_signed_isize(expr).map(SpanEndpoint::Index)
}

fn parse_upto_span_endpoint(expr: &Expr) -> Option<Option<isize>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let [limit] = args.as_slice() else {
        return None;
    };
    if !is_system_symbol_name(head, "UpTo") {
        return None;
    }

    if is_system_symbol_name(limit, heads::structure::INFINITY) {
        return Some(None);
    }

    let limit = expr_as_signed_isize(limit)?;
    (limit >= 0).then_some(Some(limit))
}

fn is_invalid_span_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, heads::structure::SPAN)
                && parse_span_part_spec(expr).is_none()
    )
}

fn first_invalid_span_expr(expr: &Expr) -> Option<&Expr> {
    if is_invalid_span_expr(expr) {
        return Some(expr);
    }

    let Expr::Call { args, .. } = expr else {
        return None;
    };

    args.iter().find_map(first_invalid_span_expr)
}

fn is_invalid_numeric_upto_limit(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return false;
    }

    match expr {
        Expr::Atom(Atom::Integer(value)) => value < &BigInt::from(0),
        Expr::Atom(Atom::Rational(_)) => true,
        _ => false,
    }
}

pub(super) fn apply_part_selectors(expr: &Expr, selectors: &[PartSelector]) -> Option<Expr> {
    let Some((first, rest)) = selectors.split_first() else {
        return Some(expr.clone());
    };

    match select_parts_for_selector(expr, first)? {
        SelectedPartShape::Leaf(selected) => apply_part_selectors(&selected, rest),
        SelectedPartShape::Branch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::Branch(apply_part_selectors_to_shape(selected, rest)?),
            Some(selected_shape_head_for_expr(expr, first)),
        )),
        SelectedPartShape::AssociationBranch(selected) => Some(selected_shape_to_expr(
            SelectedPartShape::AssociationBranch(apply_part_selectors_to_keyed_shape(
                selected, rest,
            )?),
            None,
        )),
    }
}

fn diagnose_part_selection_failure(
    expr: &Expr,
    selectors: &[PartSelector],
) -> Option<PartSelectionFailure> {
    let (first, rest) = selectors.split_first()?;
    let failure = diagnose_selector_failure(expr, first, rest)?;
    Some(match failure {
        PartSelectionFailure::PartD { .. } => PartSelectionFailure::PartD {
            spec: Some(selector_path_to_expr(selectors)),
        },
        other => other,
    })
}

fn diagnose_selector_failure(
    expr: &Expr,
    selector: &PartSelector,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    match select_parts_for_selector(expr, selector) {
        Some(shape) => diagnose_selected_shape_failure(shape, rest),
        None => diagnose_failed_selector(expr, selector),
    }
}

fn diagnose_selected_shape_failure(
    shape: SelectedPartShape,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if rest.is_empty() {
        return None;
    }

    match shape {
        SelectedPartShape::Leaf(expr) => diagnose_part_selection_failure(&expr, rest),
        SelectedPartShape::Branch(children) => diagnose_children_failure(children, rest),
        SelectedPartShape::AssociationBranch(children) => diagnose_children_failure(
            children
                .into_iter()
                .map(|(_, child)| child)
                .collect::<Vec<_>>(),
            rest,
        ),
    }
}

fn diagnose_children_failure(
    children: Vec<SelectedPartShape>,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if children.is_empty() {
        return Some(PartSelectionFailure::PartD { spec: None });
    }

    children
        .into_iter()
        .find_map(|child| diagnose_selected_shape_failure(child, rest))
}

fn diagnose_failed_selector(expr: &Expr, selector: &PartSelector) -> Option<PartSelectionFailure> {
    match selector {
        PartSelector::Key(_) => Some(PartSelectionFailure::Pspec1 {
            spec: selector_to_expr(selector),
        }),
        PartSelector::Single(AtomicPartSpec::Index(index)) => diagnose_failed_index(expr, *index),
        PartSelector::Single(AtomicPartSpec::All) => Some(PartSelectionFailure::PartD {
            spec: Some(selector_to_expr(selector)),
        }),
        PartSelector::Single(AtomicPartSpec::Span(span)) => Some(PartSelectionFailure::Take {
            span: *span,
            target: expr.clone(),
        }),
        PartSelector::Multi(specs) => diagnose_failed_multi_selector(expr, selector, specs),
    }
}

fn diagnose_failed_index(expr: &Expr, index: isize) -> Option<PartSelectionFailure> {
    if index == 0 {
        return None;
    }

    match expr {
        Expr::Atom(_) => Some(PartSelectionFailure::PartD {
            spec: Some(selector_to_expr(&PartSelector::Single(
                AtomicPartSpec::Index(index),
            ))),
        }),
        Expr::Call { .. } => Some(PartSelectionFailure::PartW {
            part: Expr::integer(BigInt::from(index)),
            target: expr.clone(),
        }),
    }
}

fn diagnose_failed_multi_selector(
    expr: &Expr,
    selector: &PartSelector,
    specs: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if association_entries_from_expr(expr).is_some() {
        if let Some(index) = specs.iter().find_map(selector_zero_index) {
            return Some(PartSelectionFailure::PartW {
                part: Expr::integer(BigInt::from(index)),
                target: expr.clone(),
            });
        }
    }

    for spec in specs {
        if select_parts_for_selector(expr, spec).is_some() {
            continue;
        }

        let failure = diagnose_failed_selector(expr, spec)?;
        return Some(match failure {
            PartSelectionFailure::PartW { target, .. } => PartSelectionFailure::PartW {
                part: selector_to_expr(selector),
                target,
            },
            PartSelectionFailure::Pspec1 { .. } => PartSelectionFailure::Pspec1 {
                spec: selector_to_expr(selector),
            },
            other => other,
        });
    }

    Some(PartSelectionFailure::Pspec1 {
        spec: selector_to_expr(selector),
    })
}

fn selector_zero_index(selector: &PartSelector) -> Option<isize> {
    match selector {
        PartSelector::Single(AtomicPartSpec::Index(0)) => Some(0),
        PartSelector::Multi(specs) => specs.iter().find_map(selector_zero_index),
        _ => None,
    }
}

fn selector_to_expr(selector: &PartSelector) -> Expr {
    match selector {
        PartSelector::Single(AtomicPartSpec::Index(index)) => Expr::integer(BigInt::from(*index)),
        PartSelector::Single(AtomicPartSpec::All) => system_expr(heads::structure::ALL),
        PartSelector::Single(AtomicPartSpec::Span(span)) => span_to_expr(*span),
        PartSelector::Key(key) => key_expr(key.clone()),
        PartSelector::Multi(specs) => Expr::call(
            system_expr(heads::structure::LIST),
            specs.iter().map(selector_to_expr).collect(),
        ),
    }
}

fn selector_path_to_expr(selectors: &[PartSelector]) -> Expr {
    Expr::call(
        system_expr(heads::structure::LIST),
        selectors.iter().map(selector_to_expr).collect(),
    )
}

fn span_to_expr(span: PartSpanSpec) -> Expr {
    Expr::call(
        system_expr(heads::structure::SPAN),
        vec![
            span_endpoint_to_expr(span.start),
            span_endpoint_to_expr(span.end),
            Expr::integer(BigInt::from(span.step)),
        ],
    )
}

fn span_take_message_range(span: PartSpanSpec) -> (Expr, Expr) {
    (
        span_take_message_start(span.start),
        span_take_message_end(span.end),
    )
}

fn span_take_message_start(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::All => Expr::integer(BigInt::from(1)),
        _ => span_endpoint_to_expr(endpoint),
    }
}

fn span_take_message_end(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::All => Expr::integer(BigInt::from(-1)),
        _ => span_endpoint_to_expr(endpoint),
    }
}

fn span_endpoint_to_expr(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::Index(index) => Expr::integer(BigInt::from(index)),
        SpanEndpoint::UpTo(limit) => Expr::call(
            system_expr("UpTo"),
            vec![
                limit
                    .map(|value| Expr::integer(BigInt::from(value)))
                    .unwrap_or_else(|| system_expr(heads::structure::INFINITY)),
            ],
        ),
        SpanEndpoint::All => system_expr(heads::structure::ALL),
    }
}

fn selected_shape_head_for_expr(expr: &Expr, selector: &PartSelector) -> Expr {
    if association_entries_from_expr(expr).is_some() && selector_contains_key(selector) {
        system_expr("Association")
    } else {
        structural_head_expr(expr)
    }
}

fn collect_missing_key_selections(
    expr: &Expr,
    selectors: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    let Some((first, rest)) = selectors.split_first() else {
        return;
    };

    if collect_missing_key_selections_for_key_selector(expr, first, rest, missing_keys) {
        return;
    }

    if let Some(shape) = select_parts_for_selector(expr, first) {
        collect_missing_key_selections_in_shape(shape, rest, missing_keys);
    }
}

fn collect_missing_key_selections_for_key_selector(
    expr: &Expr,
    selector: &PartSelector,
    rest: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) -> bool {
    match selector {
        PartSelector::Key(key) => {
            collect_missing_key_selection_for_key(expr, key, rest, missing_keys);
            true
        }
        PartSelector::Multi(specs)
            if association_entries_from_expr(expr).is_some()
                && specs
                    .iter()
                    .all(|spec| matches!(spec, PartSelector::Key(_))) =>
        {
            for spec in specs {
                let PartSelector::Key(key) = spec else {
                    continue;
                };
                collect_missing_key_selection_for_key(expr, key, rest, missing_keys);
            }
            true
        }
        _ => false,
    }
}

fn collect_missing_key_selection_for_key(
    expr: &Expr,
    key: &Expr,
    rest: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    let Some(entries) = association_entries_from_expr(expr) else {
        return;
    };

    if let Some((_, value)) = entries.iter().find(|(entry_key, _)| entry_key == key) {
        collect_missing_key_selections(value, rest, missing_keys);
    } else {
        missing_keys.push((key.clone(), expr.clone()));
    }
}

fn collect_missing_key_selections_in_shape(
    shape: SelectedPartShape,
    selectors: &[PartSelector],
    missing_keys: &mut Vec<(Expr, Expr)>,
) {
    match shape {
        SelectedPartShape::Leaf(expr) => {
            collect_missing_key_selections(&expr, selectors, missing_keys);
        }
        SelectedPartShape::Branch(children) => {
            for child in children {
                collect_missing_key_selections_in_shape(child, selectors, missing_keys);
            }
        }
        SelectedPartShape::AssociationBranch(children) => {
            for (_, child) in children {
                collect_missing_key_selections_in_shape(child, selectors, missing_keys);
            }
        }
    }
}

fn selector_contains_key(selector: &PartSelector) -> bool {
    match selector {
        PartSelector::Single(_) => false,
        PartSelector::Key(_) => true,
        PartSelector::Multi(specs) => specs.iter().any(selector_contains_key),
    }
}

fn selector_mixes_key_and_non_key(selector: &PartSelector) -> bool {
    let PartSelector::Multi(specs) = selector else {
        return false;
    };

    let has_key = specs
        .iter()
        .any(|spec| matches!(spec, PartSelector::Key(_)));
    let has_non_key = specs
        .iter()
        .any(|spec| !matches!(spec, PartSelector::Key(_)));
    (has_key && has_non_key) || specs.iter().any(selector_mixes_key_and_non_key)
}

fn apply_part_selectors_to_shape(
    selected: Vec<SelectedPartShape>,
    selectors: &[PartSelector],
) -> Option<Vec<SelectedPartShape>> {
    selected
        .into_iter()
        .map(|shape| apply_part_selectors_to_selected_shape(shape, selectors))
        .collect()
}

fn apply_part_selectors_to_selected_shape(
    shape: SelectedPartShape,
    selectors: &[PartSelector],
) -> Option<SelectedPartShape> {
    match shape {
        SelectedPartShape::Leaf(expr) => Some(SelectedPartShape::Leaf(apply_part_selectors(
            &expr, selectors,
        )?)),
        SelectedPartShape::Branch(children) => Some(SelectedPartShape::Branch(
            children
                .into_iter()
                .map(|child| apply_part_selectors_to_selected_shape(child, selectors))
                .collect::<Option<Vec<_>>>()?,
        )),
        SelectedPartShape::AssociationBranch(children) => {
            Some(SelectedPartShape::AssociationBranch(
                apply_part_selectors_to_keyed_shape(children, selectors)?,
            ))
        }
    }
}

fn apply_part_selectors_to_keyed_shape(
    selected: Vec<(Expr, SelectedPartShape)>,
    selectors: &[PartSelector],
) -> Option<Vec<(Expr, SelectedPartShape)>> {
    selected
        .into_iter()
        .map(|(key, shape)| {
            Some((
                key,
                apply_part_selectors_to_selected_shape(shape, selectors)?,
            ))
        })
        .collect()
}

fn selected_shape_to_expr(shape: SelectedPartShape, root_head: Option<Expr>) -> Expr {
    match shape {
        SelectedPartShape::Leaf(expr) => expr,
        SelectedPartShape::Branch(children) => Expr::call(
            root_head.unwrap_or_else(|| system_expr("List")),
            children
                .into_iter()
                .map(|child| selected_shape_to_expr(child, None))
                .collect(),
        ),
        SelectedPartShape::AssociationBranch(children) => render_association(
            children
                .into_iter()
                .map(|(key, child)| (key, selected_shape_to_expr(child, None)))
                .collect(),
        ),
    }
}

fn select_parts_for_selector(expr: &Expr, selector: &PartSelector) -> Option<SelectedPartShape> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return select_association_parts_for_selector(expr, &entries, selector);
    }

    match selector {
        PartSelector::Single(spec) => match spec {
            AtomicPartSpec::Index(index) => {
                Some(SelectedPartShape::Leaf(select_exact_part(expr, *index)?))
            }
            AtomicPartSpec::All | AtomicPartSpec::Span(_) => Some(SelectedPartShape::Branch(
                select_parts_for_atomic_spec(expr, spec)?
                    .into_iter()
                    .map(SelectedPartShape::Leaf)
                    .collect(),
            )),
        },
        PartSelector::Key(key) => {
            let entries = association_entries_from_expr(expr)?;
            Some(SelectedPartShape::Leaf(
                entries
                    .iter()
                    .find(|(entry_key, _)| entry_key == key)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
            ))
        }
        PartSelector::Multi(specs) => {
            if let Some(entries) = association_entries_from_expr(expr) {
                if specs
                    .iter()
                    .all(|spec| matches!(spec, PartSelector::Key(_)))
                {
                    return Some(SelectedPartShape::AssociationBranch(
                        specs
                            .iter()
                            .filter_map(|spec| match spec {
                                PartSelector::Key(key) => Some((
                                    key.clone(),
                                    SelectedPartShape::Leaf(
                                        entries
                                            .iter()
                                            .find(|(entry_key, _)| entry_key == key)
                                            .map(|(_, value)| value.clone())
                                            .unwrap_or_else(|| {
                                                missing_key_absent_expr(key_expr(key.clone()))
                                            }),
                                    ),
                                )),
                                _ => None,
                            })
                            .collect(),
                    ));
                }
            }

            let mut selected = Vec::new();
            for spec in specs {
                match (spec, select_parts_for_selector(expr, spec)?) {
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Leaf(expr),
                    ) => {
                        selected.push(SelectedPartShape::Leaf(expr));
                    }
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::Branch(children),
                    ) => {
                        selected.extend(children);
                    }
                    (
                        PartSelector::Single(_) | PartSelector::Key(_),
                        SelectedPartShape::AssociationBranch(children),
                    ) => {
                        selected.push(SelectedPartShape::AssociationBranch(children));
                    }
                    (PartSelector::Multi(_), shape) => selected.push(shape),
                }
            }
            Some(SelectedPartShape::Branch(selected))
        }
    }
}

fn select_association_parts_for_selector(
    expr: &Expr,
    entries: &[(Expr, Expr)],
    selector: &PartSelector,
) -> Option<SelectedPartShape> {
    match selector {
        PartSelector::Key(key) => Some(SelectedPartShape::Leaf(
            entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| missing_key_absent_expr(key_expr(key.clone()))),
        )),
        PartSelector::Single(AtomicPartSpec::Index(0)) => {
            Some(SelectedPartShape::Leaf(structural_head_expr(expr)))
        }
        PartSelector::Single(AtomicPartSpec::Index(index)) => {
            let (_, value) = association_entry_by_index(entries, *index)?;
            Some(SelectedPartShape::Leaf(value))
        }
        PartSelector::Single(spec @ (AtomicPartSpec::All | AtomicPartSpec::Span(_))) => {
            Some(SelectedPartShape::AssociationBranch(
                select_association_entries_for_atomic_spec(entries, spec)?
                    .into_iter()
                    .map(|(key, value)| (key, SelectedPartShape::Leaf(value)))
                    .collect(),
            ))
        }
        PartSelector::Multi(specs) => {
            if specs
                .iter()
                .all(|spec| matches!(spec, PartSelector::Key(_)))
            {
                return Some(SelectedPartShape::AssociationBranch(
                    specs
                        .iter()
                        .filter_map(|spec| match spec {
                            PartSelector::Key(key) => Some((
                                key.clone(),
                                SelectedPartShape::Leaf(
                                    entries
                                        .iter()
                                        .find(|(entry_key, _)| entry_key == key)
                                        .map(|(_, value)| value.clone())
                                        .unwrap_or_else(|| {
                                            missing_key_absent_expr(key_expr(key.clone()))
                                        }),
                                ),
                            )),
                            _ => None,
                        })
                        .collect(),
                ));
            }

            Some(SelectedPartShape::AssociationBranch(
                specs
                    .iter()
                    .map(|spec| {
                        let PartSelector::Single(AtomicPartSpec::Index(index)) = spec else {
                            return None;
                        };
                        let (key, value) = association_entry_by_index(entries, *index)?;
                        Some((key, SelectedPartShape::Leaf(value)))
                    })
                    .collect::<Option<Vec<_>>>()?,
            ))
        }
    }
}

fn association_entry_by_index(entries: &[(Expr, Expr)], index: isize) -> Option<(Expr, Expr)> {
    if index == 0 {
        return None;
    }

    let normalized = normalize_structural_arg_position(index, entries.len())?;
    entries.get(normalized).cloned()
}

fn select_association_entries_for_atomic_spec(
    entries: &[(Expr, Expr)],
    spec: &AtomicPartSpec,
) -> Option<Vec<(Expr, Expr)>> {
    match spec {
        AtomicPartSpec::Index(index) => {
            association_entry_by_index(entries, *index).map(|entry| vec![entry])
        }
        AtomicPartSpec::All => Some(entries.to_vec()),
        AtomicPartSpec::Span(span) => {
            let components = resolve_span_components(*span, entries.len())?;
            components
                .into_iter()
                .map(|component| {
                    let index = normalize_structural_arg_position(component, entries.len())?;
                    entries.get(index).cloned()
                })
                .collect()
        }
    }
}

fn key_expr(key: Expr) -> Expr {
    Expr::call(system_expr("Key"), vec![key])
}

fn select_parts_for_atomic_spec(expr: &Expr, spec: &AtomicPartSpec) -> Option<Vec<Expr>> {
    select_positioned_parts_for_atomic_spec(expr, spec).map(|parts| {
        parts
            .into_iter()
            .map(|(_, value)| value)
            .collect::<Vec<_>>()
    })
}

fn select_positioned_parts_for_atomic_spec(
    expr: &Expr,
    spec: &AtomicPartSpec,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => Some(Vec::new()),
            Expr::Call { args, .. } => Some(
                args.iter()
                    .enumerate()
                    .map(|(index, value)| (index as isize + 1, value.clone()))
                    .collect(),
            ),
        },
        AtomicPartSpec::Span(span) => select_positioned_parts_for_span(expr, *span),
    }
}

pub(super) fn select_positioned_parts_for_span(
    expr: &Expr,
    spec: PartSpanSpec,
) -> Option<Vec<(isize, Expr)>> {
    match expr {
        Expr::Atom(_) => {
            resolve_span_components(spec, 0)?;
            Some(Vec::new())
        }
        Expr::Call { args, .. } => {
            let components = resolve_span_components(spec, args.len())?;
            Some(
                components
                    .into_iter()
                    .map(|component| {
                        let index = normalize_structural_arg_position(component, args.len())?;
                        Some((component, args[index].clone()))
                    })
                    .collect::<Option<Vec<_>>>()?,
            )
        }
    }
}

fn resolve_span_components(spec: PartSpanSpec, len: usize) -> Option<Vec<isize>> {
    if spec.step == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    if spec.step > 0 {
        let start = resolve_positive_span_start(spec.start, len)?;
        let end = resolve_positive_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    } else {
        let start = resolve_negative_span_start(spec.start, len)?;
        let end = resolve_negative_span_end(spec.end, len)?;
        resolve_ordered_span_components(start, end, spec.step)
    }
}

fn resolve_ordered_span_components(start: isize, end: isize, step: isize) -> Option<Vec<isize>> {
    let mut current = start;
    let mut components = Vec::new();

    if step > 0 {
        if start > end {
            return (start == end + 1).then_some(Vec::new());
        }
        while current <= end {
            components.push(current);
            current += step;
        }
    } else {
        if start < end {
            return (start + 1 == end).then_some(Vec::new());
        }
        while current >= end {
            components.push(current);
            current += step;
        }
    }

    Some(components)
}

fn resolve_positive_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_start_boundary(limit, len)),
        SpanEndpoint::Index(0) => Some(len + 1),
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_positive_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => Some(0),
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (0..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_start(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(1),
        SpanEndpoint::UpTo(Some(0)) => None,
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_negative_span_end(endpoint: SpanEndpoint, len: isize) -> Option<isize> {
    if len == 0 {
        return None;
    }

    match endpoint {
        SpanEndpoint::All => Some(len),
        SpanEndpoint::UpTo(Some(0)) => None,
        SpanEndpoint::UpTo(limit) => Some(resolve_upto_end_boundary(limit, len)),
        SpanEndpoint::Index(0) => None,
        SpanEndpoint::Index(index) if index > 0 => (index <= len + 1).then_some(index),
        SpanEndpoint::Index(index) => {
            let normalized = len + index + 1;
            (1..=len).contains(&normalized).then_some(normalized)
        }
    }
}

fn resolve_upto_start_boundary(limit: Option<isize>, len: isize) -> isize {
    match limit {
        None => len,
        Some(0) => len + 1,
        Some(limit) if len == 0 => limit,
        Some(limit) => limit.min(len),
    }
}

fn resolve_upto_end_boundary(limit: Option<isize>, len: isize) -> isize {
    match limit {
        None => len,
        Some(limit) => limit.min(len),
    }
}

fn parse_extract_position_spec(expr: &Expr) -> Option<ExtractPositionSpec> {
    if is_empty_plain_list_expr(expr) {
        return Some(ExtractPositionSpec {
            position_specs: Vec::new(),
            multiple: true,
        });
    }

    if let Some(position_spec) = parse_single_position_spec_with_nested_lists(expr, false) {
        return Some(ExtractPositionSpec {
            position_specs: vec![position_spec],
            multiple: false,
        });
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let position_specs = args
        .iter()
        .map(|arg| parse_single_position_spec_with_nested_lists(arg, true))
        .collect::<Option<Vec<_>>>()?;
    Some(ExtractPositionSpec {
        position_specs,
        multiple: true,
    })
}

pub(super) fn parse_single_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    parse_single_position_spec_with_nested_lists(expr, false)
}

fn parse_single_position_spec_with_nested_lists(
    expr: &Expr,
    allow_nested_plain_lists: bool,
) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(vec![selector]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if !allow_nested_plain_lists && args.iter().all(is_plain_list_expr) {
        return None;
    }

    let parse_selector = if allow_nested_plain_lists {
        parse_extract_position_selector
    } else {
        parse_part_selector
    };

    args.iter().map(parse_selector).collect()
}

fn parse_extract_position_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_list_item_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn wrap_extracted_value(value: Expr, wrapper: Option<&Expr>) -> Expr {
    wrapper
        .cloned()
        .map(|head| Expr::call(head, vec![value.clone()]))
        .unwrap_or(value)
}

fn extract_operator_form(position_spec: Expr) -> Expr {
    let slot = Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1))]);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr("Extract"),
            vec![slot, position_spec],
        )],
    )
}
