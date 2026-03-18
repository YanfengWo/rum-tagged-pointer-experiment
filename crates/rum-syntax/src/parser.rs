use crate::ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr};
use crate::error::SyntaxError;
use crate::lexer::{Token, TokenKind, lex};

mod front;
mod helpers;

use front::Parser;
pub use front::parse_expr;

#[cfg(test)]
#[path = "parser/tests.rs"]
mod tests;
