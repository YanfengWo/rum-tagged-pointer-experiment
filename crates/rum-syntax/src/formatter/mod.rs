use std::collections::BTreeMap;

use crate::{
    ParsedSource, SyntaxError, SyntaxExpr, SyntaxExprKind, SyntaxTokenKind, TriviaKind,
    parse_with_diagnostics,
};

mod doc;
mod planning;
mod render;
#[cfg(test)]
mod tests;
mod trivia;

use self::{
    doc::{Doc, append_softline},
    planning::build_formatting_context,
    render::render_doc,
    trivia::build_document,
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
