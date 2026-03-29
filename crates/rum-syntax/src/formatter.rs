use std::collections::BTreeMap;

use crate::{
    parse_with_diagnostics, ParsedSource, SyntaxError, SyntaxExpr, SyntaxExprKind, SyntaxTokenKind,
    TriviaKind,
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

// The document IR forms the boundary between layout planning and final rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Doc {
    Text(String),
    #[allow(dead_code)]
    HardLine {
        indentation_level: usize,
    },
    SoftLine {
        indentation_level: usize,
    },
    Break {
        flat: String,
        indentation_level: usize,
    },
    Concat(Vec<Doc>),
    Group {
        doc: Box<Doc>,
        force_break: bool,
        flat_width: Option<usize>,
    },
}

#[path = "formatter/render.rs"]
mod render;

use render::{doc_trailing_state, flat_doc_width, render_doc};

#[derive(Debug)]
struct DocBuilder {
    parts: Vec<Doc>,
    pending_break: Option<(String, usize)>,
    ends_with_newline: bool,
    ends_with_whitespace: bool,
}

impl DocBuilder {
    fn new() -> Self {
        Self {
            parts: Vec::new(),
            pending_break: None,
            ends_with_newline: false,
            ends_with_whitespace: false,
        }
    }

    fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    fn ends_with_newline(&self) -> bool {
        self.pending_break.is_some() || self.ends_with_newline
    }

    fn ends_with_whitespace(&self) -> bool {
        self.pending_break.is_some() || self.ends_with_whitespace
    }

    fn push_char(&mut self, ch: char) {
        self.push_text(&ch.to_string());
    }

    fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.flush_pending_break();

        match self.parts.last_mut() {
            Some(Doc::Text(existing)) => existing.push_str(text),
            _ => self.parts.push(Doc::Text(text.to_owned())),
        }

        if let Some(last_char) = text.chars().next_back() {
            self.ends_with_newline = last_char == '\n';
            self.ends_with_whitespace = last_char.is_whitespace();
        }
    }

    fn push_doc(&mut self, doc: Doc) {
        self.flush_pending_break();

        match doc {
            Doc::Text(text) => self.push_text(&text),
            other => {
                let (ends_with_newline, ends_with_whitespace) = doc_trailing_state(&other);
                self.parts.push(other);
                self.ends_with_newline = ends_with_newline;
                self.ends_with_whitespace = ends_with_whitespace;
            }
        }
    }

    fn push_break(&mut self, flat: String, indentation_level: usize) {
        if self.ends_with_newline() {
            return;
        }

        self.pending_break = Some((flat, indentation_level));
        self.ends_with_newline = true;
        self.ends_with_whitespace = true;
    }

    fn finish(mut self) -> Doc {
        self.flush_pending_break();
        concat_docs(self.parts)
    }

    fn flush_pending_break(&mut self) {
        if let Some((flat, indentation_level)) = self.pending_break.take() {
            self.parts.push(Doc::Break {
                flat,
                indentation_level,
            });
        }
    }
}

fn concat_docs(parts: Vec<Doc>) -> Doc {
    if parts.len() == 1 {
        parts
            .into_iter()
            .next()
            .expect("single-part document should exist")
    } else {
        Doc::Concat(parts)
    }
}

fn append_softline(doc: Doc, indentation_level: usize) -> Doc {
    if doc_is_empty(&doc) || doc_trailing_state(&doc).0 {
        return doc;
    }

    concat_docs(vec![doc, Doc::SoftLine { indentation_level }])
}

fn doc_is_empty(doc: &Doc) -> bool {
    matches!(doc, Doc::Concat(parts) if parts.is_empty())
}

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

#[derive(Clone, Debug)]
enum GroupLayout {
    Container {
        item_start_indices: Vec<usize>,
        closing_index: usize,
    },
    BinaryChain {
        rhs_start_indices: Vec<usize>,
    },
}

impl GroupLayout {
    fn break_before_token(&self, token_index: usize) -> bool {
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
struct GroupPlan {
    start_index: usize,
    end_index: usize,
    layout: GroupLayout,
    force_break: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GroupIdentity {
    start_index: usize,
    end_index: usize,
}

struct FormattingContext<'a> {
    parsed: &'a ParsedSource,
    group_plans: BTreeMap<usize, Vec<GroupPlan>>,
}

pub fn format_source(source: &str) -> Result<String, SyntaxError> {
    let parsed = parse_with_diagnostics(source).map_err(|diagnostic| diagnostic.into_error())?;
    Ok(format_parsed_source(&parsed, FormatterOptions::default()))
}

pub fn format_parsed_source(parsed: &ParsedSource, options: FormatterOptions) -> String {
    let mut document = build_format_document(parsed);

    if options.trailing_newline {
        document = append_softline(document, 0);
    }

    render_doc(&document, options.line_width)
}

fn build_format_document(parsed: &ParsedSource) -> Doc {
    let context = build_formatting_context(parsed);
    let end_index = parsed.lexed().tokens().len().saturating_sub(1);
    build_document(&context, 0, end_index, 0, false, None, None)
}

fn build_formatting_context(parsed: &ParsedSource) -> FormattingContext<'_> {
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

fn build_document(
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

fn leading_trivia_has_newline(parsed: &ParsedSource, token_index: usize) -> bool {
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
    fn formatter_actively_expands_binary_expressions_with_long_inline_comments() {
        let formatted = format_with_options(
            "aaaaaaaa + (* very very very long inline comment *) bbbbbbbb",
            FormatterOptions {
                line_width: 30,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(
            formatted,
            "aaaaaaaa +\n(* very very very long inline comment *)\nbbbbbbbb"
        );
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
    fn formatter_actively_expands_calls_with_long_inline_comments() {
        let formatted = format_with_options(
            "f[x (* very very very long inline comment *), y]",
            FormatterOptions {
                line_width: 30,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(
            formatted,
            "f[\n  x (* very very very long inline comment *),\n  y\n]"
        );
    }

    #[test]
    fn formatter_actively_expands_calls_with_escaped_string_literals() {
        let formatted = format_with_options(
            "f[\"aaaaaaaaaa\\\"bbbbbbbbbb\", z]",
            FormatterOptions {
                line_width: 29,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(formatted, "f[\n  \"aaaaaaaaaa\\\"bbbbbbbbbb\",\n  z\n]");
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
    fn formatter_accounts_for_current_column_when_fitting_nested_groups() {
        let formatted = format_with_options(
            "f[aaaaaaaa + bbbbbbbb]",
            FormatterOptions {
                line_width: 20,
                ..FormatterOptions::default()
            },
        );
        assert_eq!(formatted, "f[aaaaaaaa +\n  bbbbbbbb]");
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
            "f[\n  g[\n    x,\n    (* inline *)\n    y,\n    (* block *)\n    h[z]\n  ],\n  w\n]"
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
            "f[\n  {\n    a,\n    (* list inline *)\n    b,\n    (* list block *)\n    {c, d}\n  },\n  t\n]"
        );
    }

    #[test]
    fn formatter_handles_multiline_comments_in_parenthesized_binary_expressions_with_inline_comments(
    ) {
        let formatted = format_source(
            "f[(x (* left inline *) +\n(* rhs block *) g[y, (* arg inline *) z]), u]",
        )
        .unwrap();
        assert_eq!(
            formatted,
            "f[\n  (x (* left inline *) +\n    (* rhs block *)\n    g[y, (* arg inline *) z]),\n  u\n]"
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
            "f[\n  g[\n    (x +\n      (* deep block *)\n      y),\n    (* after paren *)\n    z\n  ],\n  (* outer note *)\n  {m, n}\n]"
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
    fn formatter_handles_association_literals() {
        let formatted = format_source("<|a->1,b-><|c->3,d->4|>|>").unwrap();
        assert_eq!(formatted, "<|a -> 1, b -> <|c -> 3, d -> 4|>|>");
    }

    #[test]
    fn formatter_spaces_apply_sugar() {
        let formatted = format_source("f@g@x").unwrap();
        assert_eq!(formatted, "f @ g @ x");
    }

    #[test]
    fn formatter_spaces_functional_operator_sugars() {
        assert_eq!(format_source("f@@g").unwrap(), "f @@ g");
        assert_eq!(format_source("f@@@xs").unwrap(), "f @@@ xs");
        assert_eq!(format_source("f@*g@x").unwrap(), "f @* g @ x");
        assert_eq!(format_source("f/*g@x").unwrap(), "f /* g @ x");
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
