use crate::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("input is empty")]
    EmptyInput,
    #[error("expected {expected}, found {found} at bytes {span}")]
    UnexpectedToken {
        expected: &'static str,
        found: String,
        span: Span,
    },
    #[error("incomplete input: expected {expected} at bytes {span}")]
    Incomplete { expected: &'static str, span: Span },
    #[error("incomplete input: expected {expected} at bytes {span}")]
    UnclosedDelimiter {
        expected: &'static str,
        span: Span,
        opening_span: Span,
        opening: &'static str,
    },
    #[error("trailing input at bytes {span}")]
    TrailingInput { span: Span },
    #[error("parse nesting exceeded limit {max} at bytes {span}")]
    ParseDepthExceeded { max: usize, span: Span },
    #[error(
        "statement chain exceeded limit {max} at bytes {span}; split the statements across separate lines"
    )]
    StatementChainTooLong { max: usize, span: Span },
    #[error("unexpected character `{found}` at bytes {span}")]
    UnexpectedCharacter { found: char, span: Span },
    #[error("unterminated string at bytes {span}")]
    UnterminatedString { span: Span },
    #[error("unterminated comment at bytes {span}")]
    UnterminatedComment { span: Span },
}

impl SyntaxError {
    pub fn is_incomplete(&self) -> bool {
        matches!(
            self,
            Self::Incomplete { .. }
                | Self::UnclosedDelimiter { .. }
                | Self::UnterminatedString { .. }
                | Self::UnterminatedComment { .. }
        )
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::EmptyInput => None,
            Self::UnexpectedToken { span, .. }
            | Self::Incomplete { span, .. }
            | Self::UnclosedDelimiter { span, .. }
            | Self::TrailingInput { span }
            | Self::ParseDepthExceeded { span, .. }
            | Self::StatementChainTooLong { span, .. }
            | Self::UnexpectedCharacter { span, .. }
            | Self::UnterminatedString { span }
            | Self::UnterminatedComment { span } => Some(*span),
        }
    }

    pub fn related_span(&self) -> Option<Span> {
        match self {
            Self::UnclosedDelimiter { opening_span, .. } => Some(*opening_span),
            _ => None,
        }
    }

    pub fn related_label(&self) -> Option<String> {
        match self {
            Self::UnclosedDelimiter {
                opening, expected, ..
            } => Some(format!("opening `{opening}` for expected `{expected}`")),
            _ => None,
        }
    }
}
