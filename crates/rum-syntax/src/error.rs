use crate::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("input is empty")]
    EmptyInput,
    #[error("expected {expected} at bytes {span}")]
    Expected { expected: &'static str, span: Span },
    #[error("expected {expected}, found {found} at bytes {span}")]
    UnexpectedToken {
        expected: &'static str,
        found: String,
        span: Span,
    },
    #[error("incomplete input: expected {expected} at bytes {span}")]
    Incomplete { expected: &'static str, span: Span },
    #[error("trailing input at bytes {span}")]
    TrailingInput { span: Span },
    #[error("unexpected character `{found}` at bytes {span}")]
    UnexpectedCharacter { found: char, span: Span },
    #[error("unterminated string at bytes {span}")]
    UnterminatedString { span: Span },
    #[error("unterminated comment at bytes {span}")]
    UnterminatedComment { span: Span },
}

impl SyntaxError {
    pub fn is_incomplete(&self) -> bool {
        matches!(self, Self::Incomplete { .. })
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::EmptyInput => None,
            Self::Expected { span, .. }
            | Self::UnexpectedToken { span, .. }
            | Self::Incomplete { span, .. }
            | Self::TrailingInput { span }
            | Self::UnexpectedCharacter { span, .. }
            | Self::UnterminatedString { span }
            | Self::UnterminatedComment { span } => Some(*span),
        }
    }
}
