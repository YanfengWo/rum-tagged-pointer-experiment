use crate::Span;
use crate::ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr};
use crate::error::SyntaxError;
use crate::lexer::{Token, TokenKind, lex};

mod front;
mod helpers;

use front::Parser;
pub use front::parse_expr;

pub(crate) fn parse_tokens(tokens: Vec<Token>) -> Result<SyntaxExpr, SyntaxError> {
    let mut parser = Parser::new(tokens);
    parser.parse_root()
}

#[cfg(test)]
#[path = "parser/tests.rs"]
mod tests;
