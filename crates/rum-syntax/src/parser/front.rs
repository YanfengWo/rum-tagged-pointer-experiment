use super::*;
use crate::ast::SyntaxExprKind;

const SPAN_PRECEDENCE: u8 = 45;
const INFIX_APPLICATION_PRECEDENCE: u8 = 67;
/// A parser-specific guard kept below the debug and release native stack limits.
/// The recursive-descent parser uses multiple Rust frames per delimiter level,
/// so this is intentionally lower than the runtime evaluator's recursion limit.
pub(crate) const MAX_PARSE_DEPTH: usize = 128;

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let tokens = lex(input)?;
    super::parse_tokens(tokens)
}

pub(super) struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) index: usize,
    parse_depth: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            index: 0,
            parse_depth: 0,
        }
    }

    fn with_nested_parse_depth<T>(
        &mut self,
        span: Span,
        parse: impl FnOnce(&mut Self) -> Result<T, SyntaxError>,
    ) -> Result<T, SyntaxError> {
        if self.parse_depth >= MAX_PARSE_DEPTH {
            return Err(SyntaxError::ParseDepthExceeded {
                max: MAX_PARSE_DEPTH,
                span,
            });
        }

        self.parse_depth += 1;
        let result = parse(self);
        self.parse_depth -= 1;
        result
    }

    pub(super) fn parse_root(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        if self.at_end() {
            return Err(SyntaxError::EmptyInput);
        }

        let expr = self.parse_function_expr()?;

        if self.at_end() {
            Ok(expr)
        } else {
            Err(SyntaxError::TrailingInput {
                span: self.current().span,
            })
        }
    }

    pub(super) fn parse_function_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_binary_expr(0)?;

        loop {
            while let Some(ampersand_span) = self.consume_ampersand() {
                let span = expr.span().merge(ampersand_span);
                expr = SyntaxExpr::postfix(PostfixOp::Function, expr, span);
            }

            let Some(op) = self.current_binary_op() else {
                break;
            };

            if !can_follow_function_postfix(op) {
                break;
            }

            self.bump();
            let rhs = self.parse_binary_expr(next_min_precedence_for_binary_op(op))?;
            let span = expr.span().merge(rhs.span());
            expr = SyntaxExpr::binary(op, expr, rhs, span);
        }

        Ok(expr)
    }

    pub(super) fn parse_binary_expr(
        &mut self,
        min_precedence: u8,
    ) -> Result<SyntaxExpr, SyntaxError> {
        self.parse_binary_expr_internal(min_precedence, true, true)
    }

    fn parse_binary_expr_internal(
        &mut self,
        min_precedence: u8,
        allow_function_postfix: bool,
        allow_infix_application: bool,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let mut lhs = if matches!(self.current().kind, TokenKind::Span)
            && SPAN_PRECEDENCE >= min_precedence
        {
            self.parse_span_sugar(None)?
        } else {
            self.parse_prefix_expr()?
        };

        loop {
            if allow_function_postfix && PostfixOp::Function.precedence() >= min_precedence {
                if let Some(ampersand_span) = self.consume_ampersand() {
                    let span = lhs.span().merge(ampersand_span);
                    lhs = SyntaxExpr::postfix(PostfixOp::Function, lhs, span);
                    continue;
                }
            }

            if matches!(self.current().kind, TokenKind::Span) && SPAN_PRECEDENCE >= min_precedence {
                lhs = self.parse_span_sugar(Some(lhs))?;
                continue;
            }

            if matches!(self.current().kind, TokenKind::Tilde)
                && allow_infix_application
                && INFIX_APPLICATION_PRECEDENCE >= min_precedence
            {
                lhs = self.parse_infix_application(lhs, allow_function_postfix)?;
                continue;
            }

            let op = match self.current_binary_op() {
                Some(op) => op,
                None => {
                    // Implicit multiplication (juxtaposition): `a b` == Times[a, b].
                    let times = BinaryOp::Times;
                    if self.token_starts_primary() && times.precedence() >= min_precedence {
                        let next_min_precedence = next_min_precedence_for_binary_op(times);
                        let rhs = self.parse_binary_expr_internal(
                            next_min_precedence,
                            allow_function_postfix,
                            allow_infix_application,
                        )?;
                        let span = lhs.span().merge(rhs.span());
                        lhs = SyntaxExpr::binary(times, lhs, rhs, span);
                        continue;
                    }
                    break;
                }
            };

            if op.precedence() < min_precedence {
                break;
            }

            self.bump();

            let next_min_precedence = next_min_precedence_for_binary_op(op);

            let rhs =
                if matches!(op, BinaryOp::CompoundExpression) && !self.token_starts_expression() {
                    SyntaxExpr::symbol("Null", self.current().span)
                } else if matches!(op, BinaryOp::Rule | BinaryOp::RuleDelayed) {
                    self.parse_binary_expr_internal(
                        next_min_precedence,
                        false,
                        allow_infix_application,
                    )?
                } else {
                    self.parse_binary_expr_internal(
                        next_min_precedence,
                        allow_function_postfix,
                        allow_infix_application,
                    )?
                };
            let span = lhs.span().merge(rhs.span());
            lhs = SyntaxExpr::binary(op, lhs, rhs, span);
        }

        while let Some(unset_span) = self.consume_unset() {
            let span = lhs.span().merge(unset_span);
            lhs = SyntaxExpr::postfix(PostfixOp::Unset, lhs, span);
        }

        Ok(lhs)
    }

    fn parse_infix_application(
        &mut self,
        lhs: SyntaxExpr,
        allow_function_postfix: bool,
    ) -> Result<SyntaxExpr, SyntaxError> {
        self.bump();

        if !self.token_starts_expression() {
            return Err(if self.at_end() {
                SyntaxError::Incomplete {
                    expected: "expression",
                    span: self.current().span,
                }
            } else {
                SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.current_token_label(),
                    span: self.current().span,
                }
            });
        }

        let head = self.parse_binary_expr_internal(0, allow_function_postfix, false)?;
        self.expect_tilde()?;
        let rhs = self.parse_binary_expr_internal(
            INFIX_APPLICATION_PRECEDENCE + 1,
            allow_function_postfix,
            true,
        )?;
        let span = lhs.span().merge(rhs.span());

        Ok(SyntaxExpr::call(head, vec![lhs, rhs], span))
    }

    pub(super) fn parse_prefix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Plus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Plus.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Plus, expr, span))
            }
            TokenKind::PlusPlus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::PreIncrement.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::PreIncrement, expr, span))
            }
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Minus.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Minus, expr, span))
            }
            TokenKind::Bang => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Not.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Not, expr, span))
            }
            TokenKind::MinusMinus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::PreDecrement.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::PreDecrement, expr, span))
            }
            _ => self.parse_postfix_expr(),
        }
    }

    pub(super) fn parse_postfix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_primary()?;

        loop {
            if let Some(open_span) = self.consume_double_lbracket() {
                let (mut specs, close_span) = self
                    .with_nested_parse_depth(open_span, |parser| {
                        parser.parse_double_bracket_expression_list(open_span)
                    })?;
                let span = expr.span().merge(close_span);
                let mut args = vec![expr];
                args.append(&mut specs);
                expr = SyntaxExpr::call(SyntaxExpr::symbol("Part", open_span), args, span);
                continue;
            }

            if let Some(open_span) = self.consume_lbracket() {
                let (args, close_span) = self.with_nested_parse_depth(open_span, |parser| {
                    parser.parse_expression_list(TokenKind::RBracket, "]", open_span, "[")
                })?;
                let span = expr.span().merge(close_span);
                expr = SyntaxExpr::call(expr, args, span);
                continue;
            }

            if let Some((kind, blank_span)) = self.consume_blank_kind() {
                let pattern = self.parse_blank_tail(kind, blank_span)?;
                let span = expr.span().merge(pattern.span());
                expr = SyntaxExpr::pattern(expr, pattern, span);
                continue;
            }

            if let Some(increment_span) = self.consume_increment() {
                let span = expr.span().merge(increment_span);
                expr = SyntaxExpr::postfix(PostfixOp::Increment, expr, span);
                continue;
            }

            if let Some(decrement_span) = self.consume_decrement() {
                let span = expr.span().merge(decrement_span);
                expr = SyntaxExpr::postfix(PostfixOp::Decrement, expr, span);
                continue;
            }

            if let Some(factorial2_span) = self.consume_factorial2() {
                let span = expr.span().merge(factorial2_span);
                expr = SyntaxExpr::postfix(PostfixOp::Factorial2, expr, span);
                continue;
            }

            if let Some(factorial_span) = self.consume_factorial() {
                let span = expr.span().merge(factorial_span);
                expr = SyntaxExpr::postfix(PostfixOp::Factorial, expr, span);
                continue;
            }

            if let Some((order, derivative_span)) = self.consume_derivative() {
                let span = expr.span().merge(derivative_span);
                expr = SyntaxExpr::postfix(PostfixOp::Derivative(order), expr, span);
                continue;
            }

            if let Some(repeated_null_span) = self.consume_repeated_null() {
                let span = expr.span().merge(repeated_null_span);
                expr = SyntaxExpr::postfix(PostfixOp::RepeatedNull, expr, span);
                continue;
            }

            if let Some(repeated_span) = self.consume_repeated() {
                let span = expr.span().merge(repeated_span);
                expr = SyntaxExpr::postfix(PostfixOp::Repeated, expr, span);
                continue;
            }

            if is_optional_pattern_target(&expr) {
                if let Some(optional_span) = self.consume_dot() {
                    let span = expr.span().merge(optional_span);
                    expr = SyntaxExpr::postfix(PostfixOp::Optional, expr, span);
                    continue;
                }
            }

            break;
        }

        Ok(expr)
    }

    fn parse_double_bracket_expression_list(
        &mut self,
        opening_span: Span,
    ) -> Result<(Vec<SyntaxExpr>, Span), SyntaxError> {
        let mut items = Vec::new();

        if let Some(close_span) = self.consume_double_rbracket() {
            return Ok((items, close_span));
        }

        if self.at_end() {
            return Err(SyntaxError::UnclosedDelimiter {
                expected: "]]",
                span: self.current().span,
                opening_span,
                opening: "[[",
            });
        }

        loop {
            items.push(self.parse_function_expr()?);

            if let Some(close_span) = self.consume_double_rbracket() {
                return Ok((items, close_span));
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: "]]",
                    span: self.current().span,
                    opening_span,
                    opening: "[[",
                });
            }

            self.expect_comma()?;

            if self.current_starts_double_rbracket() {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.double_rbracket_label(),
                    span: self.current().span.merge(self.tokens[self.index + 1].span),
                });
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: "]]",
                    span: self.current().span,
                    opening_span,
                    opening: "[[",
                });
            }
        }
    }

    fn parse_span_sugar(&mut self, start: Option<SyntaxExpr>) -> Result<SyntaxExpr, SyntaxError> {
        let first_span = self
            .consume_span()
            .expect("parse_span_sugar only called at a span token");

        let start_span = start.as_ref().map(SyntaxExpr::span).unwrap_or(first_span);
        let mut end_span = first_span;
        let end =
            if matches!(self.current().kind, TokenKind::Span) || !self.token_starts_expression() {
                None
            } else {
                let expr = self.parse_binary_expr(SPAN_PRECEDENCE + 1)?;
                end_span = expr.span();
                Some(expr)
            };

        let step = if self.consume_span().is_some() {
            if !self.token_starts_expression() {
                return Err(if self.at_end() {
                    SyntaxError::Incomplete {
                        expected: "expression",
                        span: self.current().span,
                    }
                } else {
                    SyntaxError::UnexpectedToken {
                        expected: "expression",
                        found: self.current_token_label(),
                        span: self.current().span,
                    }
                });
            }

            let expr = self.parse_binary_expr(SPAN_PRECEDENCE + 1)?;
            end_span = expr.span();
            Some(expr)
        } else {
            None
        };

        let args = match (start, end, step) {
            (Some(start), Some(end), Some(step)) => vec![start, end, step],
            (Some(start), Some(end), None) => vec![start, end],
            (Some(start), None, Some(step)) => {
                vec![start, SyntaxExpr::symbol("All", first_span), step]
            }
            (Some(start), None, None) => vec![start, SyntaxExpr::symbol("All", first_span)],
            (None, Some(end), Some(step)) => {
                vec![SyntaxExpr::symbol("All", first_span), end, step]
            }
            (None, Some(end), None) => vec![SyntaxExpr::symbol("All", first_span), end],
            (None, None, Some(step)) => vec![
                SyntaxExpr::symbol("All", first_span),
                SyntaxExpr::symbol("All", first_span),
                step,
            ],
            (None, None, None) => vec![SyntaxExpr::symbol("All", first_span)],
        };

        let span = start_span.merge(end_span);
        Ok(SyntaxExpr::call(
            SyntaxExpr::symbol("Span", first_span),
            args,
            span,
        ))
    }

    pub(super) fn parse_primary(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let token = self.current().clone();

        match token.kind {
            TokenKind::Integer(value) => {
                self.bump();
                Ok(SyntaxExpr::integer(value, token.span))
            }
            TokenKind::String(value) => {
                self.bump();
                Ok(SyntaxExpr::string(value, token.span))
            }
            TokenKind::Symbol(value) => {
                self.bump();
                Ok(SyntaxExpr::symbol(value, token.span))
            }
            TokenKind::Hash => self.parse_slot(),
            TokenKind::Underscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::Blank, token.span)
            }
            TokenKind::DoubleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankSequence, token.span)
            }
            TokenKind::TripleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankNullSequence, token.span)
            }
            TokenKind::LParen => {
                self.bump();
                self.with_nested_parse_depth(token.span, |parser| {
                    let expr = parser.parse_function_expr()?;
                    let close_span = parser.expect_rparen(token.span, "(")?;
                    Ok(expr.with_span(token.span.merge(close_span)))
                })
            }
            TokenKind::LBrace => {
                self.bump();
                self.with_nested_parse_depth(token.span, |parser| {
                    let (items, close_span) =
                        parser.parse_expression_list(TokenKind::RBrace, "}", token.span, "{")?;
                    Ok(SyntaxExpr::list(items, token.span.merge(close_span)))
                })
            }
            TokenKind::LAssociation => {
                self.bump();
                self.with_nested_parse_depth(token.span, |parser| {
                    let (items, close_span) = parser.parse_expression_list(
                        TokenKind::RAssociation,
                        "|>",
                        token.span,
                        "<|",
                    )?;
                    Ok(SyntaxExpr::association(items, token.span.merge(close_span)))
                })
            }
            TokenKind::End => Err(SyntaxError::Incomplete {
                expected: "expression",
                span: token.span,
            }),
            _ => Err(SyntaxError::UnexpectedToken {
                expected: "expression",
                found: self.current_token_label(),
                span: token.span,
            }),
        }
    }
}

fn is_optional_pattern_target(expr: &SyntaxExpr) -> bool {
    match &expr.kind {
        SyntaxExprKind::Blank { .. } => true,
        SyntaxExprKind::Pattern { pattern, .. } => {
            matches!(&pattern.kind, SyntaxExprKind::Blank { .. })
        }
        _ => false,
    }
}

fn next_min_precedence_for_binary_op(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::ReplaceAll | BinaryOp::ReplaceRepeated => BinaryOp::Rule.precedence(),
        BinaryOp::Rule | BinaryOp::RuleDelayed => BinaryOp::Set.precedence(),
        _ if op.is_right_associative() => op.precedence(),
        _ => op.precedence() + 1,
    }
}

fn can_follow_function_postfix(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Apply | BinaryOp::Map | BinaryOp::ApplyHead | BinaryOp::MapApply
    )
}
