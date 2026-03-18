use std::fmt::{self, Write};

use crate::parser::parse_tokens;
use crate::{LexedSource, Span, SyntaxError, SyntaxExpr, lex_with_trivia};

#[derive(Clone, Debug)]
pub struct SourceText {
    source: String,
    line_starts: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub struct SourceDiagnostic {
    source: SourceText,
    error: SyntaxError,
}

#[derive(Clone, Debug)]
pub struct ParsedSource {
    source: SourceText,
    lexed: LexedSource,
    syntax: SyntaxExpr,
}

impl SourceText {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let line_starts = collect_line_starts(&source);

        Self {
            source,
            line_starts,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn text(&self, span: Span) -> Option<&str> {
        self.source.get(span.start..span.end)
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.source.len() {
            return None;
        }

        let line_index = self.line_starts.partition_point(|start| *start <= offset);
        let line_index = line_index.saturating_sub(1);
        let line_start = *self.line_starts.get(line_index)?;

        Some(SourceLocation {
            line: line_index + 1,
            column: offset.saturating_sub(line_start) + 1,
        })
    }

    pub fn span_locations(&self, span: Span) -> Option<(SourceLocation, SourceLocation)> {
        Some((self.location(span.start)?, self.location(span.end)?))
    }

    pub fn render_syntax_error(&self, error: &SyntaxError) -> String {
        let mut output = String::new();
        writeln!(&mut output, "error: {error}").unwrap();

        let Some(span) = error.span() else {
            return output.trim_end().to_string();
        };

        let Some(location) = self.location(span.start) else {
            return output.trim_end().to_string();
        };

        writeln!(&mut output, " --> {}:{}", location.line, location.column).unwrap();
        writeln!(&mut output, "  |").unwrap();

        let line_text = self.line_text(location.line).unwrap_or("");
        let gutter_width = location.line.to_string().len();

        writeln!(
            &mut output,
            "{:>width$} | {}",
            location.line,
            line_text,
            width = gutter_width
        )
        .unwrap();
        writeln!(
            &mut output,
            "{:>width$} | {}",
            "",
            self.highlight_line_span(location.line, span),
            width = gutter_width
        )
        .unwrap();

        if self.span_crosses_line(span, location.line) {
            writeln!(
                &mut output,
                "{:>width$} = note: showing only the first line of a multi-line span",
                "",
                width = gutter_width
            )
            .unwrap();
        }

        output.trim_end().to_string()
    }

    fn line_bounds(&self, line: usize) -> Option<(usize, usize)> {
        let line_index = line.checked_sub(1)?;
        let start = *self.line_starts.get(line_index)?;
        let end = self
            .line_starts
            .get(line_index + 1)
            .copied()
            .map(|next| next.saturating_sub(1))
            .unwrap_or(self.source.len());

        Some((start, end))
    }

    fn line_text(&self, line: usize) -> Option<&str> {
        let (start, end) = self.line_bounds(line)?;
        Some(self.source.get(start..end)?.trim_end_matches('\r'))
    }

    fn highlight_line_span(&self, line: usize, span: Span) -> String {
        let Some((line_start, line_end)) = self.line_bounds(line) else {
            return "^".to_string();
        };

        let line_text = self.source.get(line_start..line_end).unwrap_or("");
        let line_text = line_text.trim_end_matches('\r');
        let visible_line_end = line_start + line_text.len();

        let start_column = self.location(span.start).map_or(1, |loc| loc.column);
        let highlight_end = span.end.min(visible_line_end);
        let end_column = if highlight_end > span.start {
            self.location(highlight_end)
                .map_or(start_column + 1, |loc| loc.column)
        } else {
            start_column + 1
        };

        format!(
            "{}{}",
            " ".repeat(start_column.saturating_sub(1)),
            "^".repeat(end_column.saturating_sub(start_column).max(1))
        )
    }

    fn span_crosses_line(&self, span: Span, line: usize) -> bool {
        self.line_bounds(line)
            .is_some_and(|(_, line_end)| span.end > line_end)
    }
}

impl SourceDiagnostic {
    pub fn new(source: SourceText, error: SyntaxError) -> Self {
        Self { source, error }
    }

    pub fn source(&self) -> &SourceText {
        &self.source
    }

    pub fn error(&self) -> &SyntaxError {
        &self.error
    }

    pub fn into_error(self) -> SyntaxError {
        self.error
    }

    pub fn render(&self) -> String {
        self.source.render_syntax_error(&self.error)
    }
}

impl fmt::Display for SourceDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

pub fn parse_with_diagnostics(source: impl Into<String>) -> Result<ParsedSource, SourceDiagnostic> {
    let source = SourceText::new(source);
    let lexed = lex_with_trivia(source.source())
        .map_err(|error| SourceDiagnostic::new(source.clone(), error))?;
    let syntax = parse_tokens(lexed.tokens().to_vec())
        .map_err(|error| SourceDiagnostic::new(source.clone(), error))?;

    Ok(ParsedSource {
        source,
        lexed,
        syntax,
    })
}

impl ParsedSource {
    pub fn parse(source: impl Into<String>) -> Result<Self, SyntaxError> {
        parse_with_diagnostics(source).map_err(SourceDiagnostic::into_error)
    }

    pub fn source_text(&self) -> &SourceText {
        &self.source
    }

    pub fn source(&self) -> &str {
        self.source.source()
    }

    pub fn lexed(&self) -> &LexedSource {
        &self.lexed
    }

    pub fn syntax(&self) -> &SyntaxExpr {
        &self.syntax
    }

    pub fn text(&self, span: Span) -> Option<&str> {
        self.source.text(span)
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        self.source.location(offset)
    }

    pub fn span_locations(&self, span: Span) -> Option<(SourceLocation, SourceLocation)> {
        self.source.span_locations(span)
    }

    pub fn render_syntax_error(&self, error: &SyntaxError) -> String {
        self.source.render_syntax_error(error)
    }

    pub fn debug_dump(&self) -> String {
        let mut output = String::new();
        writeln!(&mut output, "syntax:").unwrap();
        writeln!(&mut output, "{:#?}", self.syntax).unwrap();
        writeln!(&mut output, "tokens:").unwrap();

        for (index, token) in self.lexed.tokens().iter().enumerate() {
            let text = self.text(token.span).map(debug_text).unwrap_or_default();
            let location = self
                .location(token.span.start)
                .map(|location| format!("{}:{}", location.line, location.column))
                .unwrap_or_else(|| "?".to_string());

            writeln!(
                &mut output,
                "  [{index}] {} @ {} ({location}) text=\"{}\"",
                token.kind.label(),
                token.span,
                text
            )
            .unwrap();

            if let Some(trivia) = self.lexed.leading_trivia(index) {
                for piece in trivia {
                    let text = debug_text(&piece.text);
                    writeln!(
                        &mut output,
                        "       trivia {} @ {} text=\"{}\"",
                        piece.kind.label(),
                        piece.span,
                        text
                    )
                    .unwrap();
                }
            }
        }

        output
    }
}

fn collect_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];

    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(index + ch.len_utf8());
        }
    }

    starts
}

fn debug_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| ch.escape_debug())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_source_preserves_source_slices_locations_and_trivia() {
        let parsed = ParsedSource::parse("f[\n  (* note *) x\n]").unwrap();

        assert_eq!(
            parsed.text(parsed.syntax.span()),
            Some("f[\n  (* note *) x\n]")
        );
        assert_eq!(
            parsed.location(3),
            Some(SourceLocation { line: 2, column: 1 })
        );
        assert_eq!(
            parsed.span_locations(Span::new(0, 1)),
            Some((
                SourceLocation { line: 1, column: 1 },
                SourceLocation { line: 1, column: 2 },
            ))
        );

        let trivia = parsed.lexed.leading_trivia(2).unwrap();
        assert_eq!(trivia.len(), 3);
        assert_eq!(trivia[1].text, "(* note *)");
    }

    #[test]
    fn parsed_source_debug_dump_includes_syntax_tokens_and_trivia() {
        let parsed = ParsedSource::parse("f[(* note *) x]").unwrap();
        let dump = parsed.debug_dump();

        assert!(dump.contains("syntax:"));
        assert!(dump.contains("tokens:"));
        assert!(dump.contains("symbol @ 0..1"));
        assert!(dump.contains("trivia comment"));
        assert!(dump.contains("(* note *)"));
    }

    #[test]
    fn source_diagnostic_renders_line_column_and_snippet() {
        let diagnostic = parse_with_diagnostics("f[1 2]").unwrap_err();
        let rendered = diagnostic.render();

        assert!(rendered.contains("error: expected ,"));
        assert!(rendered.contains("--> 1:5"));
        assert!(rendered.contains("1 | f[1 2]"));
        assert!(rendered.contains("^"));
    }
}
