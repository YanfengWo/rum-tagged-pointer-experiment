mod ast;
mod error;
mod lexer;
mod parser;
mod span;
mod trivia;

pub use ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
pub use error::SyntaxError;
pub use lexer::lex_with_trivia;
pub use parser::parse_expr;
pub use span::Span;
pub use trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaKind, TriviaPiece};
