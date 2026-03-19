use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ParsedSource, SyntaxError, SyntaxExpr, SyntaxExprKind, SyntaxTokenKind, TriviaKind,
    parse_with_diagnostics,
};

const INDENT_WIDTH: usize = 2;
const ACTIVE_MULTILINE_ITEM_THRESHOLD: usize = 4;
const DEFAULT_LINE_WIDTH: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatterOptions {
    pub trailing_newline: bool,
    pub line_width: usize,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            trailing_newline: false,
            line_width: DEFAULT_LINE_WIDTH,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TriviaRenderState {
    inserted_separator: bool,
    ended_with_newline: bool,
}

#[derive(Debug, Default)]
struct LayoutHints {
    newline_before_tokens: BTreeSet<usize>,
}

pub fn format_source(source: &str) -> Result<String, SyntaxError> {
    let parsed = parse_with_diagnostics(source).map_err(|diagnostic| diagnostic.into_error())?;
    Ok(format_parsed_source(&parsed, FormatterOptions::default()))
}

pub fn format_parsed_source(parsed: &ParsedSource, options: FormatterOptions) -> String {
    let mut output = String::new();
    let tokens = parsed.lexed().tokens();
    let layout_hints = collect_layout_hints(parsed, options);
    let mut before_previous_kind: Option<SyntaxTokenKind> = None;
    let mut previous_kind: Option<SyntaxTokenKind> = None;
    let mut nesting_depth = 0usize;

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, SyntaxTokenKind::End) {
            render_end_trivia(parsed, index, &mut output);
            break;
        }

        let comment_indentation_level = nesting_depth;
        let indentation_level = indentation_level_for_token(nesting_depth, &token.kind);
        let force_newline = layout_hints.newline_before_tokens.contains(&index);
        let trivia_state = render_leading_trivia(
            parsed,
            index,
            previous_kind.as_ref(),
            &token.kind,
            comment_indentation_level,
            force_newline,
            &mut output,
        );

        if output.ends_with('\n') {
            write_indentation(&mut output, indentation_level);
        }

        if !trivia_state.inserted_separator
            && !trivia_state.ended_with_newline
            && previous_kind.as_ref().is_some_and(|previous| {
                needs_space_between(before_previous_kind.as_ref(), previous, &token.kind)
            })
        {
            output.push(' ');
        }

        if let Some(text) = parsed.text(token.span) {
            output.push_str(text);
        }

        before_previous_kind = previous_kind;
        previous_kind = Some(token.kind.clone());
        update_nesting_depth(&mut nesting_depth, &token.kind);
    }

    if options.trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

fn collect_layout_hints(parsed: &ParsedSource, options: FormatterOptions) -> LayoutHints {
    let mut token_index_by_start = BTreeMap::new();
    let mut token_index_by_end = BTreeMap::new();

    for (index, token) in parsed.lexed().tokens().iter().enumerate() {
        token_index_by_start.insert(token.span.start, index);
        token_index_by_end.entry(token.span.end).or_insert(index);
    }

    let mut hints = LayoutHints::default();
    collect_layout_hints_from_expr(
        parsed,
        options,
        parsed.syntax(),
        &token_index_by_start,
        &token_index_by_end,
        &mut hints,
    );
    hints
}

fn collect_layout_hints_from_expr(
    parsed: &ParsedSource,
    options: FormatterOptions,
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    hints: &mut LayoutHints,
) {
    match &expr.kind {
        SyntaxExprKind::List(items) => {
            for item in items {
                collect_layout_hints_from_expr(
                    parsed,
                    options,
                    item,
                    token_index_by_start,
                    token_index_by_end,
                    hints,
                );
            }

            maybe_expand_container(
                parsed,
                options,
                expr,
                items,
                expr.span.end,
                SyntaxTokenKind::RBrace,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
        }
        SyntaxExprKind::Pattern { name, pattern } => {
            collect_layout_hints_from_expr(
                parsed,
                options,
                name,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
            collect_layout_hints_from_expr(
                parsed,
                options,
                pattern,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
        }
        SyntaxExprKind::Prefix { expr, .. } | SyntaxExprKind::Postfix { expr, .. } => {
            collect_layout_hints_from_expr(
                parsed,
                options,
                expr,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
        }
        SyntaxExprKind::Binary { lhs, rhs, .. } => {
            collect_layout_hints_from_expr(
                parsed,
                options,
                lhs,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
            collect_layout_hints_from_expr(
                parsed,
                options,
                rhs,
                token_index_by_start,
                token_index_by_end,
                hints,
            );

            maybe_expand_binary_expr(parsed, options, expr, token_index_by_start, hints);
        }
        SyntaxExprKind::Call { head, args } => {
            collect_layout_hints_from_expr(
                parsed,
                options,
                head,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
            for arg in args {
                collect_layout_hints_from_expr(
                    parsed,
                    options,
                    arg,
                    token_index_by_start,
                    token_index_by_end,
                    hints,
                );
            }

            maybe_expand_container(
                parsed,
                options,
                expr,
                args,
                expr.span.end,
                SyntaxTokenKind::RBracket,
                token_index_by_start,
                token_index_by_end,
                hints,
            );
        }
        SyntaxExprKind::Blank { head, .. } => {
            if let Some(head) = head {
                collect_layout_hints_from_expr(
                    parsed,
                    options,
                    head,
                    token_index_by_start,
                    token_index_by_end,
                    hints,
                );
            }
        }
        SyntaxExprKind::Integer(_)
        | SyntaxExprKind::String(_)
        | SyntaxExprKind::Symbol(_)
        | SyntaxExprKind::Slot(_) => {}
    }
}

fn maybe_expand_container(
    parsed: &ParsedSource,
    options: FormatterOptions,
    container: &SyntaxExpr,
    items: &[SyntaxExpr],
    closing_offset: usize,
    closing_kind: SyntaxTokenKind,
    token_index_by_start: &BTreeMap<usize, usize>,
    token_index_by_end: &BTreeMap<usize, usize>,
    hints: &mut LayoutHints,
) {
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

    if !should_expand_container(
        parsed,
        options,
        container,
        &item_start_indices,
        closing_index,
    ) {
        return;
    }

    for index in item_start_indices {
        hints.newline_before_tokens.insert(index);
    }

    hints.newline_before_tokens.insert(closing_index);
}

fn should_expand_container(
    parsed: &ParsedSource,
    options: FormatterOptions,
    container: &SyntaxExpr,
    item_start_indices: &[usize],
    closing_index: usize,
) -> bool {
    if item_start_indices.len() >= ACTIVE_MULTILINE_ITEM_THRESHOLD {
        return true;
    }

    if item_start_indices.len() >= 2 && container.to_string().len() > options.line_width {
        return true;
    }

    item_start_indices
        .iter()
        .copied()
        .chain(std::iter::once(closing_index))
        .any(|index| leading_trivia_has_newline(parsed, index))
}

fn maybe_expand_binary_expr(
    parsed: &ParsedSource,
    options: FormatterOptions,
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
    hints: &mut LayoutHints,
) {
    let rhs_start_indices = collect_binary_chain_rhs_start_indices(expr, token_index_by_start);

    if should_expand_binary_expr(parsed, options, expr, &rhs_start_indices) {
        for rhs_start_index in rhs_start_indices {
            hints.newline_before_tokens.insert(rhs_start_index);
        }
    }
}

fn collect_binary_chain_rhs_start_indices(
    expr: &SyntaxExpr,
    token_index_by_start: &BTreeMap<usize, usize>,
) -> Vec<usize> {
    let SyntaxExprKind::Binary { op, lhs, rhs } = &expr.kind else {
        return Vec::new();
    };

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

fn binary_chain_child_matches(expr: &SyntaxExpr, parent_op: crate::BinaryOp) -> bool {
    matches!(
        &expr.kind,
        SyntaxExprKind::Binary { op, .. }
            if op.precedence() == parent_op.precedence()
                && op.is_right_associative() == parent_op.is_right_associative()
    )
}

fn should_expand_binary_expr(
    parsed: &ParsedSource,
    options: FormatterOptions,
    expr: &SyntaxExpr,
    rhs_start_indices: &[usize],
) -> bool {
    expr.to_string().len() > options.line_width
        || rhs_start_indices
            .iter()
            .copied()
            .any(|rhs_start_index| leading_trivia_has_newline(parsed, rhs_start_index))
}

fn leading_trivia_has_newline(parsed: &ParsedSource, token_index: usize) -> bool {
    parsed
        .lexed()
        .leading_trivia(token_index)
        .into_iter()
        .flatten()
        .any(|piece| matches!(piece.kind, TriviaKind::Whitespace) && piece.text.contains('\n'))
}

fn render_end_trivia(parsed: &ParsedSource, token_index: usize, output: &mut String) {
    let Some(trivia) = parsed.lexed().leading_trivia(token_index) else {
        return;
    };

    let mut emitted_any = false;

    for piece in trivia {
        match piece.kind {
            TriviaKind::Whitespace => {
                if piece.text.contains('\n') {
                    if emitted_any && !output.ends_with('\n') {
                        output.push('\n');
                    }
                } else if emitted_any && !output.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
            }
            TriviaKind::Comment => {
                if !output.is_empty() && !output.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
                output.push_str(&piece.text);
                emitted_any = true;
            }
        }
    }
}

fn render_leading_trivia(
    parsed: &ParsedSource,
    token_index: usize,
    previous_kind: Option<&SyntaxTokenKind>,
    current_kind: &SyntaxTokenKind,
    indentation_level: usize,
    force_newline: bool,
    output: &mut String,
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
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }

            return TriviaRenderState {
                ended_with_newline: true,
                ..TriviaRenderState::default()
            };
        }

        return TriviaRenderState::default();
    }

    if has_newline {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }

        for (index, comment) in comments.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            write_indentation(output, indentation_level);
            output.push_str(comment);
        }

        if !output.ends_with('\n') {
            output.push('\n');
        }

        TriviaRenderState {
            ended_with_newline: true,
            ..TriviaRenderState::default()
        }
    } else {
        if !output.is_empty() && needs_space_before_inline_comment(previous_kind) {
            if !output.ends_with(char::is_whitespace) {
                output.push(' ');
            }
        }

        for (index, comment) in comments.iter().enumerate() {
            if index > 0 {
                output.push(' ');
            }
            output.push_str(comment);
        }

        let inserted_separator = needs_space_after_inline_comment(current_kind);
        if inserted_separator && !output.ends_with(char::is_whitespace) {
            output.push(' ');
        }

        TriviaRenderState {
            inserted_separator,
            ..TriviaRenderState::default()
        }
    }
}

fn write_indentation(output: &mut String, indentation_level: usize) {
    for _ in 0..indentation_level.saturating_mul(INDENT_WIDTH) {
        output.push(' ');
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
        SyntaxTokenKind::LBracket | SyntaxTokenKind::LBrace | SyntaxTokenKind::LParen
    )
}

fn is_closing_delimiter(kind: &SyntaxTokenKind) -> bool {
    matches!(
        kind,
        SyntaxTokenKind::RBracket | SyntaxTokenKind::RBrace | SyntaxTokenKind::RParen
    )
}

fn needs_space_before_inline_comment(previous_kind: Option<&SyntaxTokenKind>) -> bool {
    !matches!(
        previous_kind,
        None | Some(
            SyntaxTokenKind::LBracket
                | SyntaxTokenKind::LBrace
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
            | SyntaxTokenKind::RParen
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
            | SyntaxTokenKind::RParen
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
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_with_options(source: &str, options: FormatterOptions) -> String {
        let parsed =
            parse_with_diagnostics(source).expect("source should parse for formatter test");
        format_parsed_source(&parsed, options)
    }

    #[test]
    fn formatter_canonicalizes_core_spacing() {
        let formatted = format_source("f [ x_Integer,  y]&").unwrap();
        assert_eq!(formatted, "f[x_Integer, y] &");
    }

    #[test]
    fn formatter_preserves_plain_multiline_call_arguments() {
        let formatted = format_source("f[x,\ny]").unwrap();
        assert_eq!(formatted, "f[\n  x,\n  y\n]");
    }

    #[test]
    fn formatter_preserves_plain_multiline_nested_call_arguments() {
        let formatted = format_source("f[g[x,\ny],\nz]").unwrap();
        assert_eq!(formatted, "f[\n  g[\n    x,\n    y\n  ],\n  z\n]");
    }

    #[test]
    fn formatter_preserves_plain_multiline_list_elements() {
        let formatted = format_source("f[{a,\nb},\nc]").unwrap();
        assert_eq!(formatted, "f[\n  {\n    a,\n    b\n  },\n  c\n]");
    }

    #[test]
    fn formatter_preserves_plain_multiline_binary_rhs_inside_calls() {
        let formatted = format_source("f[(x +\ny),\nz]").unwrap();
        assert_eq!(formatted, "f[\n  (x +\n    y),\n  z\n]");
    }

    #[test]
    fn formatter_leaves_top_level_multiline_binary_rhs_flush_left() {
        let formatted = format_source("x +\ny").unwrap();
        assert_eq!(formatted, "x +\ny");
    }

    #[test]
    fn formatter_keeps_max_width_binary_expressions_inline() {
        let formatted = format_source("aaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb").unwrap();
        assert_eq!(formatted, "aaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb");
    }

    #[test]
    fn formatter_actively_expands_wide_top_level_binary_expressions() {
        let formatted = format_source("aaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb").unwrap();
        assert_eq!(formatted, "aaaaaaaaaaaaaaaaaaa +\nbbbbbbbbbbbbbbbbbbb");
    }

    #[test]
    fn formatter_actively_expands_wide_binary_rhs_inside_calls() {
        let formatted = format_source("f[aaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbb]").unwrap();
        assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaaaa +\n  bbbbbbbbbbbbbbbbbbb]");
    }

    #[test]
    fn formatter_respects_custom_line_width_for_calls() {
        let formatted = format_with_options(
            "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]",
            FormatterOptions {
                line_width: 60,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]");
    }

    #[test]
    fn formatter_respects_custom_line_width_for_binary_chains() {
        let formatted = format_with_options(
            "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc",
            FormatterOptions {
                line_width: 60,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(
            formatted,
            "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc"
        );
    }

    #[test]
    fn formatter_keeps_short_binary_chains_inline() {
        let formatted = format_source("a + b + c + d").unwrap();
        assert_eq!(formatted, "a + b + c + d");
    }

    #[test]
    fn formatter_actively_expands_wide_top_level_binary_chains() {
        let formatted = format_source("aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc").unwrap();
        assert_eq!(
            formatted,
            "aaaaaaaaaaaaaa +\nbbbbbbbbbbbbbb +\ncccccccccccccc"
        );
    }

    #[test]
    fn formatter_actively_expands_wide_binary_chains_inside_calls() {
        let formatted =
            format_source("f[aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc]").unwrap();
        assert_eq!(
            formatted,
            "f[aaaaaaaaaaaaaa +\n  bbbbbbbbbbbbbb +\n  cccccccccccccc]"
        );
    }

    #[test]
    fn formatter_keeps_parent_calls_inline_when_nested_binary_chains_fit() {
        let formatted = format_source("f[(a +\nb + c), z]").unwrap();
        assert_eq!(formatted, "f[(a +\n    b +\n    c), z]");
    }

    #[test]
    fn formatter_expands_parent_calls_when_nested_binary_chains_make_the_parent_wide() {
        let formatted =
            format_source("f[(aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc), z]").unwrap();
        assert_eq!(
            formatted,
            "f[\n  (aaaaaaaaaaaaaa +\n    bbbbbbbbbbbbbb +\n    cccccccccccccc),\n  z\n]"
        );
    }

    #[test]
    fn formatter_wraps_right_associative_assignment_chains() {
        let formatted = format_source("aaaaaaaaaaaaaa = bbbbbbbbbbbbbb = cccccccccccccc").unwrap();
        assert_eq!(
            formatted,
            "aaaaaaaaaaaaaa =\nbbbbbbbbbbbbbb =\ncccccccccccccc"
        );
    }

    #[test]
    fn formatter_normalizes_partially_multiline_binary_chains() {
        let formatted = format_source("a +\nb + c").unwrap();
        assert_eq!(formatted, "a +\nb +\nc");
    }

    #[test]
    fn formatter_normalizes_binary_chains_with_middle_multiline_comments() {
        let formatted = format_source("a +\n(* note *) b + c").unwrap();
        assert_eq!(formatted, "a +\n(* note *)\nb +\nc");
    }

    #[test]
    fn formatter_normalizes_binary_chains_with_trailing_multiline_comments() {
        let formatted = format_source("a + b +\n(* tail note *) c").unwrap();
        assert_eq!(formatted, "a +\nb +\n(* tail note *)\nc");
    }

    #[test]
    fn formatter_indents_binary_chains_with_multiline_comments_inside_calls() {
        let formatted = format_source("f[a +\n(* note *) b + c]").unwrap();
        assert_eq!(formatted, "f[a +\n  (* note *)\n  b +\n  c]");
    }

    #[test]
    fn formatter_indents_binary_chains_with_multiline_comments_inside_parenthesized_calls() {
        let formatted = format_source("f[(a +\n(* note *) b + c), z]").unwrap();
        assert_eq!(formatted, "f[(a +\n    (* note *)\n    b +\n    c), z]");
    }

    #[test]
    fn formatter_normalizes_binary_chains_with_mixed_inline_and_multiline_comments() {
        let formatted = format_source("a + (* mid inline *) b +\n(* tail block *) c").unwrap();
        assert_eq!(formatted, "a +\n(* mid inline *)\nb +\n(* tail block *)\nc");
    }

    #[test]
    fn formatter_keeps_lhs_inline_comments_while_expanding_binary_chains() {
        let formatted = format_source("a (* lhs inline *) + b +\n(* tail block *) c").unwrap();
        assert_eq!(formatted, "a (* lhs inline *) +\nb +\n(* tail block *)\nc");
    }

    #[test]
    fn formatter_indents_binary_chains_with_mixed_inline_and_multiline_comments_inside_calls() {
        let formatted = format_source("f[a + (* mid inline *) b +\n(* tail block *) c]").unwrap();
        assert_eq!(
            formatted,
            "f[a +\n  (* mid inline *)\n  b +\n  (* tail block *)\n  c]"
        );
    }

    #[test]
    fn formatter_preserves_outer_comment_trivia() {
        let formatted = format_source("(* header *)\n f[x] (* tail *)").unwrap();
        assert_eq!(formatted, "(* header *)\nf[x] (* tail *)");
    }

    #[test]
    fn formatter_preserves_inline_comments_inside_calls_and_binary_ops() {
        let formatted = format_source("f[x(* inner *)+y,(* note *)z]").unwrap();
        assert_eq!(formatted, "f[x (* inner *) + y, (* note *) z]");
    }

    #[test]
    fn formatter_keeps_multiline_comments_attached_in_place() {
        let formatted = format_source("f[x,\n(* note *) y]").unwrap();
        assert_eq!(formatted, "f[\n  x,\n  (* note *)\n  y\n]");
    }

    #[test]
    fn formatter_indents_multiline_comments_inside_nested_calls() {
        let formatted = format_source("f[g[x,\n(* note *) y], z]").unwrap();
        assert_eq!(formatted, "f[g[\n    x,\n    (* note *)\n    y\n  ], z]");
    }

    #[test]
    fn formatter_dedents_closing_delimiters_after_multiline_comments() {
        let formatted = format_source("f[x\n(* note *)]").unwrap();
        assert_eq!(formatted, "f[\n  x\n  (* note *)\n]");
    }

    #[test]
    fn formatter_indents_multiline_comments_before_binary_rhs_inside_calls() {
        let formatted = format_source("f[x +\n(* note *) y]").unwrap();
        assert_eq!(formatted, "f[x +\n  (* note *)\n  y]");
    }

    #[test]
    fn formatter_keeps_stacked_multiline_comments_grouped_with_the_following_arg() {
        let formatted = format_source("f[x,\n(* first *)\n(* second *) y]").unwrap();
        assert_eq!(formatted, "f[\n  x,\n  (* first *)\n  (* second *)\n  y\n]");
    }

    #[test]
    fn formatter_indents_multiline_comments_inside_nested_lists() {
        let formatted = format_source("f[{a,\n(* list note *) b}, c]").unwrap();
        assert_eq!(
            formatted,
            "f[{\n    a,\n    (* list note *)\n    b\n  }, c]"
        );
    }

    #[test]
    fn formatter_indents_multiline_comments_before_nested_outer_arguments() {
        let formatted = format_source("f[g[x],\n(* outer *) h[z]]").unwrap();
        assert_eq!(formatted, "f[\n  g[x],\n  (* outer *)\n  h[z]\n]");
    }

    #[test]
    fn formatter_dedents_nested_closing_delimiters_after_multiline_comments() {
        let formatted = format_source("f[g[x\n(* inner close *)], z]").unwrap();
        assert_eq!(formatted, "f[g[\n    x\n    (* inner close *)\n  ], z]");
    }

    #[test]
    fn formatter_indents_multiline_comments_inside_parenthesized_binary_rhs() {
        let formatted = format_source("f[(x +\n(* rhs note *) g[y]), z]").unwrap();
        assert_eq!(formatted, "f[(x +\n    (* rhs note *)\n    g[y]), z]");
    }

    #[test]
    fn formatter_handles_mixed_inline_and_multiline_comments_inside_nested_calls() {
        let formatted = format_source("f[g[x, (* inline *) y,\n(* block *) h[z]], w]").unwrap();
        assert_eq!(
            formatted,
            "f[g[\n    x,\n    (* inline *)\n    y,\n    (* block *)\n    h[z]\n  ], w]"
        );
    }

    #[test]
    fn formatter_handles_multiline_comments_at_multiple_nesting_levels() {
        let formatted = format_source(
            "f[g[x,\n(* inner block *) h[y, (* tail *) z]],\n(* outer block *) q[r]]",
        )
        .unwrap();
        assert_eq!(
            formatted,
            "f[\n  g[\n    x,\n    (* inner block *)\n    h[y, (* tail *) z]\n  ],\n  (* outer block *)\n  q[r]\n]"
        );
    }

    #[test]
    fn formatter_handles_mixed_inline_and_multiline_comments_inside_nested_lists() {
        let formatted =
            format_source("f[{a, (* list inline *) b,\n(* list block *) {c, d}}, t]").unwrap();
        assert_eq!(
            formatted,
            "f[{\n    a,\n    (* list inline *)\n    b,\n    (* list block *)\n    {c, d}\n  }, t]"
        );
    }

    #[test]
    fn formatter_handles_multiline_comments_in_parenthesized_binary_expressions_with_inline_comments()
     {
        let formatted = format_source(
            "f[(x (* left inline *) +\n(* rhs block *) g[y, (* arg inline *) z]), u]",
        )
        .unwrap();
        assert_eq!(
            formatted,
            "f[(x (* left inline *) +\n    (* rhs block *)\n    g[y, (* arg inline *) z]), u]"
        );
    }

    #[test]
    fn formatter_handles_deeply_nested_multiline_comments_with_following_inline_comments() {
        let formatted = format_source(
            "f[g[(x +\n(* deep block *) y), (* after paren *) z],\n(* outer note *) {m, n}]",
        )
        .unwrap();
        assert_eq!(
            formatted,
            "f[\n  g[(x +\n      (* deep block *)\n      y), (* after paren *) z],\n  (* outer note *)\n  {m, n}\n]"
        );
    }

    #[test]
    fn formatter_handles_stacked_comments_before_nested_closing_delimiters() {
        let formatted =
            format_source("f[g[x\n(* first *)\n(* second *)],\n(* outer *) y]").unwrap();
        assert_eq!(
            formatted,
            "f[\n  g[\n    x\n    (* first *)\n    (* second *)\n  ],\n  (* outer *)\n  y\n]"
        );
    }

    #[test]
    fn formatter_handles_stacked_rhs_comments_inside_parenthesized_calls() {
        let formatted =
            format_source("f[(x +\n(* first rhs *)\n(* second rhs *) h[y]),\n(* outer *) z]")
                .unwrap();
        assert_eq!(
            formatted,
            "f[\n  (x +\n    (* first rhs *)\n    (* second rhs *)\n    h[y]),\n  (* outer *)\n  z\n]"
        );
    }

    #[test]
    fn formatter_handles_nested_lists_followed_by_outer_comment_blocks() {
        let formatted =
            format_source("f[g[{a,\n(* list block *) b}, c],\n(* outer *) q[r, (* inline *) s]]")
                .unwrap();
        assert_eq!(
            formatted,
            "f[\n  g[{\n      a,\n      (* list block *)\n      b\n    }, c],\n  (* outer *)\n  q[r, (* inline *) s]\n]"
        );
    }

    #[test]
    fn formatter_actively_expands_long_call_argument_lists() {
        let formatted = format_source("f[a, b, c, d]").unwrap();
        assert_eq!(formatted, "f[\n  a,\n  b,\n  c,\n  d\n]");
    }

    #[test]
    fn formatter_actively_expands_long_list_literals() {
        let formatted = format_source("f[{a, b, c, d}, z]").unwrap();
        assert_eq!(formatted, "f[{\n    a,\n    b,\n    c,\n    d\n  }, z]");
    }

    #[test]
    fn formatter_expands_long_nested_calls_without_expanding_the_parent() {
        let formatted = format_source("f[g[a, b, c, d], z]").unwrap();
        assert_eq!(formatted, "f[g[\n    a,\n    b,\n    c,\n    d\n  ], z]");
    }

    #[test]
    fn formatter_keeps_max_width_calls_inline() {
        let formatted = format_source("f[aaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]").unwrap();
        assert_eq!(formatted, "f[aaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]");
    }

    #[test]
    fn formatter_actively_expands_wide_two_argument_calls() {
        let formatted = format_source("f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]").unwrap();
        assert_eq!(
            formatted,
            "f[\n  aaaaaaaaaaaaaaaaaa,\n  bbbbbbbbbbbbbbbbbb\n]"
        );
    }

    #[test]
    fn formatter_actively_expands_wide_top_level_lists() {
        let formatted = format_source("{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}").unwrap();
        assert_eq!(
            formatted,
            "{\n  aaaaaaaaaaaa,\n  bbbbbbbbbbbb,\n  cccccccccccc\n}"
        );
    }

    #[test]
    fn formatter_expands_wide_parent_calls_around_nested_lists() {
        let formatted = format_source("f[{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}, z]").unwrap();
        assert_eq!(
            formatted,
            "f[\n  {\n    aaaaaaaaaaaa,\n    bbbbbbbbbbbb,\n    cccccccccccc\n  },\n  z\n]"
        );
    }

    #[test]
    fn formatter_is_idempotent_on_regression_corpus() {
        let corpus = [
            "f [ x_Integer,  y]&",
            "f[g[x,\ny],\nz]",
            "f[(x +\ny),\nz]",
            "aaaaaaaaaaaaaa + bbbbbbbbbbbbbb + cccccccccccccc",
            "a + (* mid inline *) b +\n(* tail block *) c",
            "f[g[x, (* inline *) y,\n(* block *) h[z]], w]",
            "f[{a, (* list inline *) b,\n(* list block *) {c, d}}, t]",
            "f[{aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc}, z]",
            "aaaaaaaaaaaaaa = bbbbbbbbbbbbbb = cccccccccccccc",
            "(* header *)\n f[x] (* tail *)",
        ];

        for source in corpus {
            let once = format_source(source).expect("corpus case should format");
            let twice = format_source(&once).expect("formatted corpus case should reformat");
            assert_eq!(twice, once, "formatter should be idempotent for {source:?}");
        }
    }
}
