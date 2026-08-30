pub(super) use crate::ast::{BinaryOp, BlankKind, PostfixOp, SlotSpec, SyntaxExpr, SyntaxExprKind};
pub(super) use crate::{Span, SyntaxError};

pub(super) use super::super::front::{MAX_DELIMITER_NESTING_DEPTH, MAX_OPERATOR_CHAIN_DEPTH};
pub(super) use super::super::{parse_expr, parse_expr_with_offset, parse_script_form_with_source};

pub(super) fn z() -> Span {
    Span::default()
}
