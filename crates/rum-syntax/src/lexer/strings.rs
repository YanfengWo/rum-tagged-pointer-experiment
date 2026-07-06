use super::operators::unicode_operator_token_kind;
use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn lex_string(&mut self) -> Result<TokenKind, SyntaxError> {
        let start = self.offset;
        if self.consume_str("\"\"\"") {
            return self.lex_escaped_multiline_string(start);
        }

        self.bump_char();
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    return Ok(TokenKind::String(value));
                }
                '\\' => {
                    self.lex_string_escape(start, &mut value)?;
                }
                other => {
                    value.push(other);
                    self.bump_char();
                }
            }
        }

        Err(SyntaxError::UnterminatedString {
            span: Span::new(start, self.offset),
        })
    }

    pub(super) fn lex_raw_multiline_string(&mut self) -> Result<TokenKind, SyntaxError> {
        let start = self.offset;
        self.offset += 3;
        self.consume_initial_multiline_newline();

        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            if self.input[self.offset..].starts_with("'''") {
                self.offset += 3;
                return Ok(TokenKind::String(value));
            }

            value.push(ch);
            self.bump_char();
        }

        Err(SyntaxError::UnterminatedString {
            span: Span::new(start, self.offset),
        })
    }

    fn lex_escaped_multiline_string(&mut self, start: usize) -> Result<TokenKind, SyntaxError> {
        self.consume_initial_multiline_newline();

        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            if self.input[self.offset..].starts_with("\"\"\"") {
                self.offset += 3;
                return Ok(TokenKind::String(value));
            }

            match ch {
                '\\' => self.lex_string_escape(start, &mut value)?,
                other => {
                    value.push(other);
                    self.bump_char();
                }
            }
        }

        Err(SyntaxError::UnterminatedString {
            span: Span::new(start, self.offset),
        })
    }

    fn consume_initial_multiline_newline(&mut self) {
        if self.input[self.offset..].starts_with("\r\n") {
            self.offset += 2;
        } else if self.input[self.offset..].starts_with('\n') {
            self.offset += 1;
        }
    }

    fn lex_string_escape(
        &mut self,
        string_start: usize,
        value: &mut String,
    ) -> Result<(), SyntaxError> {
        self.bump_char();

        let escaped = self.peek_char().ok_or(SyntaxError::UnterminatedString {
            span: Span::new(string_start, self.offset),
        })?;

        match escaped {
            '"' | '\\' => {
                value.push(escaped);
                self.bump_char();
            }
            '[' => {
                let escape_start = self.offset - 1;
                let Some((decoded, end)) = self.named_character_escape_at(escape_start)? else {
                    unreachable!("string escape branch checked '['");
                };
                value.push(decoded);
                self.offset = end;
            }
            'n' => {
                value.push('\n');
                self.bump_char();
            }
            't' => {
                value.push('\t');
                self.bump_char();
            }
            'r' => {
                value.push('\r');
                self.bump_char();
            }
            'u' => {
                if let Some(unicode) = self.try_lex_unicode_escape() {
                    value.push(unicode);
                } else {
                    value.push('u');
                    self.bump_char();
                }
            }
            other => {
                value.push(other);
                self.bump_char();
            }
        }

        Ok(())
    }

    pub(super) fn lex_named_character_token(
        &mut self,
        start: usize,
    ) -> Result<TokenKind, SyntaxError> {
        let Some((decoded, end)) = self.named_character_escape_at(start)? else {
            return Err(SyntaxError::UnexpectedCharacter {
                found: '\\',
                span: Span::new(start, start + 1),
            });
        };

        self.offset = end;

        if let Some(kind) = unicode_operator_token_kind(decoded) {
            return Ok(kind);
        }

        let mut value = String::new();
        value.push(decoded);
        self.lex_symbol_continuation(value)
    }

    pub(super) fn named_character_escape_at(
        &self,
        start: usize,
    ) -> Result<Option<(char, usize)>, SyntaxError> {
        if !self.input[start..].starts_with("\\[") {
            return Ok(None);
        }

        let name_start = start + 2;
        let mut scan = name_start;

        while scan < self.input.len() {
            let ch = self.input[scan..].chars().next().expect("scan in bounds");
            if ch == ']' {
                let end = scan + 1;
                let name = &self.input[name_start..scan];
                let Some(decoded) = named_character(name) else {
                    return Err(SyntaxError::UnexpectedCharacter {
                        found: '\\',
                        span: Span::new(start, end),
                    });
                };
                return Ok(Some((decoded, end)));
            }
            scan += ch.len_utf8();
        }

        Err(SyntaxError::UnexpectedCharacter {
            found: '\\',
            span: Span::new(start, self.input.len()),
        })
    }

    fn try_lex_unicode_escape(&mut self) -> Option<char> {
        if !self.input[self.offset..].starts_with("u{") {
            return None;
        }

        let original_offset = self.offset;
        self.offset += 2;
        let digits_start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_hexdigit()) {
            self.bump_char();
        }

        if self.offset == digits_start || self.peek_char() != Some('}') {
            self.offset = original_offset;
            return None;
        }

        let digits = &self.input[digits_start..self.offset];
        self.bump_char();

        let value = u32::from_str_radix(digits, 16).ok()?;
        char::from_u32(value)
    }
}
