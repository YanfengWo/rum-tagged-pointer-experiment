use super::{Doc, INDENT_WIDTH};

pub(super) fn render_doc(doc: &Doc, line_width: usize) -> String {
    render_doc_mode(doc, line_width, RenderMode::Broken)
}

fn render_doc_mode(doc: &Doc, line_width: usize, mode: RenderMode) -> String {
    let mut output = String::new();
    render_doc_into(doc, line_width, mode, 0, &mut output);
    output
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderMode {
    Flat,
    Broken,
}

fn render_doc_into(
    doc: &Doc,
    line_width: usize,
    mode: RenderMode,
    current_column: usize,
    output: &mut String,
) -> usize {
    match doc {
        Doc::Text(text) => {
            output.push_str(text);
            advance_column(current_column, text)
        }
        Doc::HardLine { indentation_level } => {
            output.push('\n');
            for _ in 0..indentation_level.saturating_mul(INDENT_WIDTH) {
                output.push(' ');
            }
            indentation_level.saturating_mul(INDENT_WIDTH)
        }
        Doc::SoftLine { indentation_level } => {
            if matches!(mode, RenderMode::Flat) {
                return current_column;
            }

            output.push('\n');
            for _ in 0..indentation_level.saturating_mul(INDENT_WIDTH) {
                output.push(' ');
            }
            indentation_level.saturating_mul(INDENT_WIDTH)
        }
        Doc::Break {
            flat,
            indentation_level,
        } => {
            if matches!(mode, RenderMode::Flat) {
                output.push_str(flat);
                return advance_column(current_column, flat);
            }

            output.push('\n');
            for _ in 0..indentation_level.saturating_mul(INDENT_WIDTH) {
                output.push(' ');
            }
            indentation_level.saturating_mul(INDENT_WIDTH)
        }
        Doc::Concat(parts) => {
            let mut column = current_column;
            for part in parts {
                column = render_doc_into(part, line_width, mode, column, output);
            }
            column
        }
        Doc::Group {
            doc,
            force_break,
            flat_width,
        } => {
            let can_render_flat = !force_break
                && flat_width.is_some_and(|width| current_column + width <= line_width);
            let group_mode = if can_render_flat {
                RenderMode::Flat
            } else {
                RenderMode::Broken
            };
            render_doc_into(doc, line_width, group_mode, current_column, output)
        }
    }
}

fn advance_column(current_column: usize, text: &str) -> usize {
    if let Some(last_line) = text.rsplit('\n').next() {
        if text.contains('\n') {
            last_line.len()
        } else {
            current_column + text.len()
        }
    } else {
        current_column
    }
}

pub(super) fn doc_trailing_state(doc: &Doc) -> (bool, bool) {
    match doc {
        Doc::Text(text) => {
            let ends_with_newline = text.ends_with('\n');
            let ends_with_whitespace = text.chars().next_back().is_some_and(char::is_whitespace);
            (ends_with_newline, ends_with_whitespace)
        }
        Doc::HardLine { .. } => (true, true),
        Doc::SoftLine { .. } => (true, true),
        Doc::Break { .. } => (true, true),
        Doc::Concat(parts) => parts
            .last()
            .map(doc_trailing_state)
            .unwrap_or((false, false)),
        Doc::Group { doc, .. } => doc_trailing_state(doc),
    }
}
