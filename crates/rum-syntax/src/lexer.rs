use crate::error::SyntaxError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Integer(String),
    String(String),
    Symbol(String),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Arrow,
    RuleDelayed,
    Set,
    SetDelayed,
    ReplaceAll,
    Condition,
    Ampersand,
    Hash,
    Underscore,
    DoubleUnderscore,
    TripleUnderscore,
    End,
}

pub(crate) fn lex(input: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    while let Some(token) = lexer.next_token()? {
        let is_end = matches!(token.kind, TokenKind::End);
        tokens.push(token);

        if is_end {
            break;
        }
    }

    Ok(tokens)
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }

    fn next_token(&mut self) -> Result<Option<Token>, SyntaxError> {
        self.skip_whitespace();

        if self.is_eof() {
            return Ok(Some(Token {
                kind: TokenKind::End,
                start: self.offset,
                end: self.offset,
            }));
        }

        let start = self.offset;
        let ch = self.peek_char().expect("checked eof above");

        let token = match ch {
            '[' => self.single(TokenKind::LBracket),
            ']' => self.single(TokenKind::RBracket),
            '{' => self.single(TokenKind::LBrace),
            '}' => self.single(TokenKind::RBrace),
            '(' => self.single(TokenKind::LParen),
            ')' => self.single(TokenKind::RParen),
            ',' => self.single(TokenKind::Comma),
            '+' => self.single(TokenKind::Plus),
            '*' => self.single(TokenKind::Star),
            '^' => self.single(TokenKind::Caret),
            '&' => self.single(TokenKind::Ampersand),
            '#' => self.single(TokenKind::Hash),
            '-' => {
                if self.consume_str("->") {
                    TokenKind::Arrow
                } else {
                    self.single(TokenKind::Minus)
                }
            }
            ':' => {
                if self.consume_str(":=") {
                    TokenKind::SetDelayed
                } else if self.consume_str(":>") {
                    TokenKind::RuleDelayed
                } else {
                    return Err(SyntaxError::UnexpectedCharacter {
                        found: ch,
                        offset: start,
                    });
                }
            }
            '=' => self.single(TokenKind::Set),
            '/' => {
                if self.consume_str("/.") {
                    TokenKind::ReplaceAll
                } else if self.consume_str("/;") {
                    TokenKind::Condition
                } else {
                    self.single(TokenKind::Slash)
                }
            }
            '_' => {
                if self.consume_str("___") {
                    TokenKind::TripleUnderscore
                } else if self.consume_str("__") {
                    TokenKind::DoubleUnderscore
                } else {
                    self.single(TokenKind::Underscore)
                }
            }
            '"' => self.lex_string()?,
            ch if ch.is_ascii_digit() => self.lex_integer(),
            ch if is_symbol_start(ch) => self.lex_symbol(),
            other => {
                return Err(SyntaxError::UnexpectedCharacter {
                    found: other,
                    offset: start,
                });
            }
        };

        Ok(Some(Token {
            kind: token,
            start,
            end: self.offset,
        }))
    }

    fn lex_integer(&mut self) -> TokenKind {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }

        TokenKind::Integer(self.input[start..self.offset].to_string())
    }

    fn lex_symbol(&mut self) -> TokenKind {
        let start = self.offset;
        self.bump_char();

        while matches!(self.peek_char(), Some(ch) if is_symbol_continue(ch)) {
            self.bump_char();
        }

        TokenKind::Symbol(self.input[start..self.offset].to_string())
    }

    fn lex_string(&mut self) -> Result<TokenKind, SyntaxError> {
        let start = self.offset;
        self.bump_char();
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    return Ok(TokenKind::String(value));
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

    fn single(&mut self, kind: TokenKind) -> TokenKind {
        self.bump_char();
        kind
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.offset..].starts_with(expected) {
            self.offset += expected.len();
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
    fn lexes_multi_char_operators() {
        let tokens = lex("x := y /. z -> w /; p").unwrap();

        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::SetDelayed))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::ReplaceAll))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Arrow))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Condition))
        );
    }

    #[test]
    fn lexes_context_qualified_symbol() {
        let tokens = lex("System`List").unwrap();
        assert!(matches!(
            &tokens[0].kind,
            TokenKind::Symbol(value) if value == "System`List"
        ));
    }
}
