use crate::{ParsedSource, SyntaxError, SyntaxTokenKind, TriviaKind, parse_with_diagnostics};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FormatterOptions {
    pub trailing_newline: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct TriviaRenderState {
    inserted_separator: bool,
    ended_with_newline: bool,
}

pub fn format_source(source: &str) -> Result<String, SyntaxError> {
    let parsed = parse_with_diagnostics(source).map_err(|diagnostic| diagnostic.into_error())?;
    Ok(format_parsed_source(&parsed, FormatterOptions::default()))
}

pub fn format_parsed_source(parsed: &ParsedSource, options: FormatterOptions) -> String {
    let mut output = String::new();
    let tokens = parsed.lexed().tokens();
    let mut before_previous_kind: Option<SyntaxTokenKind> = None;
    let mut previous_kind: Option<SyntaxTokenKind> = None;

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.kind, SyntaxTokenKind::End) {
            render_end_trivia(parsed, index, &mut output);
            break;
        }

        let trivia_state = render_leading_trivia(
            parsed,
            index,
            previous_kind.as_ref(),
            &token.kind,
            &mut output,
        );

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
    }

    if options.trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }

    output
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
    output: &mut String,
) -> TriviaRenderState {
    let Some(trivia) = parsed.lexed().leading_trivia(token_index) else {
        return TriviaRenderState::default();
    };

    let mut has_newline = false;
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

    #[test]
    fn formatter_preserves_inline_comments_inside_calls_and_binary_ops() {
        let formatted = format_source("f[x(* inner *)+y,(* note *)z]").unwrap();
        assert_eq!(formatted, "f[x (* inner *) + y, (* note *) z]");
    }

    #[test]
    fn formatter_keeps_multiline_comments_attached_in_place() {
        let formatted = format_source("f[x,\n(* note *) y]").unwrap();
        assert_eq!(formatted, "f[x,\n(* note *)\ny]");
    }
}
