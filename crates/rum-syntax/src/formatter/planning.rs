use super::{
    trivia::{
        binary_chain_child_matches, collect_binary_chain_rhs_start_indices,
        leading_trivia_has_newline,
    },
    *,
};

#[derive(Clone, Debug)]
pub(super) enum GroupLayout {
    Container {
        item_start_indices: Vec<usize>,
        closing_index: usize,
    },
    BinaryChain {
        rhs_start_indices: Vec<usize>,
    },
}

impl GroupLayout {
    pub(super) fn break_before_token(&self, token_index: usize) -> bool {
        match self {
            Self::Container {
                item_start_indices,
                closing_index,
            } => item_start_indices.contains(&token_index) || *closing_index == token_index,
            Self::BinaryChain { rhs_start_indices } => rhs_start_indices.contains(&token_index),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct GroupPlan {
    pub(super) start_index: usize,
    pub(super) end_index: usize,
    pub(super) layout: GroupLayout,
    pub(super) force_break: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GroupIdentity {
    pub(super) start_index: usize,
    pub(super) end_index: usize,
}

pub(super) struct FormattingContext<'a> {
    pub(super) parsed: &'a ParsedSource,
    pub(super) group_plans: BTreeMap<usize, Vec<GroupPlan>>,
}

pub(super) fn build_formatting_context(parsed: &ParsedSource) -> FormattingContext<'_> {
    let mut token_index_by_start = BTreeMap::new();
    let mut token_index_by_end = BTreeMap::new();

    for (index, token) in parsed.lexed().tokens().iter().enumerate() {
        token_index_by_start.insert(token.span.start, index);
        token_index_by_end.entry(token.span.end).or_insert(index);
    }

    let mut group_plans = BTreeMap::new();
    collect_container_group_plans(
        parsed,
        parsed.syntax(),
        &token_index_by_start,
        &token_index_by_end,
        &mut group_plans,
    );
    collect_binary_group_plans(
        parsed,
        parsed.syntax(),
        &token_index_by_start,
        &token_index_by_end,
        None,
        &mut group_plans,
    );

    FormattingContext {
        parsed,
        group_plans,
    }
}

fn collect_container_group_plans(
    parsed: &ParsedSource,
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    plans: &mut BTreeMap<usize, Vec<GroupPlan>>,
) {
    match &expr.kind {
        SyntaxExprKind::List(items) => {
            for item in items {
                collect_container_group_plans(
                    parsed,
                    item,
                    token_index_by_start,
                    token_index_by_end,
                    plans,
                );
            }

            maybe_collect_container_group_plan(
                parsed,
                expr,
                items,
                expr.span.end,
                SyntaxTokenKind::RBrace,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Association(items) => {
            for item in items {
                collect_container_group_plans(
                    parsed,
                    item,
                    token_index_by_start,
                    token_index_by_end,
                    plans,
                );
            }

            maybe_collect_container_group_plan(
                parsed,
                expr,
                items,
                expr.span.end,
                SyntaxTokenKind::RAssociation,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Pattern { name, pattern } => {
            collect_container_group_plans(
                parsed,
                name,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
            collect_container_group_plans(
                parsed,
                pattern,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Prefix { expr, .. } | SyntaxExprKind::Postfix { expr, .. } => {
            collect_container_group_plans(
                parsed,
                expr,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Binary { lhs, rhs, .. } => {
            collect_container_group_plans(
                parsed,
                lhs,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
            collect_container_group_plans(
                parsed,
                rhs,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Call { head, args } => {
            collect_container_group_plans(
                parsed,
                head,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
            for arg in args {
                collect_container_group_plans(
                    parsed,
                    arg,
                    token_index_by_start,
                    token_index_by_end,
                    plans,
                );
            }

            maybe_collect_container_group_plan(
                parsed,
                expr,
                args,
                expr.span.end,
                SyntaxTokenKind::RBracket,
                token_index_by_start,
                token_index_by_end,
                plans,
            );
        }
        SyntaxExprKind::Blank { head, .. } => {
            if let Some(head) = head {
                collect_container_group_plans(
                    parsed,
                    head,
                    token_index_by_start,
                    token_index_by_end,
                    plans,
                );
            }
        }
        SyntaxExprKind::Integer(_)
        | SyntaxExprKind::String(_)
        | SyntaxExprKind::Symbol(_)
        | SyntaxExprKind::Slot(_) => {}
    }
}

fn maybe_collect_container_group_plan(
    parsed: &ParsedSource,
    container: &SyntaxExpr,
    items: &[SyntaxExpr],
    closing_offset: usize,
    closing_kind: SyntaxTokenKind,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    plans: &mut BTreeMap<usize, Vec<GroupPlan>>,
) {
    let Some(start_index) = token_index_by_start.get(&container.span.start).copied() else {
        return;
    };

    let Some(closing_index) = token_index_by_end.get(&closing_offset).copied() else {
        return;
    };

    if parsed.lexed().tokens()[closing_index].kind != closing_kind {
        return;
    }

    let item_start_indices = items
        .iter()
        .filter_map(|item| token_index_by_start.get(&item.span.start).copied())
        .collect::<Vec<_>>();

    let force_break = should_force_break_container(parsed, &item_start_indices, closing_index);

    if !force_break && item_start_indices.len() < 2 {
        return;
    }

    insert_group_plan(
        plans,
        GroupPlan {
            start_index,
            end_index: closing_index,
            layout: GroupLayout::Container {
                item_start_indices,
                closing_index,
            },
            force_break,
        },
    );
}

fn collect_binary_group_plans(
    parsed: &ParsedSource,
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    parent_binary_op: Option<crate::BinaryOp>,
    plans: &mut BTreeMap<usize, Vec<GroupPlan>>,
) {
    match &expr.kind {
        SyntaxExprKind::Pattern { name, pattern } => {
            collect_binary_group_plans(
                parsed,
                name,
                token_index_by_start,
                token_index_by_end,
                None,
                plans,
            );
            collect_binary_group_plans(
                parsed,
                pattern,
                token_index_by_start,
                token_index_by_end,
                None,
                plans,
            );
        }
        SyntaxExprKind::Prefix { expr, .. } | SyntaxExprKind::Postfix { expr, .. } => {
            collect_binary_group_plans(
                parsed,
                expr,
                token_index_by_start,
                token_index_by_end,
                None,
                plans,
            );
        }
        SyntaxExprKind::Binary { op, lhs, rhs } => {
            collect_binary_group_plans(
                parsed,
                lhs,
                token_index_by_start,
                token_index_by_end,
                Some(*op),
                plans,
            );
            collect_binary_group_plans(
                parsed,
                rhs,
                token_index_by_start,
                token_index_by_end,
                Some(*op),
                plans,
            );
            if !parent_binary_op
                .is_some_and(|parent_op| binary_chain_child_matches(expr, parent_op))
            {
                maybe_collect_binary_group_plan(
                    parsed,
                    expr,
                    token_index_by_start,
                    token_index_by_end,
                    plans,
                );
            }
        }
        SyntaxExprKind::Call { head, args } => {
            collect_binary_group_plans(
                parsed,
                head,
                token_index_by_start,
                token_index_by_end,
                None,
                plans,
            );
            for arg in args {
                collect_binary_group_plans(
                    parsed,
                    arg,
                    token_index_by_start,
                    token_index_by_end,
                    None,
                    plans,
                );
            }
        }
        SyntaxExprKind::Association(items) | SyntaxExprKind::List(items) => {
            for item in items {
                collect_binary_group_plans(
                    parsed,
                    item,
                    token_index_by_start,
                    token_index_by_end,
                    None,
                    plans,
                );
            }
        }
        SyntaxExprKind::Blank { head, .. } => {
            if let Some(head) = head {
                collect_binary_group_plans(
                    parsed,
                    head,
                    token_index_by_start,
                    token_index_by_end,
                    None,
                    plans,
                );
            }
        }
        SyntaxExprKind::Integer(_)
        | SyntaxExprKind::String(_)
        | SyntaxExprKind::Symbol(_)
        | SyntaxExprKind::Slot(_) => {}
    }
}

fn maybe_collect_binary_group_plan(
    parsed: &ParsedSource,
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    plans: &mut BTreeMap<usize, Vec<GroupPlan>>,
) {
    let Some(start_index) = token_index_by_start.get(&expr.span.start).copied() else {
        return;
    };

    let Some(end_index) = token_index_by_end.get(&expr.span.end).copied() else {
        return;
    };

    let rhs_start_indices = collect_binary_chain_rhs_start_indices(expr, token_index_by_start);
    if rhs_start_indices.is_empty() {
        return;
    }

    let force_break = rhs_start_indices
        .iter()
        .copied()
        .any(|rhs_start_index| leading_trivia_has_newline(parsed, rhs_start_index));

    insert_group_plan(
        plans,
        GroupPlan {
            start_index,
            end_index,
            layout: GroupLayout::BinaryChain { rhs_start_indices },
            force_break,
        },
    );
}

fn insert_group_plan(plans: &mut BTreeMap<usize, Vec<GroupPlan>>, plan: GroupPlan) {
    plans.entry(plan.start_index).or_default().push(plan);
}

fn should_force_break_container(
    parsed: &ParsedSource,
    item_start_indices: &[usize],
    closing_index: usize,
) -> bool {
    if item_start_indices.len() >= ACTIVE_MULTILINE_ITEM_THRESHOLD {
        return true;
    }

    item_start_indices
        .iter()
        .copied()
        .chain(std::iter::once(closing_index))
        .any(|index| leading_trivia_has_newline(parsed, index))
}
