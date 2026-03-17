use std::fmt;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxExpr {
    Integer(String),
    String(String),
    Symbol(String),
    Call {
        head: Box<SyntaxExpr>,
        args: Vec<SyntaxExpr>,
    },
}

impl fmt::Display for SyntaxExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => f.write_str(value),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::Symbol(value) => f.write_str(value),
            Self::Call { head, args } => {
                write!(f, "{head}[")?;

                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                f.write_str("]")
            }
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SyntaxError {
    #[error("input is empty")]
    EmptyInput,
    #[error("expected {expected} at byte {offset}")]
    Expected {
        expected: &'static str,
        offset: usize,
    },
    #[error("trailing input at byte {offset}")]
    TrailingInput { offset: usize },
    #[error("unexpected character `{found}` at byte {offset}")]
    UnexpectedCharacter { found: char, offset: usize },
    #[error("unterminated string starting at byte {offset}")]
    UnterminatedString { offset: usize },
}

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr()?;
    parser.skip_whitespace();

    if parser.is_eof() {
        Ok(expr)
    } else {
        Err(SyntaxError::TrailingInput {
            offset: parser.offset,
        })
    }
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }

    fn parse_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        self.skip_whitespace();

        if self.is_eof() {
            return Err(SyntaxError::EmptyInput);
        }

        let mut expr = self.parse_atom()?;

        loop {
            self.skip_whitespace();

            if !self.consume('[') {
                break;
            }

            let mut args = Vec::new();
            self.skip_whitespace();

            if !self.consume(']') {
                loop {
                    args.push(self.parse_expr()?);
                    self.skip_whitespace();

                    if self.consume(']') {
                        break;
                    }

                    self.expect(',', ",")?;
                }
            }

            expr = SyntaxExpr::Call {
                head: Box::new(expr),
                args,
            };
        }

        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        self.skip_whitespace();

        match self.peek_char() {
            Some('"') => self.parse_string(),
            Some(ch) if self.looks_like_integer_start(ch) => self.parse_integer(),
            Some(ch) if is_symbol_start(ch) => self.parse_symbol(),
            Some(found) => Err(SyntaxError::UnexpectedCharacter {
                found,
                offset: self.offset,
            }),
            None => Err(SyntaxError::EmptyInput),
        }
    }

    fn parse_integer(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start = self.offset;

        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.bump_char();
        }

        let digits_start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }

        if self.offset == digits_start {
            return Err(SyntaxError::Expected {
                expected: "digit",
                offset: self.offset,
            });
        }

        Ok(SyntaxExpr::Integer(
            self.input[start..self.offset].to_string(),
        ))
    }

    fn parse_string(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start = self.offset;
        self.expect('"', "\"")?;
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    return Ok(SyntaxExpr::String(value));
                }
                '\\' => {
                    self.bump_char();

                    let escaped = self
                        .peek_char()
                        .ok_or(SyntaxError::UnterminatedString { offset: start })?;

                    match escaped {
                        '"' | '\\' => {
                            value.push(escaped);
                            self.bump_char();
                        }
                        'n' => {
                            value.push('\n');
                            self.bump_char();
                        }
                        't' => {
                            value.push('\t');
                            self.bump_char();
                        }
                        other => {
                            value.push(other);
                            self.bump_char();
                        }
                    }
                }
                other => {
                    value.push(other);
                    self.bump_char();
                }
            }
        }

        Err(SyntaxError::UnterminatedString { offset: start })
    }

    fn parse_symbol(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start = self.offset;
        self.bump_char();

        while matches!(self.peek_char(), Some(ch) if is_symbol_continue(ch)) {
            self.bump_char();
        }

        Ok(SyntaxExpr::Symbol(
            self.input[start..self.offset].to_string(),
        ))
    }

    fn looks_like_integer_start(&self, ch: char) -> bool {
        if ch.is_ascii_digit() {
            return true;
        }

        if matches!(ch, '+' | '-') {
            if let Some(next) = self.peek_next_char() {
                return next.is_ascii_digit();
            }
        }

        false
    }

    fn expect(&mut self, expected: char, label: &'static str) -> Result<(), SyntaxError> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(SyntaxError::Expected {
                expected: label,
                offset: self.offset,
            })
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_whitespace()) {
            self.bump_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.input.len()
    }
}

fn is_symbol_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '$'
}

fn is_symbol_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '$' | '`')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_large_integer_literal() {
        let parsed = parse_expr("123456789012345678901234567890").unwrap();
        assert_eq!(
            parsed,
            SyntaxExpr::Integer("123456789012345678901234567890".to_string())
        );
    }

    #[test]
    fn parses_nested_call() {
        let parsed = parse_expr("f[g[1], x]").unwrap();
        assert_eq!(parsed.to_string(), "f[g[1], x]");
    }

    #[test]
    fn parses_escaped_string() {
        let parsed = parse_expr("\"hello\\nworld\"").unwrap();
        assert_eq!(parsed, SyntaxExpr::String("hello\nworld".to_string()));
    }
}
