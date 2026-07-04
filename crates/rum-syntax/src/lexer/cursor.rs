use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }
    pub(super) fn single(&mut self, kind: TokenKind) -> TokenKind {
        self.bump_char();
        kind
    }

    pub(super) fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.offset..].starts_with(expected) {
            self.offset += expected.len();
            true
        } else {
            false
        }
    }

    pub(super) fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    pub(super) fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    pub(super) fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    pub(super) fn is_eof(&self) -> bool {
        self.offset >= self.input.len()
    }
}
