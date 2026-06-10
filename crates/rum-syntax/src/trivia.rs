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
    StarEqual,
    Slash,
    SlashAt,
    SlashEqual,
    SlashSlashEqual,
    SlashStar,
    Caret,
    Bang,
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
    Ampersand,
    Bar,
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
            Self::StarEqual => "*=",
            Self::Slash => "/",
            Self::SlashAt => "/@",
            Self::SlashEqual => "/=",
            Self::SlashSlashEqual => "//=",
            Self::SlashStar => "/*",
            Self::Caret => "^",
            Self::Bang => "!",
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
            Self::Ampersand => "&",
            Self::Bar => "|",
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
