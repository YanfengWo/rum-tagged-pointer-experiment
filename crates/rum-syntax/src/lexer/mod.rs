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

pub(crate) fn lex(input: &str) -> Result<Vec<Token>, SyntaxError> {
    Ok(lex_with_trivia(input)?.into_tokens())
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
}
