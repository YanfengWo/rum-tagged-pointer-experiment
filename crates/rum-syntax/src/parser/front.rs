use super::*;

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let tokens = lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse_root()
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
                offset: self.current().start,
            })
        }
    }

    pub(super) fn parse_function_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_binary_expr(0)?;

        while self.consume_ampersand() {
            expr = SyntaxExpr::Postfix {
                op: PostfixOp::Function,
                expr: Box::new(expr),
            };
        }

        Ok(expr)
    }

    pub(super) fn parse_binary_expr(
        &mut self,
        min_precedence: u8,
    ) -> Result<SyntaxExpr, SyntaxError> {
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

        while self.consume_unset() {
            lhs = SyntaxExpr::Postfix {
                op: PostfixOp::Unset,
                expr: Box::new(lhs),
            };
        }

        Ok(lhs)
    }

    pub(super) fn parse_prefix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
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

    pub(super) fn parse_postfix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
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

    pub(super) fn parse_primary(&mut self) -> Result<SyntaxExpr, SyntaxError> {
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
}
