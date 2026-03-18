mod ast;
mod document;
mod error;
mod formatter;
mod lexer;
mod parser;
mod span;
mod trivia;

pub use ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
pub use document::{
    ParsedSource, SourceDiagnostic, SourceLocation, SourceText, parse_with_diagnostics,
};
pub use error::SyntaxError;
pub use formatter::{FormatterOptions, format_parsed_source, format_source};
pub use lexer::lex_with_trivia;
pub use parser::parse_expr;
pub use span::Span;
pub use trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaKind, TriviaPiece};
