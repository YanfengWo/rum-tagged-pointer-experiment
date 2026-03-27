use super::*;

impl Parser {
    pub(super) fn parse_slot(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start_span = self.bump();
        let mut end_span = start_span;
        let is_sequence = matches!(self.current().kind, TokenKind::Hash)
            && self.current().leading_trivia.is_empty();

        if is_sequence {
            end_span = self.bump();
        }

        let spec = match (&self.current().kind, is_sequence) {
            (TokenKind::Integer(value), false) => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::Number(value)
            }
            (TokenKind::Symbol(value), false) => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::Named(value)
            }
            (TokenKind::Integer(value), true) => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::SequenceNumber(value)
            }
            _ if is_sequence => SlotSpec::Sequence,
            _ => SlotSpec::Anonymous,
        };

        Ok(SyntaxExpr::slot(spec, start_span.merge(end_span)))
    }

    pub(super) fn parse_blank_tail(
        &mut self,
        kind: BlankKind,
        start_span: Span,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let head = if self.token_starts_primary() {
            Some(self.parse_postfix_expr()?)
        } else {
            None
        };

        let span = head
            .as_ref()
            .map(|head| start_span.merge(head.span()))
            .unwrap_or(start_span);

        Ok(SyntaxExpr::blank(kind, head, span))
    }

    pub(super) fn parse_expression_list(
        &mut self,
        end_kind: TokenKind,
        end_label: &'static str,
        opening_span: Span,
        opening: &'static str,
    ) -> Result<(Vec<SyntaxExpr>, Span), SyntaxError> {
        let mut items = Vec::new();

        if self.current().kind == end_kind {
            let end_span = self.bump();
            return Ok((items, end_span));
        }

        if self.at_end() {
            return Err(SyntaxError::UnclosedDelimiter {
                expected: end_label,
                span: self.current().span,
                opening_span,
                opening,
            });
        }

        loop {
            items.push(self.parse_function_expr()?);

            if self.current().kind == end_kind {
                let end_span = self.bump();
                return Ok((items, end_span));
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: end_label,
                    span: self.current().span,
                    opening_span,
                    opening,
                });
            }

            self.expect_comma()?;

            if self.current().kind == end_kind {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.current_token_label(),
                    span: self.current().span,
                });
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: end_label,
                    span: self.current().span,
                    opening_span,
                    opening,
                });
            }
        }
    }

    pub(super) fn token_starts_primary(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Integer(_)
                | TokenKind::String(_)
                | TokenKind::Symbol(_)
                | TokenKind::Hash
                | TokenKind::Underscore
                | TokenKind::DoubleUnderscore
                | TokenKind::TripleUnderscore
                | TokenKind::LParen
                | TokenKind::LBrace
        )
    }

    pub(super) fn current_binary_op(&self) -> Option<BinaryOp> {
        match self.current().kind {
            TokenKind::MessageName => Some(BinaryOp::MessageName),
            TokenKind::TagSet => Some(BinaryOp::TagSet),
            TokenKind::Set => Some(BinaryOp::Set),
            TokenKind::SetDelayed => Some(BinaryOp::SetDelayed),
            TokenKind::ReplaceAll => Some(BinaryOp::ReplaceAll),
            TokenKind::ReplaceRepeated => Some(BinaryOp::ReplaceRepeated),
            TokenKind::Arrow => Some(BinaryOp::Rule),
            TokenKind::RuleDelayed => Some(BinaryOp::RuleDelayed),
            TokenKind::Condition => Some(BinaryOp::Condition),
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Subtract),
            TokenKind::Star => Some(BinaryOp::Times),
            TokenKind::Slash => Some(BinaryOp::Divide),
            TokenKind::Caret => Some(BinaryOp::Power),
            _ => None,
        }
    }

    pub(super) fn consume_ampersand(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Ampersand) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_unset(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Unset) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_lbracket(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::LBracket) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_double_lbracket(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::DoubleLBracket) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn current_starts_double_rbracket(&self) -> bool {
        matches!(self.current().kind, TokenKind::RBracket)
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|next| matches!(next.kind, TokenKind::RBracket))
    }

    pub(super) fn consume_double_rbracket(&mut self) -> Option<Span> {
        if !self.current_starts_double_rbracket() {
            return None;
        }

        let first = self.bump();
        let second = self.bump();
        Some(first.merge(second))
    }

    pub(super) fn consume_span(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Span) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_blank_kind(&mut self) -> Option<(BlankKind, Span)> {
        let kind = match self.current().kind {
            TokenKind::Underscore => BlankKind::Blank,
            TokenKind::DoubleUnderscore => BlankKind::BlankSequence,
            TokenKind::TripleUnderscore => BlankKind::BlankNullSequence,
            _ => return None,
        };

        let span = self.bump();
        Some((kind, span))
    }

    pub(super) fn expect_comma(&mut self) -> Result<(), SyntaxError> {
        if matches!(self.current().kind, TokenKind::Comma) {
            self.bump();
            Ok(())
        } else if self.at_end() {
            Err(SyntaxError::Incomplete {
                expected: ",",
                span: self.current().span,
            })
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: ",",
                found: self.current_token_label(),
                span: self.current().span,
            })
        }
    }

    pub(super) fn expect_rparen(
        &mut self,
        opening_span: Span,
        opening: &'static str,
    ) -> Result<Span, SyntaxError> {
        if matches!(self.current().kind, TokenKind::RParen) {
            Ok(self.bump())
        } else if self.at_end() {
            Err(SyntaxError::UnclosedDelimiter {
                expected: ")",
                span: self.current().span,
                opening_span,
                opening,
            })
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: ")",
                found: self.current_token_label(),
                span: self.current().span,
            })
        }
    }

    pub(super) fn current_token_label(&self) -> String {
        self.current().kind.label().to_string()
    }

    pub(super) fn double_rbracket_label(&self) -> String {
        "]]".to_string()
    }

    pub(super) fn token_starts_expression(&self) -> bool {
        self.token_starts_primary()
            || matches!(self.current().kind, TokenKind::Plus | TokenKind::Minus)
    }

    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub(super) fn bump(&mut self) -> Span {
        let span = self.current().span;

        if !self.at_end() {
            self.index += 1;
        }

        span
    }

    pub(super) fn at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::End)
    }
}
