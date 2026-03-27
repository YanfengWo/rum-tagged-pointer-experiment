use super::*;

const SPAN_PRECEDENCE: u8 = 45;

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let tokens = lex(input)?;
    super::parse_tokens(tokens)
}

pub(super) struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) index: usize,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
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

        while let Some(ampersand_span) = self.consume_ampersand() {
            let span = expr.span().merge(ampersand_span);
            expr = SyntaxExpr::postfix(PostfixOp::Function, expr, span);
        }

        Ok(expr)
    }

    pub(super) fn parse_binary_expr(
        &mut self,
        min_precedence: u8,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let mut lhs = if matches!(self.current().kind, TokenKind::Span)
            && SPAN_PRECEDENCE >= min_precedence
        {
            self.parse_span_sugar(None)?
        } else {
            self.parse_prefix_expr()?
        };

        loop {
            if matches!(self.current().kind, TokenKind::Span) && SPAN_PRECEDENCE >= min_precedence {
                lhs = self.parse_span_sugar(Some(lhs))?;
                continue;
            }

            let Some(op) = self.current_binary_op() else {
                break;
            };

            if op.precedence() < min_precedence {
                break;
            }

            self.bump();

            let next_min_precedence = if op.is_right_associative() {
                op.precedence()
            } else {
                op.precedence() + 1
            };

            let rhs = self.parse_binary_expr(next_min_precedence)?;
            let span = lhs.span().merge(rhs.span());
            lhs = SyntaxExpr::binary(op, lhs, rhs, span);
        }

        while let Some(unset_span) = self.consume_unset() {
            let span = lhs.span().merge(unset_span);
            lhs = SyntaxExpr::postfix(PostfixOp::Unset, lhs, span);
        }

        Ok(lhs)
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
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Minus.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Minus, expr, span))
            }
            _ => self.parse_postfix_expr(),
        }
    }

    pub(super) fn parse_postfix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_primary()?;

        loop {
            if let Some(open_span) = self.consume_double_lbracket() {
                let (mut specs, close_span) =
                    self.parse_double_bracket_expression_list(open_span)?;
                let span = expr.span().merge(close_span);
                let mut args = vec![expr];
                args.append(&mut specs);
                expr = SyntaxExpr::call(SyntaxExpr::symbol("Part", open_span), args, span);
                continue;
            }

            if let Some(open_span) = self.consume_lbracket() {
                let (args, close_span) =
                    self.parse_expression_list(TokenKind::RBracket, "]", open_span, "[")?;
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
                let expr = self.parse_function_expr()?;
                let close_span = self.expect_rparen(token.span, "(")?;
                Ok(expr.with_span(token.span.merge(close_span)))
            }
            TokenKind::LBrace => {
                self.bump();
                let (items, close_span) =
                    self.parse_expression_list(TokenKind::RBrace, "}", token.span, "{")?;
                Ok(SyntaxExpr::list(items, token.span.merge(close_span)))
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
