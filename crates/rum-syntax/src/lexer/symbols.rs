use super::named_characters::is_named_character_symbol_char;
use super::operators::unicode_operator_token_kind;
use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn lex_symbol(&mut self) -> Result<TokenKind, SyntaxError> {
        let mut value = String::new();
        value.push(self.bump_char().expect("symbol start"));
        self.lex_symbol_continuation(value)
    }

    pub(super) fn lex_symbol_continuation(
        &mut self,
        mut value: String,
    ) -> Result<TokenKind, SyntaxError> {
        while let Some(ch) = self.peek_char() {
            if is_symbol_continue(ch) {
                value.push(ch);
                self.bump_char();
                continue;
            }

            if self.input[self.offset..].starts_with("\\[") {
                let Some((decoded, end)) = self.named_character_escape_at(self.offset)? else {
                    unreachable!("starts_with checked above");
                };

                if unicode_operator_token_kind(decoded).is_none() {
                    value.push(decoded);
                    self.offset = end;
                    continue;
                }
            }

            break;
        }

        Ok(TokenKind::Symbol(value))
    }
}

pub(super) fn is_symbol_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '$' || is_named_character_symbol_char(ch)
}

fn is_symbol_continue(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '$' | '`') || is_named_character_symbol_char(ch)
}
