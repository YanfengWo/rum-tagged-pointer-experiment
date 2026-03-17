use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("input is empty")]
    EmptyInput,
    #[error("expected {expected} at byte {offset}")]
    Expected {
        expected: &'static str,
        offset: usize,
    },
    #[error("trailing input at byte {offset}")]
    TrailingInput { offset: usize },
    #[error("unexpected character `{found}` at byte {offset}")]
    UnexpectedCharacter { found: char, offset: usize },
    #[error("unterminated string starting at byte {offset}")]
    UnterminatedString { offset: usize },
}
