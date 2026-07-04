pub(super) use crate::ast::{BinaryOp, BlankKind, PostfixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
pub(super) use crate::{Span, SyntaxError};

pub(super) use super::super::front::MAX_PARSE_DEPTH;
pub(super) use super::super::parse_expr;

pub(super) fn z() -> Span {
    Span::default()
}
