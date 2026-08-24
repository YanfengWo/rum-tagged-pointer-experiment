use crate::Span;
use crate::ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr};
use crate::error::SyntaxError;
use crate::lexer::{Token, TokenKind, lex};

mod front;
mod helpers;
mod script;

use front::Parser;
pub use front::parse_expr;
pub use script::{ScriptForm, ScriptFormsError, parse_script_forms};

pub(crate) fn parse_tokens(tokens: Vec<Token>) -> Result<SyntaxExpr, SyntaxError> {
    let mut parser = Parser::new(&tokens);
    parser.parse_root()
}

#[cfg(test)]
#[path = "parser/tests/mod.rs"]
mod tests;
