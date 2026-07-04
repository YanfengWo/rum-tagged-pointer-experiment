use super::*;

pub(super) fn unicode_operator_token_kind(ch: char) -> Option<TokenKind> {
    match ch {
        '\u{2227}' => Some(TokenKind::And),
        '\u{2228}' => Some(TokenKind::Or),
        '\u{ac}' => Some(TokenKind::Bang),
        '\u{f431}' => Some(TokenKind::Equal),
        '\u{2260}' => Some(TokenKind::Unequal),
        '\u{2264}' => Some(TokenKind::LessEqual),
        '\u{2265}' => Some(TokenKind::GreaterEqual),
        '\u{d7}' => Some(TokenKind::Star),
        '\u{f7}' => Some(TokenKind::Slash),
        '\u{f522}' => Some(TokenKind::Arrow),
        '\u{f51f}' => Some(TokenKind::RuleDelayed),
        '\u{f4a1}' => Some(TokenKind::Ampersand),
        _ => None,
    }
}
