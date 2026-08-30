use super::{
    doc::{Doc, DocBuilder},
    planning::{FormattingContext, GroupIdentity, GroupLayout, GroupPlan},
    render::flat_doc_width,
    *,
};

fn canonical_token_separator(
    before_previous_kind: Option<&SyntaxTokenKind>,
    previous_kind: Option<&SyntaxTokenKind>,
    current_kind: &SyntaxTokenKind,
) -> String {
    previous_kind
        .is_some_and(|previous| needs_space_between(before_previous_kind, previous, current_kind))
        .then_some(" ".to_owned())
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, Default)]
struct TriviaRenderState {
    inserted_separator: bool,
    ended_with_newline: bool,
}

pub(super) fn build_document(
    context: &FormattingContext<'_>,
    start_index: usize,
    end_index: usize,
    starting_nesting_depth: usize,
    skip_first_leading_trivia: bool,
    layout: Option<&GroupLayout>,
    suppressed_group: Option<GroupIdentity>,
) -> Doc {
    let mut document = DocBuilder::new();
    let tokens = context.parsed.lexed().tokens();
    let mut before_previous_kind: Option<SyntaxTokenKind> = None;
    let mut previous_kind: Option<SyntaxTokenKind> = None;
    let mut nesting_depth = starting_nesting_depth;
    let mut index = start_index;

    while index <= end_index {
        let token = &tokens[index];

        if matches!(token.kind, SyntaxTokenKind::End) {
            render_end_trivia(context.parsed, index, &mut document);
            break;
        }

        if let Some(plan) = select_group_plan(context, index, end_index, suppressed_group) {
            let trivia_state = render_token_leading_trivia(
                context,
                &mut document,
                start_index,
                index,
                before_previous_kind.as_ref(),
                previous_kind.as_ref(),
                &token.kind,
                nesting_depth,
                skip_first_leading_trivia,
                layout,
            );

            if !trivia_state.inserted_separator
                && !trivia_state.ended_with_newline
                && previous_kind.as_ref().is_some_and(|previous| {
                    needs_space_between(before_previous_kind.as_ref(), previous, &token.kind)
                })
            {
                document.push_char(' ');
            }

            document.push_doc(build_group_doc(context, plan, nesting_depth));
            before_previous_kind = previous_kind;
            previous_kind = Some(tokens[plan.end_index].kind.clone());
            index = plan.end_index.saturating_add(1);
            continue;
        }

        let trivia_state = render_token_leading_trivia(
            context,
            &mut document,
            start_index,
            index,
            before_previous_kind.as_ref(),
            previous_kind.as_ref(),
            &token.kind,
            nesting_depth,
            skip_first_leading_trivia,
            layout,
        );

        if !trivia_state.inserted_separator
            && !trivia_state.ended_with_newline
            && previous_kind.as_ref().is_some_and(|previous| {
                needs_space_between(before_previous_kind.as_ref(), previous, &token.kind)
            })
        {
            document.push_char(' ');
        }

        if let Some(text) = context.parsed.text(token.span) {
            document.push_text(text);
        }

        before_previous_kind = previous_kind;
        previous_kind = Some(token.kind.clone());
        update_nesting_depth(&mut nesting_depth, &token.kind);
        index = index.saturating_add(1);
    }

    document.finish()
}

fn build_group_doc(context: &FormattingContext<'_>, plan: &GroupPlan, nesting_depth: usize) -> Doc {
    let group_identity = Some(GroupIdentity {
        start_index: plan.start_index,
        end_index: plan.end_index,
    });
    let doc = build_document(
        context,
        plan.start_index,
        plan.end_index,
        nesting_depth,
        true,
        Some(&plan.layout),
        group_identity,
    );
    let flat_width = flat_doc_width(&doc);

    Doc::Group {
        doc: Box::new(doc),
        force_break: plan.force_break,
        flat_width,
    }
}

fn select_group_plan<'a>(
    context: &'a FormattingContext<'_>,
    start_index: usize,
    range_end_index: usize,
    suppressed_group: Option<GroupIdentity>,
) -> Option<&'a GroupPlan> {
    context.group_plans.get(&start_index).and_then(|plans| {
        plans
            .iter()
            .filter(|plan| {
                plan.end_index <= range_end_index
                    && Some(GroupIdentity {
                        start_index: plan.start_index,
                        end_index: plan.end_index,
                    }) != suppressed_group
            })
            .max_by_key(|plan| plan.end_index)
    })
}

fn render_token_leading_trivia(
    context: &FormattingContext<'_>,
    document: &mut DocBuilder,
    range_start_index: usize,
    token_index: usize,
    before_previous_kind: Option<&SyntaxTokenKind>,
    previous_kind: Option<&SyntaxTokenKind>,
    current_kind: &SyntaxTokenKind,
    nesting_depth: usize,
    skip_first_leading_trivia: bool,
    layout: Option<&GroupLayout>,
) -> TriviaRenderState {
    if skip_first_leading_trivia && token_index == range_start_index {
        return TriviaRenderState::default();
    }

    let force_newline =
        layout.is_some_and(|group_layout| group_layout.break_before_token(token_index));

    render_leading_trivia(
        context.parsed,
        token_index,
        before_previous_kind,
        previous_kind,
        current_kind,
        nesting_depth,
        force_newline,
        document,
    )
}

pub(super) fn collect_binary_chain_rhs_start_indices(
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
) -> Vec<usize> {
    match &expr.kind {
        SyntaxExprKind::ComparisonChain { operands, .. } => operands
            .iter()
            .skip(1)
            .filter_map(|operand| token_index_by_start.get(&operand.span.start).copied())
            .collect(),
        SyntaxExprKind::Binary { op, lhs, rhs } => {
            let mut indices = Vec::new();

            if op.is_right_associative() {
                if binary_chain_child_matches(rhs, *op) {
                    indices.extend(collect_binary_chain_rhs_start_indices(
                        rhs,
                        token_index_by_start,
                    ));
                }

                if let Some(rhs_start_index) = token_index_by_start.get(&rhs.span.start).copied() {
                    indices.insert(0, rhs_start_index);
                }
            } else {
                if binary_chain_child_matches(lhs, *op) {
                    indices.extend(collect_binary_chain_rhs_start_indices(
                        lhs,
                        token_index_by_start,
                    ));
                }

                if let Some(rhs_start_index) = token_index_by_start.get(&rhs.span.start).copied() {
                    indices.push(rhs_start_index);
                }
            }

            indices
        }
        _ => Vec::new(),
    }
}

pub(super) fn binary_chain_child_matches(expr: &SyntaxExpr, parent_op: crate::BinaryOp) -> bool {
    matches!(
        &expr.kind,
        SyntaxExprKind::Binary { op, .. }
            if op.precedence() == parent_op.precedence()
                && op.is_right_associative() == parent_op.is_right_associative()
    )
}

pub(super) fn leading_trivia_has_newline(parsed: &ParsedSource, token_index: usize) -> bool {
    parsed
        .lexed()
        .leading_trivia(token_index)
        .into_iter()
        .flatten()
        .any(|piece| matches!(piece.kind, TriviaKind::Whitespace) && piece.text.contains('\n'))
}

fn render_end_trivia(parsed: &ParsedSource, token_index: usize, output: &mut DocBuilder) {
    let Some(trivia) = parsed.lexed().leading_trivia(token_index) else {
        return;
    };

    let mut emitted_any = false;

    for piece in trivia {
        match piece.kind {
            TriviaKind::Whitespace => {
                if piece.text.contains('\n') {
                    if emitted_any && !output.ends_with_newline() {
                        output.push_break(" ".to_owned(), 0);
                    }
                } else if emitted_any && !output.ends_with_whitespace() {
                    output.push_char(' ');
                }
            }
            TriviaKind::Comment => {
                if !output.is_empty() && !output.ends_with_whitespace() {
                    output.push_char(' ');
                }
                output.push_text(&piece.text);
                emitted_any = true;
            }
        }
    }
}

fn render_leading_trivia(
    parsed: &ParsedSource,
    token_index: usize,
    before_previous_kind: Option<&SyntaxTokenKind>,
    previous_kind: Option<&SyntaxTokenKind>,
    current_kind: &SyntaxTokenKind,
    nesting_depth: usize,
    force_newline: bool,
    output: &mut DocBuilder,
) -> TriviaRenderState {
    let Some(trivia) = parsed.lexed().leading_trivia(token_index) else {
        return TriviaRenderState::default();
    };

    let mut has_newline = force_newline;
    let mut comments = Vec::new();

    for piece in trivia {
        match piece.kind {
            TriviaKind::Whitespace => {
                has_newline |= piece.text.contains('\n');
            }
            TriviaKind::Comment => comments.push(piece.text.as_str()),
        }
    }

    if comments.is_empty() {
        if has_newline {
            if !output.is_empty() {
                output.push_break(
                    canonical_token_separator(before_previous_kind, previous_kind, current_kind),
                    indentation_level_for_token(nesting_depth, current_kind),
                );
            }

            return TriviaRenderState {
                ended_with_newline: true,
                ..TriviaRenderState::default()
            };
        }

        return TriviaRenderState::default();
    }

    if has_newline {
        let comment_indentation_level = nesting_depth;
        let token_indentation_level = indentation_level_for_token(nesting_depth, current_kind);

        if !output.is_empty() && !output.ends_with_newline() {
            let flat = needs_space_before_inline_comment(previous_kind)
                .then_some(" ".to_owned())
                .unwrap_or_default();
            output.push_break(flat, comment_indentation_level);
        }

        for (index, comment) in comments.iter().enumerate() {
            if index > 0 {
                output.push_break(" ".to_owned(), comment_indentation_level);
            }
            output.push_text(comment);
        }

        if !output.ends_with_newline() {
            let flat = needs_space_after_inline_comment(current_kind)
                .then_some(" ".to_owned())
                .unwrap_or_default();
            output.push_break(flat, token_indentation_level);
        }

        TriviaRenderState {
            ended_with_newline: true,
            ..TriviaRenderState::default()
        }
    } else {
        if !output.is_empty() && needs_space_before_inline_comment(previous_kind) {
            if !output.ends_with_whitespace() {
                output.push_char(' ');
            }
        }

        for (index, comment) in comments.iter().enumerate() {
            if index > 0 {
                output.push_char(' ');
            }
            output.push_text(comment);
        }

        let inserted_separator = needs_space_after_inline_comment(current_kind);
        if inserted_separator && !output.ends_with_whitespace() {
            output.push_char(' ');
        }

        TriviaRenderState {
            inserted_separator,
            ..TriviaRenderState::default()
        }
    }
}

fn indentation_level_for_token(nesting_depth: usize, kind: &SyntaxTokenKind) -> usize {
    if is_closing_delimiter(kind) {
        nesting_depth.saturating_sub(1)
    } else {
        nesting_depth
    }
}

fn update_nesting_depth(nesting_depth: &mut usize, kind: &SyntaxTokenKind) {
    if is_opening_delimiter(kind) {
        *nesting_depth += 1;
    } else if is_closing_delimiter(kind) {
        *nesting_depth = nesting_depth.saturating_sub(1);
    }
}

fn is_opening_delimiter(kind: &SyntaxTokenKind) -> bool {
    matches!(
        kind,
        SyntaxTokenKind::LBracket
            | SyntaxTokenKind::LBrace
            | SyntaxTokenKind::LAssociation
            | SyntaxTokenKind::LParen
    )
}

fn is_closing_delimiter(kind: &SyntaxTokenKind) -> bool {
    matches!(
        kind,
        SyntaxTokenKind::RBracket
            | SyntaxTokenKind::RBrace
            | SyntaxTokenKind::RAssociation
            | SyntaxTokenKind::RParen
    )
}

fn needs_space_before_inline_comment(previous_kind: Option<&SyntaxTokenKind>) -> bool {
    !matches!(
        previous_kind,
        None | Some(
            SyntaxTokenKind::LBracket
                | SyntaxTokenKind::LBrace
                | SyntaxTokenKind::LAssociation
                | SyntaxTokenKind::LParen
                | SyntaxTokenKind::MessageName
                | SyntaxTokenKind::Hash
                | SyntaxTokenKind::Underscore
                | SyntaxTokenKind::DoubleUnderscore
                | SyntaxTokenKind::TripleUnderscore
        )
    )
}

fn needs_space_after_inline_comment(current_kind: &SyntaxTokenKind) -> bool {
    !matches!(
        current_kind,
        SyntaxTokenKind::RBracket
            | SyntaxTokenKind::RBrace
            | SyntaxTokenKind::RAssociation
            | SyntaxTokenKind::RParen
            | SyntaxTokenKind::Semicolon
            | SyntaxTokenKind::Comma
            | SyntaxTokenKind::MessageName
            | SyntaxTokenKind::Underscore
            | SyntaxTokenKind::DoubleUnderscore
            | SyntaxTokenKind::TripleUnderscore
    )
}

fn needs_space_between(
    before_previous: Option<&SyntaxTokenKind>,
    previous: &SyntaxTokenKind,
    current: &SyntaxTokenKind,
) -> bool {
    if matches!(current, SyntaxTokenKind::MessageName)
        || matches!(previous, SyntaxTokenKind::MessageName)
    {
        return false;
    }

    if matches!(
        current,
        SyntaxTokenKind::LBracket
            | SyntaxTokenKind::RBracket
            | SyntaxTokenKind::RBrace
            | SyntaxTokenKind::RAssociation
            | SyntaxTokenKind::RParen
            | SyntaxTokenKind::Semicolon
            | SyntaxTokenKind::Comma
            | SyntaxTokenKind::Underscore
            | SyntaxTokenKind::DoubleUnderscore
            | SyntaxTokenKind::TripleUnderscore
    ) {
        return false;
    }

    if matches!(
        previous,
        SyntaxTokenKind::LBracket
            | SyntaxTokenKind::LBrace
            | SyntaxTokenKind::LAssociation
            | SyntaxTokenKind::LParen
            | SyntaxTokenKind::Underscore
            | SyntaxTokenKind::DoubleUnderscore
            | SyntaxTokenKind::TripleUnderscore
    ) {
        return false;
    }

    if matches!(previous, SyntaxTokenKind::Hash)
        && matches!(
            current,
            SyntaxTokenKind::Hash | SyntaxTokenKind::Integer(_) | SyntaxTokenKind::Symbol(_)
        )
    {
        return false;
    }

    if matches!(previous, SyntaxTokenKind::Hash) {
        return true;
    }

    if matches!(current, SyntaxTokenKind::Hash) {
        return false;
    }

    if matches!(current, SyntaxTokenKind::Plus | SyntaxTokenKind::Minus)
        && is_prefix_position(previous)
    {
        return false;
    }

    if matches!(previous, SyntaxTokenKind::Plus | SyntaxTokenKind::Minus)
        && is_prefix_operator(before_previous, previous, current)
    {
        return false;
    }

    true
}

fn is_prefix_position(previous: &SyntaxTokenKind) -> bool {
    matches!(
        previous,
        SyntaxTokenKind::LBracket
            | SyntaxTokenKind::LBrace
            | SyntaxTokenKind::LAssociation
            | SyntaxTokenKind::LParen
            | SyntaxTokenKind::Comma
            | SyntaxTokenKind::Set
            | SyntaxTokenKind::SetDelayed
            | SyntaxTokenKind::Arrow
            | SyntaxTokenKind::RuleDelayed
            | SyntaxTokenKind::ReplaceAll
            | SyntaxTokenKind::ReplaceRepeated
            | SyntaxTokenKind::Condition
            | SyntaxTokenKind::TagSet
            | SyntaxTokenKind::SlashAt
            | SyntaxTokenKind::Star
            | SyntaxTokenKind::Slash
            | SyntaxTokenKind::Caret
    )
}

fn is_prefix_operator(
    before_previous: Option<&SyntaxTokenKind>,
    previous: &SyntaxTokenKind,
    current: &SyntaxTokenKind,
) -> bool {
    matches!(previous, SyntaxTokenKind::Plus | SyntaxTokenKind::Minus)
        && before_previous.is_none_or(is_prefix_position)
        && matches!(
            current,
            SyntaxTokenKind::Integer(_)
                | SyntaxTokenKind::String(_)
                | SyntaxTokenKind::Symbol(_)
                | SyntaxTokenKind::Hash
                | SyntaxTokenKind::Underscore
                | SyntaxTokenKind::DoubleUnderscore
                | SyntaxTokenKind::TripleUnderscore
                | SyntaxTokenKind::LParen
                | SyntaxTokenKind::LBrace
                | SyntaxTokenKind::LAssociation
        )
}
