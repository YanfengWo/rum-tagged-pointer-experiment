//! Syntax infrastructure for `rum` source text.
//!
//! This crate owns the syntax tree, lexer, parser, source diagnostics,
//! preserved trivia, and formatter entrypoints used by the runtime and command
//! line tools.

mod ast;
mod document;
mod error;
mod format_request;
mod formatter;
mod lexer;
mod parser;
mod span;
mod trivia;

pub use ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
pub use document::{
    ParsedSource, SourceDiagnostic, SourceDiagnosticSpan, SourceLocation, SourceText,
    StructuredSourceDiagnostic, parse_with_diagnostics,
};
pub use error::SyntaxError;
pub use format_request::{parse_format_cli_args, parse_format_command};
pub use formatter::{FormatterOptions, format_parsed_source, format_source};
pub use lexer::lex_with_trivia;
pub use parser::parse_expr;
pub use span::Span;
pub use trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaKind, TriviaPiece};
