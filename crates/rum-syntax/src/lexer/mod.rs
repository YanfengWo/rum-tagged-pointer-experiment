use crate::error::SyntaxError;
use crate::span::Span;
use crate::trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaPiece};

mod core;
mod cursor;
mod named_characters;
mod numbers;
mod operators;
mod strings;
mod symbols;
#[cfg(test)]
mod tests;
mod trivia;

pub(crate) type Token = SyntaxToken;
pub(crate) type TokenKind = SyntaxTokenKind;

pub use named_characters::named_character;

pub fn lex_with_trivia(input: &str) -> Result<LexedSource, SyntaxError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    let mut trivia = Vec::new();

    while let Some(token) = lexer.next_token(&mut trivia)? {
        let is_end = matches!(token.kind, TokenKind::End);
        tokens.push(token);

        if is_end {
            break;
        }
    }

    Ok(LexedSource::new(tokens, trivia))
}

/// Returns whether a following `'''` token starts a raw string after `token`.
///
/// Incremental lexer consumers use this to preserve the lexer's context-sensitive
/// distinction between a raw string opener and three postfix prime tokens.
pub fn raw_string_operand_position_after(
    token: &SyntaxTokenKind,
    token_started_operand_position: bool,
) -> bool {
    core::token_can_be_followed_by_raw_string_operand(token, token_started_operand_position)
}

pub(crate) fn lex(input: &str) -> Result<Vec<Token>, SyntaxError> {
    Ok(lex_with_trivia(input)?.into_tokens())
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
    can_start_raw_string: bool,
}
