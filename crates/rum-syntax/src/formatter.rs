use crate::{ParsedSource, SyntaxError, TriviaKind, parse_with_diagnostics};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FormatterOptions {
    pub trailing_newline: bool,
}

pub fn format_source(source: &str) -> Result<String, SyntaxError> {
    let parsed = parse_with_diagnostics(source).map_err(|diagnostic| diagnostic.into_error())?;
    Ok(format_parsed_source(&parsed, FormatterOptions::default()))
}

pub fn format_parsed_source(parsed: &ParsedSource, options: FormatterOptions) -> String {
    let mut output = String::new();

    if let Some(prefix) = leading_comment_banner(parsed, 0) {
        output.push_str(&prefix);
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }

    output.push_str(&parsed.syntax().to_string());

    let end_index = parsed.lexed().tokens().len().saturating_sub(1);

    if let Some(suffix) = outer_trivia_text(parsed, end_index) {
        if !suffix.is_empty() {
            if !output.ends_with(char::is_whitespace) && !suffix.starts_with(char::is_whitespace) {
                output.push(' ');
            }

            output.push_str(&suffix);
        }
    }

    if options.trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

fn outer_trivia_text(parsed: &ParsedSource, token_index: usize) -> Option<String> {
    let trivia = parsed.lexed().leading_trivia(token_index)?;

    let text = trivia
        .iter()
        .filter(|piece| matches!(piece.kind, TriviaKind::Whitespace | TriviaKind::Comment))
        .map(|piece| piece.text.as_str())
        .collect::<String>();

    if text.is_empty() { None } else { Some(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatter_canonicalizes_core_spacing() {
        let formatted = format_source("f [ x_Integer,  y]&").unwrap();
        assert_eq!(formatted, "f[x_Integer, y] &");
    }

    #[test]
    fn formatter_preserves_outer_comment_trivia() {
        let formatted = format_source("(* header *)\n f[x] (* tail *)").unwrap();
        assert_eq!(formatted, "(* header *)\nf[x] (* tail *)");
    }
}

fn leading_comment_banner(parsed: &ParsedSource, token_index: usize) -> Option<String> {
    let trivia = parsed.lexed().leading_trivia(token_index)?;

    let comments = trivia
        .iter()
        .filter(|piece| matches!(piece.kind, TriviaKind::Comment))
        .map(|piece| piece.text.as_str())
        .collect::<Vec<_>>();

    if comments.is_empty() {
        None
    } else {
        Some(comments.join("\n"))
    }
}
