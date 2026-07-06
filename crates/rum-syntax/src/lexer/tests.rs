use super::*;
use crate::TriviaKind;

#[test]
fn lexes_multi_char_operators() {
    let tokens = lex(
        "x::tag := y //. z /. q -> w /; p f /: g =. a[[1]] b;;c <|d -> e|> h@i j@@k l@@@m n@*o p/*q r/@s aa^=bb cc^:=dd ee**ff gg//@hh ii|->jj a&&b c||d !e f===g h=!=i j==k l!=m n<o p>q r<=s t>=u v:_ \"a\"<>\"b\"",
    )
    .unwrap();

    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::MessageName))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::SetDelayed))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::ReplaceRepeated))
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
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::TagSet))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Unset))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::DoubleLBracket))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Span))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::LAssociation))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::RAssociation))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::At))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::AtAt))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::AtAtAt))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::AtStar))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::SlashStar))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::SlashAt))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::CaretEqual))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::CaretColonEqual))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::StarStar))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::SlashSlashAt))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::BarMinusGreater))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::And))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Or))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Bang))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Same))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Unsame))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Equal))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Unequal))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Less))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Greater))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::LessEqual))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::GreaterEqual))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Colon))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::StringJoin))
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

#[test]
fn lexes_unicode_letter_symbols_used_by_package_source() {
    let source = "Module[{\u{03C3} = <||>, \u{03B1}Ty = tyVar[\"A\"]}, \u{03C3}[\u{03B1}Ty]]";
    let tokens = lex(source).unwrap();
    let sigma = '\u{03C3}'.to_string();
    let alpha_ty = format!("{}Ty", '\u{03B1}');

    assert!(
        tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Symbol(value) if value == &sigma))
    );
    assert!(
        tokens
            .iter()
            .any(|token| matches!(&token.kind, TokenKind::Symbol(value) if value == &alpha_ty))
    );
}

#[test]
fn decodes_named_character_escapes_in_strings() {
    let tokens = lex(r#""\[Exists]\[ForAll]\[And]\[Or]\[Not]\[DoubleRightArrow]""#).unwrap();

    assert!(matches!(
        &tokens[0].kind,
        TokenKind::String(value) if value == "∃∀∧∨¬⇒" && value.chars().count() == 6
    ));
}

#[test]
fn decodes_named_character_escapes_in_code_symbols() {
    let tokens = lex(r"\[Alpha] x\[Beta] \[CapitalOmega]1 \[Exists]").unwrap();

    assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "α"));
    assert!(matches!(&tokens[1].kind, TokenKind::Symbol(value) if value == "xβ"));
    assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "Ω1"));
    assert!(matches!(&tokens[3].kind, TokenKind::Symbol(value) if value == "∃"));
}

#[test]
fn decodes_named_character_operator_with_source_span() {
    let tokens = lex(r"\[Alpha] \[And] \[Beta]").unwrap();

    assert_eq!(tokens[0].span, Span::new(0, 8));
    assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "α"));
    assert_eq!(tokens[1].span, Span::new(9, 15));
    assert!(matches!(tokens[1].kind, TokenKind::And));
    assert_eq!(tokens[2].span, Span::new(16, 23));
    assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "β"));
}

#[test]
fn reports_unknown_named_character_escapes() {
    let err = lex(r#""\[NotARealName]""#).unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnexpectedCharacter {
            found: '\\',
            span: Span::new(1, 16),
        }
    );

    let err = lex(r"\[NotARealName]").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnexpectedCharacter {
            found: '\\',
            span: Span::new(0, 15),
        }
    );
}

#[test]
fn skips_comments_and_nested_comments_as_trivia() {
    let tokens = lex("f[(* outer (* inner *) *)x, y]").unwrap();

    assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "f"));
    assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "x"));
    assert!(matches!(&tokens[4].kind, TokenKind::Symbol(value) if value == "y"));
}

#[test]
fn treats_initial_shebang_as_script_comment_trivia() {
    let source = "#!/usr/bin/env rum\nPrint[1]";
    let lexed = lex_with_trivia(source).unwrap();
    let trivia = lexed.trivia();

    assert_eq!(trivia[0].kind, TriviaKind::Comment);
    assert_eq!(trivia[0].text, "#!/usr/bin/env rum");

    let tokens = lex(source).unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "Print"));
}

#[test]
fn reports_unterminated_nested_comment() {
    let err = lex("f[(* outer (* inner *)").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnterminatedComment {
            span: Span::new(2, 22),
        }
    );
}

#[test]
fn reports_unexpected_character_byte_span() {
    let err = lex("\\").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnexpectedCharacter {
            found: '\\',
            span: Span::new(0, 1),
        }
    );

    let err = lex("\u{2603}").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnexpectedCharacter {
            found: '\u{2603}',
            span: Span::new(0, 3),
        }
    );
}

#[test]
fn lexes_replace_repeated_before_replace_all_prefix() {
    let tokens = lex("expr //. rule").unwrap();

    assert!(matches!(tokens[1].kind, TokenKind::ReplaceRepeated));
}

#[test]
fn lexes_specific_slash_operators_before_postfix_apply() {
    let tokens = lex("a //. b a //= b a /. b a // b").unwrap();

    assert!(matches!(tokens[1].kind, TokenKind::ReplaceRepeated));
    assert!(matches!(tokens[4].kind, TokenKind::SlashSlashEqual));
    assert!(matches!(tokens[7].kind, TokenKind::ReplaceAll));
    assert!(matches!(tokens[10].kind, TokenKind::SlashSlash));
}

#[test]
fn lexes_postfix_operator_tokens_without_splitting_decimal_literals() {
    let tokens = lex("a .. b ... f' 1.5 .5 1. 1.. 1... 1.5e3").unwrap();

    assert!(matches!(tokens[1].kind, TokenKind::Repeated));
    assert!(matches!(tokens[3].kind, TokenKind::RepeatedNull));
    assert!(matches!(tokens[5].kind, TokenKind::Prime));
    assert!(matches!(&tokens[6].kind, TokenKind::Integer(value) if value == "1.5"));
    assert!(matches!(&tokens[7].kind, TokenKind::Integer(value) if value == ".5"));
    assert!(matches!(&tokens[8].kind, TokenKind::Integer(value) if value == "1."));
    assert!(matches!(&tokens[9].kind, TokenKind::Integer(value) if value == "1"));
    assert!(matches!(tokens[10].kind, TokenKind::Repeated));
    assert!(matches!(&tokens[11].kind, TokenKind::Integer(value) if value == "1"));
    assert!(matches!(tokens[12].kind, TokenKind::RepeatedNull));
    assert!(matches!(&tokens[13].kind, TokenKind::Integer(value) if value == "1.5e3"));
}

#[test]
fn lexes_bare_dot_without_regressing_decimal_or_repeated_tokens() {
    let tokens = lex("a . b 1.5 .5 1. 1.5e3 x_.. x_...").unwrap();

    assert!(matches!(tokens[1].kind, TokenKind::Dot));
    assert!(matches!(&tokens[3].kind, TokenKind::Integer(value) if value == "1.5"));
    assert!(matches!(&tokens[4].kind, TokenKind::Integer(value) if value == ".5"));
    assert!(matches!(&tokens[5].kind, TokenKind::Integer(value) if value == "1."));
    assert!(matches!(&tokens[6].kind, TokenKind::Integer(value) if value == "1.5e3"));
    assert!(matches!(tokens[9].kind, TokenKind::Repeated));
    assert!(matches!(tokens[12].kind, TokenKind::RepeatedNull));
}

#[test]
fn lexes_new_infix_operators_before_shared_prefixes() {
    let tokens = lex(
        "a ^:= b a ^= b a ^ b a ** b a *= b a * b f //@ x f /@ x a // f x |-> y a || b a | b a ~~ b a ~ f ~ b",
    )
    .unwrap();

    assert!(matches!(tokens[1].kind, TokenKind::CaretColonEqual));
    assert!(matches!(tokens[4].kind, TokenKind::CaretEqual));
    assert!(matches!(tokens[7].kind, TokenKind::Caret));
    assert!(matches!(tokens[10].kind, TokenKind::StarStar));
    assert!(matches!(tokens[13].kind, TokenKind::StarEqual));
    assert!(matches!(tokens[16].kind, TokenKind::Star));
    assert!(matches!(tokens[19].kind, TokenKind::SlashSlashAt));
    assert!(matches!(tokens[22].kind, TokenKind::SlashAt));
    assert!(matches!(tokens[25].kind, TokenKind::SlashSlash));
    assert!(matches!(tokens[28].kind, TokenKind::BarMinusGreater));
    assert!(matches!(tokens[31].kind, TokenKind::Or));
    assert!(matches!(tokens[34].kind, TokenKind::Bar));
    assert!(matches!(tokens[37].kind, TokenKind::TildeTilde));
    assert!(matches!(tokens[40].kind, TokenKind::Tilde));
    assert!(matches!(tokens[42].kind, TokenKind::Tilde));
}

#[test]
fn tokens_carry_byte_spans() {
    let tokens = lex("f[x]").unwrap();

    assert_eq!(tokens[0].span, Span::new(0, 1));
    assert_eq!(tokens[1].span, Span::new(1, 2));
    assert_eq!(tokens[2].span, Span::new(2, 3));
    assert_eq!(tokens[3].span, Span::new(3, 4));
    assert_eq!(tokens[4].span, Span::point(4));
}

#[test]
fn comments_do_not_shift_token_spans_for_following_tokens() {
    let tokens = lex("x(*comment*)+y").unwrap();

    assert_eq!(tokens[0].span, Span::new(0, 1));
    assert_eq!(tokens[1].span, Span::new(12, 13));
    assert_eq!(tokens[2].span, Span::new(13, 14));
}

#[test]
fn preserves_comments_and_whitespace_as_leading_trivia() {
    let lexed = lex_with_trivia("f[ (* a *) x, y]").unwrap();
    let trivia = lexed.leading_trivia(2).unwrap();

    assert_eq!(trivia.len(), 3);
    assert_eq!(trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(trivia[0].span, Span::new(2, 3));
    assert_eq!(trivia[0].text, " ");
    assert_eq!(trivia[1].kind, TriviaKind::Comment);
    assert_eq!(trivia[1].span, Span::new(3, 10));
    assert_eq!(trivia[1].text, "(* a *)");
    assert_eq!(trivia[2].kind, TriviaKind::Whitespace);
    assert_eq!(trivia[2].span, Span::new(10, 11));
    assert_eq!(trivia[2].text, " ");
}

#[test]
fn preserves_trailing_trivia_on_end_token() {
    let lexed = lex_with_trivia("x (* tail *)").unwrap();
    let trivia = lexed.leading_trivia(1).unwrap();

    assert_eq!(trivia.len(), 2);
    assert_eq!(trivia[0].kind, TriviaKind::Whitespace);
    assert_eq!(trivia[1].kind, TriviaKind::Comment);
    assert_eq!(trivia[1].text, "(* tail *)");
}

#[test]
fn lexes_string_literals_with_basic_and_unicode_escapes() {
    let tokens = lex("\"a\\n\\r\\t\\u{1F642}\"").unwrap();

    assert!(matches!(
        &tokens[0].kind,
        TokenKind::String(value) if value == "a\n\r\t🙂"
    ));
}

#[test]
fn lexes_escaped_multiline_string_literals() {
    let tokens = lex("\"\"\"\nab\\n\\t\\\"quote\"\"\"").unwrap();

    assert!(matches!(
        &tokens[0].kind,
        TokenKind::String(value) if value == "ab\n\t\"quote"
    ));

    let tokens = lex("\"\"\"\r\nab\"c\"\"d\"\"\"").unwrap();
    assert!(matches!(
        &tokens[0].kind,
        TokenKind::String(value) if value == "ab\"c\"\"d"
    ));

    let tokens = lex("\"\"\"\"\"\"").unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::String(value) if value.is_empty()));
}

#[test]
fn lexes_raw_multiline_string_literals_only_in_operand_position() {
    let tokens = lex("x = '''\nC:\\path \\t raw\n'''").unwrap();
    assert!(matches!(
        &tokens[2].kind,
        TokenKind::String(value) if value == "C:\\path \\t raw\n"
    ));

    let tokens = lex("x = '''\r\nraw'''").unwrap();
    assert!(matches!(&tokens[2].kind, TokenKind::String(value) if value == "raw"));

    let tokens = lex("''''''").unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::String(value) if value.is_empty()));

    let tokens = lex("f'''[x]").unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "f"));
    assert!(matches!(tokens[1].kind, TokenKind::Prime));
    assert!(matches!(tokens[2].kind, TokenKind::Prime));
    assert!(matches!(tokens[3].kind, TokenKind::Prime));
    assert!(matches!(tokens[4].kind, TokenKind::LBracket));
}

#[test]
fn reports_unterminated_multiline_string_literals_with_spans() {
    let err = lex("\"\"\"raw").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnterminatedString {
            span: Span::new(0, 6),
        }
    );
    assert!(err.is_incomplete());

    let err = lex("x = '''raw").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnterminatedString {
            span: Span::new(4, 10),
        }
    );
    assert!(err.is_incomplete());
}

#[test]
fn pins_multiline_string_delimiter_greedy_edges() {
    let err = lex("\"\"\"\"").unwrap_err();
    assert_eq!(
        err,
        SyntaxError::UnterminatedString {
            span: Span::new(0, 4),
        }
    );

    let tokens = lex("\"\"\"\"\"\"").unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::String(value) if value.is_empty()));
    assert!(matches!(tokens[1].kind, TokenKind::End));

    let tokens = lex(concat!("\"\"\"", r#"a\"""b"#, "\"\"\"")).unwrap();
    assert!(matches!(
        &tokens[0].kind,
        TokenKind::String(value) if value == "a\"\"\"b"
    ));
}
