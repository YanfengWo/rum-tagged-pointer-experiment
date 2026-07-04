use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn lex_number(&mut self) -> TokenKind {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }

        let has_decimal_point = self.peek_char() == Some('.') && self.peek_next_char() != Some('.');
        if has_decimal_point {
            self.bump_char();
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
                self.bump_char();
            }
            self.consume_decimal_exponent();
        }

        TokenKind::Integer(self.input[start..self.offset].to_string())
    }

    pub(super) fn lex_dot_prefixed_token(&mut self) -> TokenKind {
        if self.peek_next_char().is_some_and(|ch| ch.is_ascii_digit()) {
            let start = self.offset;
            self.bump_char();
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
                self.bump_char();
            }
            self.consume_decimal_exponent();
            return TokenKind::Integer(self.input[start..self.offset].to_string());
        }

        if self.consume_str("...") {
            TokenKind::RepeatedNull
        } else if self.consume_str("..") {
            TokenKind::Repeated
        } else {
            self.single(TokenKind::Dot)
        }
    }

    fn consume_decimal_exponent(&mut self) {
        let Some(exponent_marker) = self.peek_char() else {
            return;
        };
        if !matches!(exponent_marker, 'e' | 'E') {
            return;
        }

        let mut scan = self.offset + exponent_marker.len_utf8();
        if let Some(sign) = self.input[scan..].chars().next() {
            if matches!(sign, '+' | '-') {
                scan += sign.len_utf8();
            }
        }

        let digits_start = scan;
        while let Some(ch) = self.input[scan..].chars().next() {
            if !ch.is_ascii_digit() {
                break;
            }
            scan += ch.len_utf8();
        }

        if scan > digits_start {
            self.offset = scan;
        }
    }
}
