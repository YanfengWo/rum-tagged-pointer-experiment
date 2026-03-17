use crate::ast::{BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxExpr};
use crate::error::SyntaxError;
use crate::lexer::{Token, TokenKind, lex};

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let tokens = lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse_root()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_root(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        if self.at_end() {
            return Err(SyntaxError::EmptyInput);
        }

        let expr = self.parse_function_expr()?;

        if self.at_end() {
            Ok(expr)
        } else {
            Err(SyntaxError::TrailingInput {
                offset: self.current().start,
            })
        }
    }

    fn parse_function_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_binary_expr(0)?;

        while self.consume_ampersand() {
            expr = SyntaxExpr::Postfix {
                op: PostfixOp::Function,
                expr: Box::new(expr),
            };
        }

        Ok(expr)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<SyntaxExpr, SyntaxError> {
        let mut lhs = self.parse_prefix_expr()?;

        loop {
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
            lhs = SyntaxExpr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        match &self.current().kind {
            TokenKind::Plus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Plus.precedence())?;
                Ok(SyntaxExpr::Prefix {
                    op: PrefixOp::Plus,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_binary_expr(PrefixOp::Minus.precedence())?;
                Ok(SyntaxExpr::Prefix {
                    op: PrefixOp::Minus,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.consume_lbracket() {
                let args = self.parse_expression_list(TokenKind::RBracket, "]")?;
                expr = SyntaxExpr::Call {
                    head: Box::new(expr),
                    args,
                };
                continue;
            }

            if let Some(kind) = self.consume_blank_kind() {
                let pattern = self.parse_blank_tail(kind)?;
                expr = SyntaxExpr::Pattern {
                    name: Box::new(expr),
                    pattern: Box::new(pattern),
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let token = self.current().clone();

        match token.kind {
            TokenKind::Integer(value) => {
                self.bump();
                Ok(SyntaxExpr::Integer(value))
            }
            TokenKind::String(value) => {
                self.bump();
                Ok(SyntaxExpr::String(value))
            }
            TokenKind::Symbol(value) => {
                self.bump();
                Ok(SyntaxExpr::Symbol(value))
            }
            TokenKind::Hash => self.parse_slot(),
            TokenKind::Underscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::Blank)
            }
            TokenKind::DoubleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankSequence)
            }
            TokenKind::TripleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankNullSequence)
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_function_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                self.bump();
                let items = self.parse_expression_list(TokenKind::RBrace, "}")?;
                Ok(SyntaxExpr::List(items))
            }
            TokenKind::End => Err(SyntaxError::EmptyInput),
            _ => Err(SyntaxError::Expected {
                expected: "expression",
                offset: token.start,
            }),
        }
    }

    fn parse_slot(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        self.bump();
        let is_sequence = matches!(self.current().kind, TokenKind::Hash);

        if is_sequence {
            self.bump();
        }

        let spec = match (&self.current().kind, is_sequence) {
            (TokenKind::Integer(value), false) => {
                let value = value.clone();
                self.bump();
                SlotSpec::Number(value)
            }
            (TokenKind::Symbol(value), false) => {
                let value = value.clone();
                self.bump();
                SlotSpec::Named(value)
            }
            (TokenKind::Integer(value), true) => {
                let value = value.clone();
                self.bump();
                SlotSpec::SequenceNumber(value)
            }
            _ if is_sequence => SlotSpec::Sequence,
            _ => SlotSpec::Anonymous,
        };

        Ok(SyntaxExpr::Slot(spec))
    }

    fn parse_blank_tail(&mut self, kind: BlankKind) -> Result<SyntaxExpr, SyntaxError> {
        let head = if self.token_starts_primary() {
            Some(Box::new(self.parse_postfix_expr()?))
        } else {
            None
        };

        Ok(SyntaxExpr::Blank { kind, head })
    }

    fn parse_expression_list(
        &mut self,
        end_kind: TokenKind,
        end_label: &'static str,
    ) -> Result<Vec<SyntaxExpr>, SyntaxError> {
        let mut items = Vec::new();

        if self.current().kind == end_kind {
            self.bump();
            return Ok(items);
        }

        loop {
            items.push(self.parse_function_expr()?);

            if self.current().kind == end_kind {
                self.bump();
                return Ok(items);
            }

            self.expect_comma()?;

            if self.current().kind == end_kind {
                return Err(SyntaxError::Expected {
                    expected: "expression",
                    offset: self.current().start,
                });
            }

            if self.at_end() {
                return Err(SyntaxError::Expected {
                    expected: end_label,
                    offset: self.current().start,
                });
            }
        }
    }

    fn token_starts_primary(&self) -> bool {
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

    fn current_binary_op(&self) -> Option<BinaryOp> {
        match self.current().kind {
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

    fn consume_ampersand(&mut self) -> bool {
        if matches!(self.current().kind, TokenKind::Ampersand) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn consume_lbracket(&mut self) -> bool {
        if matches!(self.current().kind, TokenKind::LBracket) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn consume_blank_kind(&mut self) -> Option<BlankKind> {
        let kind = match self.current().kind {
            TokenKind::Underscore => BlankKind::Blank,
            TokenKind::DoubleUnderscore => BlankKind::BlankSequence,
            TokenKind::TripleUnderscore => BlankKind::BlankNullSequence,
            _ => return None,
        };

        self.bump();
        Some(kind)
    }

    fn expect_comma(&mut self) -> Result<(), SyntaxError> {
        if matches!(self.current().kind, TokenKind::Comma) {
            self.bump();
            Ok(())
        } else {
            Err(SyntaxError::Expected {
                expected: ",",
                offset: self.current().start,
            })
        }
    }

    fn expect_rparen(&mut self) -> Result<(), SyntaxError> {
        if matches!(self.current().kind, TokenKind::RParen) {
            self.bump();
            Ok(())
        } else {
            Err(SyntaxError::Expected {
                expected: ")",
                offset: self.current().start,
            })
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn bump(&mut self) {
        if !self.at_end() {
            self.index += 1;
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::End)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{BinaryOp, BlankKind, PostfixOp, SlotSpec, SyntaxExpr};

    use super::parse_expr;

    #[test]
    fn parses_large_integer_literal() {
        let parsed = parse_expr("123456789012345678901234567890").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Integer("123456789012345678901234567890".to_string())
        );
    }

    #[test]
    fn parses_lists_and_nested_calls() {
        let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
        assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
    }

    #[test]
    fn parses_named_patterns() {
        let parsed = parse_expr("x_Integer").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Pattern {
                name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                pattern: Box::new(SyntaxExpr::Blank {
                    kind: BlankKind::Blank,
                    head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
                }),
            }
        );
    }

    #[test]
    fn parses_rule_and_replace_all_precedence() {
        let parsed = parse_expr("expr /. x_Integer -> y").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Binary {
                op: BinaryOp::ReplaceAll,
                lhs: Box::new(SyntaxExpr::Symbol("expr".to_string())),
                rhs: Box::new(SyntaxExpr::Binary {
                    op: BinaryOp::Rule,
                    lhs: Box::new(SyntaxExpr::Pattern {
                        name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                        pattern: Box::new(SyntaxExpr::Blank {
                            kind: BlankKind::Blank,
                            head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
                        }),
                    }),
                    rhs: Box::new(SyntaxExpr::Symbol("y".to_string())),
                }),
            }
        );
    }

    #[test]
    fn parses_replace_repeated_precedence() {
        let parsed = parse_expr("expr //. x_Integer -> y").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Binary {
                op: BinaryOp::ReplaceRepeated,
                lhs: Box::new(SyntaxExpr::Symbol("expr".to_string())),
                rhs: Box::new(SyntaxExpr::Binary {
                    op: BinaryOp::Rule,
                    lhs: Box::new(SyntaxExpr::Pattern {
                        name: Box::new(SyntaxExpr::Symbol("x".to_string())),
                        pattern: Box::new(SyntaxExpr::Blank {
                            kind: BlankKind::Blank,
                            head: Some(Box::new(SyntaxExpr::Symbol("Integer".to_string()))),
                        }),
                    }),
                    rhs: Box::new(SyntaxExpr::Symbol("y".to_string())),
                }),
            }
        );
    }

    #[test]
    fn parses_slots_and_pure_functions() {
        let parsed = parse_expr("#1 + #2 &").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Postfix {
                op: PostfixOp::Function,
                expr: Box::new(SyntaxExpr::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(SyntaxExpr::Slot(SlotSpec::Number("1".to_string()))),
                    rhs: Box::new(SyntaxExpr::Slot(SlotSpec::Number("2".to_string()))),
                }),
            }
        );

        let sequence_parsed = parse_expr("## + ##2 &").unwrap();
        assert_eq!(
            sequence_parsed,
            SyntaxExpr::Postfix {
                op: PostfixOp::Function,
                expr: Box::new(SyntaxExpr::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(SyntaxExpr::Slot(SlotSpec::Sequence)),
                    rhs: Box::new(SyntaxExpr::Slot(SlotSpec::SequenceNumber("2".to_string()))),
                }),
            }
        );

        let named_parsed = parse_expr("#x &").unwrap();
        assert_eq!(
            named_parsed,
            SyntaxExpr::Postfix {
                op: PostfixOp::Function,
                expr: Box::new(SyntaxExpr::Slot(SlotSpec::Named("x".to_string()))),
            }
        );
    }

    #[test]
    fn parses_prefix_minus_with_power_precedence() {
        let parsed = parse_expr("-x^2").unwrap();
        assert_eq!(parsed.to_string(), "-x ^ 2");
    }
}
