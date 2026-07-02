use std::ops::Range;

use crate::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: SyntaxTokenKind,
    pub span: Span,
    pub leading_trivia: Range<usize>,
}

impl SyntaxToken {
    pub fn new(kind: SyntaxTokenKind, span: Span, leading_trivia: Range<usize>) -> Self {
        Self {
            kind,
            span,
            leading_trivia,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxTokenKind {
    Integer(String),
    String(String),
    Symbol(String),
    LBracket,
    RBracket,
    DoubleLBracket,
    DoubleRBracket,
    LBrace,
    RBrace,
    LAssociation,
    RAssociation,
    LParen,
    RParen,
    Comma,
    Semicolon,
    Span,
    At,
    AtAt,
    AtAtAt,
    AtStar,
    Plus,
    PlusPlus,
    PlusEqual,
    Minus,
    MinusMinus,
    MinusEqual,
    Star,
    StarStar,
    StarEqual,
    Slash,
    SlashAt,
    SlashEqual,
    SlashSlash,
    SlashSlashAt,
    SlashSlashEqual,
    SlashStar,
    TildeTilde,
    Tilde,
    Caret,
    CaretEqual,
    CaretColonEqual,
    Bang,
    Prime,
    And,
    Or,
    Same,
    Unsame,
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    StringJoin,
    Colon,
    MessageName,
    TagSet,
    Arrow,
    RuleDelayed,
    Set,
    Unset,
    SetDelayed,
    ReplaceAll,
    ReplaceRepeated,
    Condition,
    Dot,
    Repeated,
    RepeatedNull,
    Ampersand,
    Bar,
    BarMinusGreater,
    Question,
    Hash,
    Underscore,
    DoubleUnderscore,
    TripleUnderscore,
    End,
}

impl SyntaxTokenKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Integer(_) => "integer",
            Self::String(_) => "string",
            Self::Symbol(_) => "symbol",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::DoubleLBracket => "[[",
            Self::DoubleRBracket => "]]",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LAssociation => "<|",
            Self::RAssociation => "|>",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::Comma => ",",
            Self::Semicolon => ";",
            Self::Span => ";;",
            Self::At => "@",
            Self::AtAt => "@@",
            Self::AtAtAt => "@@@",
            Self::AtStar => "@*",
            Self::Plus => "+",
            Self::PlusPlus => "++",
            Self::PlusEqual => "+=",
            Self::Minus => "-",
            Self::MinusMinus => "--",
            Self::MinusEqual => "-=",
            Self::Star => "*",
            Self::StarStar => "**",
            Self::StarEqual => "*=",
            Self::Slash => "/",
            Self::SlashAt => "/@",
            Self::SlashEqual => "/=",
            Self::SlashSlash => "//",
            Self::SlashSlashAt => "//@",
            Self::SlashSlashEqual => "//=",
            Self::SlashStar => "/*",
            Self::TildeTilde => "~~",
            Self::Tilde => "~",
            Self::Caret => "^",
            Self::CaretEqual => "^=",
            Self::CaretColonEqual => "^:=",
            Self::Bang => "!",
            Self::Prime => "'",
            Self::And => "&&",
            Self::Or => "||",
            Self::Same => "===",
            Self::Unsame => "=!=",
            Self::Equal => "==",
            Self::Unequal => "!=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::LessEqual => "<=",
            Self::GreaterEqual => ">=",
            Self::StringJoin => "<>",
            Self::Colon => ":",
            Self::MessageName => "::",
            Self::TagSet => "/:",
            Self::Arrow => "->",
            Self::RuleDelayed => ":>",
            Self::Set => "=",
            Self::Unset => "=.",
            Self::SetDelayed => ":=",
            Self::ReplaceAll => "/.",
            Self::ReplaceRepeated => "//.",
            Self::Condition => "/;",
            Self::Dot => ".",
            Self::Repeated => "..",
            Self::RepeatedNull => "...",
            Self::Ampersand => "&",
            Self::Bar => "|",
            Self::BarMinusGreater => "|->",
            Self::Question => "?",
            Self::Hash => "#",
            Self::Underscore => "_",
            Self::DoubleUnderscore => "__",
            Self::TripleUnderscore => "___",
            Self::End => "end of input",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    Comment,
}

impl TriviaKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Whitespace => "whitespace",
            Self::Comment => "comment",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriviaPiece {
    pub kind: TriviaKind,
    pub span: Span,
    pub text: String,
}

impl TriviaPiece {
    pub fn new(kind: TriviaKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }

    pub fn whitespace(span: Span, text: impl Into<String>) -> Self {
        Self::new(TriviaKind::Whitespace, span, text.into())
    }

    pub fn comment(span: Span, text: impl Into<String>) -> Self {
        Self::new(TriviaKind::Comment, span, text.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexedSource {
    tokens: Vec<SyntaxToken>,
    trivia: Vec<TriviaPiece>,
}

impl LexedSource {
    pub fn new(tokens: Vec<SyntaxToken>, trivia: Vec<TriviaPiece>) -> Self {
        Self { tokens, trivia }
    }

    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }

    pub fn trivia(&self) -> &[TriviaPiece] {
        &self.trivia
    }

    pub fn leading_trivia(&self, token_index: usize) -> Option<&[TriviaPiece]> {
        let token = self.tokens.get(token_index)?;
        Some(&self.trivia[token.leading_trivia.clone()])
    }

    pub fn into_tokens(self) -> Vec<SyntaxToken> {
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{format_source, lex_with_trivia};

    #[test]
    fn syntax_token_kind_labels_cover_atoms_punctuation_and_operators() {
        let cases = vec![
            (SyntaxTokenKind::Integer("1".to_string()), "integer"),
            (SyntaxTokenKind::String("s".to_string()), "string"),
            (SyntaxTokenKind::Symbol("x".to_string()), "symbol"),
            (SyntaxTokenKind::LBracket, "["),
            (SyntaxTokenKind::RBracket, "]"),
            (SyntaxTokenKind::DoubleLBracket, "[["),
            (SyntaxTokenKind::DoubleRBracket, "]]"),
            (SyntaxTokenKind::LBrace, "{"),
            (SyntaxTokenKind::RBrace, "}"),
            (SyntaxTokenKind::LAssociation, "<|"),
            (SyntaxTokenKind::RAssociation, "|>"),
            (SyntaxTokenKind::LParen, "("),
            (SyntaxTokenKind::RParen, ")"),
            (SyntaxTokenKind::Comma, ","),
            (SyntaxTokenKind::Semicolon, ";"),
            (SyntaxTokenKind::Span, ";;"),
            (SyntaxTokenKind::At, "@"),
            (SyntaxTokenKind::AtAt, "@@"),
            (SyntaxTokenKind::AtAtAt, "@@@"),
            (SyntaxTokenKind::AtStar, "@*"),
            (SyntaxTokenKind::Plus, "+"),
            (SyntaxTokenKind::PlusPlus, "++"),
            (SyntaxTokenKind::PlusEqual, "+="),
            (SyntaxTokenKind::Minus, "-"),
            (SyntaxTokenKind::MinusMinus, "--"),
            (SyntaxTokenKind::MinusEqual, "-="),
            (SyntaxTokenKind::Star, "*"),
            (SyntaxTokenKind::StarStar, "**"),
            (SyntaxTokenKind::StarEqual, "*="),
            (SyntaxTokenKind::Slash, "/"),
            (SyntaxTokenKind::SlashAt, "/@"),
            (SyntaxTokenKind::SlashEqual, "/="),
            (SyntaxTokenKind::SlashSlash, "//"),
            (SyntaxTokenKind::SlashSlashAt, "//@"),
            (SyntaxTokenKind::SlashSlashEqual, "//="),
            (SyntaxTokenKind::SlashStar, "/*"),
            (SyntaxTokenKind::TildeTilde, "~~"),
            (SyntaxTokenKind::Tilde, "~"),
            (SyntaxTokenKind::Caret, "^"),
            (SyntaxTokenKind::CaretEqual, "^="),
            (SyntaxTokenKind::CaretColonEqual, "^:="),
            (SyntaxTokenKind::Bang, "!"),
            (SyntaxTokenKind::Prime, "'"),
            (SyntaxTokenKind::And, "&&"),
            (SyntaxTokenKind::Or, "||"),
            (SyntaxTokenKind::Same, "==="),
            (SyntaxTokenKind::Unsame, "=!="),
            (SyntaxTokenKind::Equal, "=="),
            (SyntaxTokenKind::Unequal, "!="),
            (SyntaxTokenKind::Less, "<"),
            (SyntaxTokenKind::Greater, ">"),
            (SyntaxTokenKind::LessEqual, "<="),
            (SyntaxTokenKind::GreaterEqual, ">="),
            (SyntaxTokenKind::StringJoin, "<>"),
            (SyntaxTokenKind::Colon, ":"),
            (SyntaxTokenKind::MessageName, "::"),
            (SyntaxTokenKind::TagSet, "/:"),
            (SyntaxTokenKind::Arrow, "->"),
            (SyntaxTokenKind::RuleDelayed, ":>"),
            (SyntaxTokenKind::Set, "="),
            (SyntaxTokenKind::Unset, "=."),
            (SyntaxTokenKind::SetDelayed, ":="),
            (SyntaxTokenKind::ReplaceAll, "/."),
            (SyntaxTokenKind::ReplaceRepeated, "//."),
            (SyntaxTokenKind::Condition, "/;"),
            (SyntaxTokenKind::Dot, "."),
            (SyntaxTokenKind::Repeated, ".."),
            (SyntaxTokenKind::RepeatedNull, "..."),
            (SyntaxTokenKind::Ampersand, "&"),
            (SyntaxTokenKind::Bar, "|"),
            (SyntaxTokenKind::BarMinusGreater, "|->"),
            (SyntaxTokenKind::Question, "?"),
            (SyntaxTokenKind::Hash, "#"),
            (SyntaxTokenKind::Underscore, "_"),
            (SyntaxTokenKind::DoubleUnderscore, "__"),
            (SyntaxTokenKind::TripleUnderscore, "___"),
            (SyntaxTokenKind::End, "end of input"),
        ];

        for (kind, label) in cases {
            assert_eq!(kind.label(), label);
        }
    }

    #[test]
    fn trivia_constructors_labels_and_lexed_source_accessors_preserve_ranges() {
        let trivia = vec![
            TriviaPiece::whitespace(Span::new(0, 1), " "),
            TriviaPiece::comment(Span::new(1, 11), "(* note *)"),
            TriviaPiece::whitespace(Span::new(11, 12), "\n"),
        ];
        let tokens = vec![
            SyntaxToken::new(
                SyntaxTokenKind::Symbol("x".to_string()),
                Span::new(12, 13),
                0..2,
            ),
            SyntaxToken::new(SyntaxTokenKind::End, Span::point(13), 2..3),
        ];
        let lexed = LexedSource::new(tokens.clone(), trivia.clone());

        assert_eq!(TriviaKind::Whitespace.label(), "whitespace");
        assert_eq!(TriviaKind::Comment.label(), "comment");
        assert_eq!(lexed.tokens(), tokens.as_slice());
        assert_eq!(lexed.trivia(), trivia.as_slice());
        assert_eq!(lexed.leading_trivia(0), Some(&trivia[0..2]));
        assert_eq!(lexed.leading_trivia(1), Some(&trivia[2..3]));
        assert_eq!(lexed.leading_trivia(2), None);
        assert_eq!(lexed.into_tokens(), tokens);
    }

    #[test]
    fn parsed_comments_attach_to_following_tokens_and_end_token_trivia() {
        let source = "(* header *)\n f[x, (* inline *) y]\n(* tail *)";
        let lexed = lex_with_trivia(source).expect("source should lex");

        assert!(matches!(
            &lexed.tokens()[0].kind,
            SyntaxTokenKind::Symbol(name) if name == "f"
        ));
        assert_eq!(
            lexed
                .leading_trivia(0)
                .expect("f should carry leading header trivia")
                .iter()
                .map(|piece| (piece.kind, piece.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (TriviaKind::Comment, "(* header *)"),
                (TriviaKind::Whitespace, "\n "),
            ]
        );

        assert!(matches!(
            &lexed.tokens()[4].kind,
            SyntaxTokenKind::Symbol(name) if name == "y"
        ));
        assert_eq!(
            lexed
                .leading_trivia(4)
                .expect("y should carry inline comment trivia")
                .iter()
                .map(|piece| (piece.kind, piece.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (TriviaKind::Whitespace, " "),
                (TriviaKind::Comment, "(* inline *)"),
                (TriviaKind::Whitespace, " "),
            ]
        );

        let end_index = lexed.tokens().len() - 1;
        assert!(matches!(
            lexed.tokens()[end_index].kind,
            SyntaxTokenKind::End
        ));
        assert_eq!(
            lexed
                .leading_trivia(end_index)
                .expect("end token should carry trailing trivia")
                .iter()
                .map(|piece| (piece.kind, piece.text.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (TriviaKind::Whitespace, "\n"),
                (TriviaKind::Comment, "(* tail *)"),
            ]
        );

        let formatted = format_source(source).expect("formatting should preserve comments");
        assert_eq!(formatted, "(* header *)\nf[x, (* inline *) y] (* tail *)");
    }
}
