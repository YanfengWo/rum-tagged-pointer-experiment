use super::*;

impl Parser {
    pub(super) fn parse_slot(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start_span = self.bump();
        let mut end_span = start_span;
        let is_sequence = matches!(self.current().kind, TokenKind::Hash);

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
    ) -> Result<(Vec<SyntaxExpr>, Span), SyntaxError> {
        let mut items = Vec::new();

        if self.current().kind == end_kind {
            let end_span = self.bump();
            return Ok((items, end_span));
        }

        if self.at_end() {
            return Err(SyntaxError::Incomplete {
                expected: end_label,
                span: self.current().span,
            });
        }

        loop {
            items.push(self.parse_function_expr()?);

            if self.current().kind == end_kind {
                let end_span = self.bump();
                return Ok((items, end_span));
            }

            if self.at_end() {
                return Err(SyntaxError::Incomplete {
                    expected: end_label,
                    span: self.current().span,
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
                return Err(SyntaxError::Incomplete {
                    expected: end_label,
                    span: self.current().span,
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

    pub(super) fn expect_rparen(&mut self) -> Result<Span, SyntaxError> {
        if matches!(self.current().kind, TokenKind::RParen) {
            Ok(self.bump())
        } else if self.at_end() {
            Err(SyntaxError::Incomplete {
                expected: ")",
                span: self.current().span,
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
        token_label(&self.current().kind).to_string()
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

fn token_label(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Integer(_) => "integer",
        TokenKind::String(_) => "string",
        TokenKind::Symbol(_) => "symbol",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::Comma => ",",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Caret => "^",
        TokenKind::MessageName => "::",
        TokenKind::TagSet => "/:",
        TokenKind::Arrow => "->",
        TokenKind::RuleDelayed => ":>",
        TokenKind::Set => "=",
        TokenKind::Unset => "=.",
        TokenKind::SetDelayed => ":=",
        TokenKind::ReplaceAll => "/.",
        TokenKind::ReplaceRepeated => "//.",
        TokenKind::Condition => "/;",
        TokenKind::Ampersand => "&",
        TokenKind::Hash => "#",
        TokenKind::Underscore => "_",
        TokenKind::DoubleUnderscore => "__",
        TokenKind::TripleUnderscore => "___",
        TokenKind::End => "end of input",
    }
}
