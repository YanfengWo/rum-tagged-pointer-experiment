use super::*;

#[test]
fn string_construction_and_measurement_use_code_point_semantics() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringJoin[\"ru\", \"m\"]"),
        "\"rum\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringJoin[{\"a\", \"\\u{1F642}\", \"b\"}]"),
        "\"a🙂b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLength[\"\\u{732B}\\u{1F642}\"]"),
        "2"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringLength[{\"a\", \"bc\"}]"),
        "List[1, 2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Row[{\"x = \", 1 + 2, \"!\"}]"),
        "\"x = 3!\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Row[{a, b, c}, \", \"]"),
        "\"a, b, c\""
    );
}

#[test]
fn characters_character_range_and_tostring_cover_basic_text_flows() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Characters[\"a\\u{1F642}\"]"),
        "List[\"a\", \"🙂\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterRange[\"x\", \"z\"]"),
        "List[\"x\", \"y\", \"z\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x, InputForm]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x, OutputForm]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x, StandardForm]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x, TraditionalForm]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[1/2 + x, FullForm]"),
        "\"Plus[1/2, x]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[\"line\\n\"]"),
        "\"\\\"line\\\\n\\\"\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[(a + b)^2, InputForm]"),
        "\"(a + b) ^ 2\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[HoldForm[1 + 1]]"),
        "\"1 + 1\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[HoldForm[1 + 1], OutputForm]"),
        "\"1 + 1\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[HoldForm[1 + 1], StandardForm]"),
        "\"1 + 1\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[HoldForm[1 + 1], InputForm]"),
        "\"HoldForm[1 + 1]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[{a, b -> c}]"),
        "\"{a, b -> c}\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[<|a -> 1, b -> 2|>]"),
        "\"<|a -> 1, b -> 2|>\""
    );
    assert_eq!(eval_to_string(&mut session, "ToString[x_]"), "\"x_\"");
    assert_eq!(
        eval_to_string(&mut session, "ToString[x_Integer]"),
        "\"x_Integer\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[# + #2 &]"),
        "\"# + #2 &\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[y_ -> y]"),
        "\"y_ -> y\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[y_ :> y]"),
        "\"y_ :> y\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[f[x_ /; True]]"),
        "\"f[x_ /; True]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Alternatives[y_, z__]]"),
        "\"y_ | z__\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToString[StringExpression[\"a\", Pattern[x, BlankSequence[]], EndOfString]]"
        ),
        "\"\\\"a\\\" ~~ x__ ~~ EndOfString\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Span[All, 3]]"),
        "\";; 3\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Span[All, All, 2]]"),
        "\";; ;; 2\""
    );
}

#[test]
fn string_generators_respect_resource_limits() {
    let mut character_range_session = core_session();
    character_range_session.set_generated_element_limit(2);
    let character_range = character_range_session
        .eval_text("CharacterRange[\"x\", \"z\"]")
        .unwrap();
    assert_eq!(
        character_range.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(
        character_range
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut repeat_session = core_session();
    repeat_session.set_generated_element_limit(3);
    let repeat = repeat_session.eval_text("StringRepeat[\"ab\", 2]").unwrap();
    assert_eq!(
        repeat.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(
        repeat
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let mut memory_session = core_session();
    assert_eq!(
        memory_session
            .eval_text("MemoryConstrained[StringRepeat[\"x\", 100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn input_like_renderer_uses_part_and_map_family_syntax() {
    let part = Expr::call(
        system_expr("Part"),
        vec![
            system_expr("f"),
            Expr::integer(BigInt::from(1_u8)),
            Expr::call(
                system_expr("Span"),
                vec![Expr::integer(BigInt::from(2_u8)), system_expr("All")],
            ),
        ],
    );
    assert_eq!(
        crate::strings::render_input_like_expr(&part),
        "f[[1, 2 ;;]]"
    );

    let map = Expr::call(
        system_expr("Map"),
        vec![system_expr("g"), system_expr("xs")],
    );
    assert_eq!(crate::strings::render_input_like_expr(&map), "g /@ xs");

    let apply = Expr::call(
        system_expr("Apply"),
        vec![system_expr("h"), system_expr("xs")],
    );
    assert_eq!(crate::strings::render_input_like_expr(&apply), "h @@ xs");

    let map_apply = Expr::call(
        system_expr("MapApply"),
        vec![system_expr("k"), system_expr("xss")],
    );
    assert_eq!(
        crate::strings::render_input_like_expr(&map_apply),
        "k @@@ xss"
    );

    let pattern_test = Expr::call(
        system_expr("PatternTest"),
        vec![
            Expr::call(
                system_expr("Pattern"),
                vec![system_expr("x"), Expr::call(system_expr("Blank"), vec![])],
            ),
            system_expr("NumberQ"),
        ],
    );
    assert_eq!(
        crate::strings::render_input_like_expr(&pattern_test),
        "x_?NumberQ"
    );

    let optional = Expr::call(
        system_expr("Optional"),
        vec![Expr::call(
            system_expr("Pattern"),
            vec![system_expr("y"), Expr::call(system_expr("Blank"), vec![])],
        )],
    );
    assert_eq!(crate::strings::render_input_like_expr(&optional), "y_.");

    let held = Expr::call(
        system_expr("HoldForm"),
        vec![Expr::call(
            system_expr("Plus"),
            vec![
                Expr::integer(BigInt::from(1_u8)),
                Expr::integer(BigInt::from(2_u8)),
            ],
        )],
    );
    assert_eq!(crate::strings::render_input_like_expr(&held), "1 + 2");

    let deferred = Expr::call(
        system_expr("Defer"),
        vec![Expr::call(
            system_expr("Map"),
            vec![
                system_expr("f"),
                Expr::call(
                    system_expr("List"),
                    vec![Expr::string("🙂"), Expr::string("中")],
                ),
            ],
        )],
    );
    assert_eq!(
        crate::strings::render_input_like_expr(&deferred),
        "f /@ {\"🙂\", \"中\"}"
    );

    let repeated = Expr::call(
        system_expr("Repeated"),
        vec![Expr::call(
            system_expr("Pattern"),
            vec![system_expr("x"), Expr::call(system_expr("Blank"), vec![])],
        )],
    );
    assert_eq!(crate::strings::render_input_like_expr(&repeated), "x_..");

    let repeated_null = Expr::call(system_expr("RepeatedNull"), vec![Expr::string("🙂")]);
    assert_eq!(
        crate::strings::render_input_like_expr(&repeated_null),
        "\"🙂\"..."
    );
}

#[test]
fn string_character_classes_and_case_mapping_extend_the_string_subset() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "ToUpperCase[\"ab🙂c\"]"),
        "\"AB🙂C\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToUpperCase[{\"alpha\", \"Beta\"}]"),
        "List[\"ALPHA\", \"BETA\"]"
    );

    assert_eq!(eval_to_string(&mut session, "LetterQ[\"\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "LetterQ[\"abc\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "LetterQ[\"ab1\"]"), "False");
    assert_eq!(
        eval_to_string(&mut session, "LetterQ[{\"abc\", \"123\"}]"),
        "List[True, False]"
    );

    assert_eq!(eval_to_string(&mut session, "DigitQ[\"\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "DigitQ[\"123\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "DigitQ[\"12a\"]"), "False");
    assert_eq!(
        eval_to_string(&mut session, "DigitQ[{\"123\", \"1a\"}]"),
        "List[True, False]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\" \\t\", Whitespace]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"Snap`Private`\", \"Snap`*\"]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"Other`\", \"Snap`*\"]"),
        "False"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringFreeQ[\"abc\", Whitespace]"),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringCases[\"x=-1.5 y=.25 z=1e3\", NumberString]"
        ),
        "List[\"-1.5\", \".25\", \"1\", \"3\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringMatchQ[{\"123\", \".5\", \"1e3\"}, NumberString]"
        ),
        "List[True, True, False]"
    );
}

#[test]
fn string_pattern_symbols_are_protected() {
    let mut session = core_session();

    for name in [
        "Alternatives",
        "Blank",
        "BlankNullSequence",
        "BlankSequence",
        "DigitCharacter",
        "EndOfLine",
        "EndOfString",
        "Except",
        "LetterCharacter",
        "Pattern",
        "PunctuationCharacter",
        "RegularExpression",
        "StartOfLine",
        "StartOfString",
        "StringExpression",
        "WhitespaceCharacter",
        "WordBoundary",
        "WordCharacter",
    ] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            eval_to_string(&mut session, &source),
            "List[Protected]",
            "{source}"
        );
    }
}

#[test]
fn protected_string_pattern_symbols_reject_definitions_without_changing_matching() {
    let mut session = core_session();

    let digit_assignment = session.eval_text("DigitCharacter = 5").unwrap();
    assert_eq!(digit_assignment.value.to_string(), "5");
    assert!(digit_assignment.messages.iter().any(|message| {
        message.symbol == "Set" && message.tag == "wrsym" && message.text.contains("Protected")
    }));
    assert_eq!(
        session
            .eval_text("OwnValues[DigitCharacter]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringMatchQ[\"5\", DigitCharacter]"),
        "True"
    );

    let blank_definition = session.eval_text("Blank[x_] := x").unwrap();
    assert_eq!(blank_definition.value.to_string(), "Null");
    assert!(blank_definition.messages.iter().any(|message| {
        message.symbol == "SetDelayed"
            && message.tag == "wrsym"
            && message.text.contains("Protected")
    }));
    assert_eq!(
        session
            .eval_text("DownValues[Blank]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(eval_to_string(&mut session, "MatchQ[anything, _]"), "True");
}

#[test]
fn string_helper_builtins_cover_surface_edges() {
    let mut session = core_session();

    for name in ["StringReverse", "ToLowerCase", "ToUpperCase"] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            eval_to_string(&mut session, &source),
            "List[Listable, Protected]",
            "{source}"
        );
    }
    for name in ["LowerCaseQ", "UpperCaseQ"] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            eval_to_string(&mut session, &source),
            "List[Protected]",
            "{source}"
        );
    }
    for name in ["Capitalize", "Decapitalize"] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            eval_to_string(&mut session, &source),
            "List[Listable, Protected, ReadProtected]",
            "{source}"
        );
    }

    assert_eq!(
        eval_to_string(&mut session, "ToLowerCase[\"HELLO\"]"),
        "\"hello\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToLowerCase[\"AB\\u{1F642}C\"]"),
        "\"ab🙂c\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToLowerCase[{\"A\", \"B\"}]"),
        "List[\"a\", \"b\"]"
    );

    assert_eq!(
        eval_to_string(&mut session, "StringReverse[\"hello\"]"),
        "\"olleh\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringReverse[\"a\\u{1F642}b\"]"),
        "\"b🙂a\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringReverse[{\"ab\", \"cd\"}]"),
        "List[\"ba\", \"dc\"]"
    );

    assert_eq!(
        eval_to_string(&mut session, "Capitalize[\"hello world\"]"),
        "\"Hello world\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Capitalize[{\"alpha\", \"beta\"}]"),
        "List[\"Alpha\", \"Beta\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Decapitalize[\"Hello\"]"),
        "\"hello\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Decapitalize[{\"Alpha\", \"Beta\"}]"),
        "List[\"alpha\", \"beta\"]"
    );

    assert_eq!(eval_to_string(&mut session, "LowerCaseQ[\"abc\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "LowerCaseQ[\"aBc\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "LowerCaseQ[\"a1\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "LowerCaseQ[\"123\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "LowerCaseQ[\"\"]"), "True");
    assert_eq!(
        eval_to_string(&mut session, "LowerCaseQ[\"\\u{00DF}\"]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "LowerCaseQ[\"\\u{1F642}\"]"),
        "False"
    );

    assert_eq!(eval_to_string(&mut session, "UpperCaseQ[\"ABC\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "UpperCaseQ[\"ABc\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "UpperCaseQ[\"A1\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "UpperCaseQ[\"123\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "UpperCaseQ[\"\"]"), "True");
    assert_eq!(
        eval_to_string(&mut session, "UpperCaseQ[\"\\u{00DF}\"]"),
        "False"
    );
    assert_eq!(
        eval_to_string(&mut session, "UpperCaseQ[\"\\u{1F642}\"]"),
        "False"
    );

    for source in [
        "StringReverse[x]",
        "ToLowerCase[x]",
        "LowerCaseQ[x]",
        "UpperCaseQ[x]",
        "Capitalize[x]",
        "Decapitalize[x]",
    ] {
        assert_eq!(eval_to_string(&mut session, source), source, "{source}");
    }
}

#[test]
fn case_conversion_threads_nested_and_mixed_lists_like_listable() {
    let mut session = core_session();

    for (source, expected) in [
        ("ToUpperCase[{\"a\", \"b\"}]", "List[\"A\", \"B\"]"),
        ("Capitalize[{\"ab\", \"cd\"}]", "List[\"Ab\", \"Cd\"]"),
        ("Decapitalize[{\"Ab\", \"Cd\"}]", "List[\"ab\", \"cd\"]"),
        (
            "ToUpperCase[{{\"a\"}, {\"b\"}}]",
            "List[List[\"A\"], List[\"B\"]]",
        ),
        ("ToUpperCase[{\"a\", 1}]", "List[\"A\", ToUpperCase[1]]"),
        (
            "Capitalize[{{\"ab\"}, {\"cd\"}}]",
            "List[List[\"Ab\"], List[\"Cd\"]]",
        ),
        ("Capitalize[{\"ab\", 1}]", "List[\"Ab\", Capitalize[1]]"),
        (
            "Decapitalize[{{\"Ab\"}, {\"Cd\"}}]",
            "List[List[\"ab\"], List[\"cd\"]]",
        ),
        ("Decapitalize[{\"Ab\", 1}]", "List[\"ab\", Decapitalize[1]]"),
    ] {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn string_formatting_padding_and_riffle_cover_the_next_surface_wave() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"a``b``\", 1, x]"),
        "\"a1bx\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[StringForm[\"a``b``\", 1, x]]"),
        "\"\\\"a1bx\\\"\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextString[StringForm[\"a``b``\", 1, x]]"),
        "\"a1bx\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringRiffle[{\"a\", \"b\", \"c\"}]"),
        "\"a b c\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringRiffle[{\"a\", \"b\", \"c\"}, {\"(\", \", \", \")\"}]"
        ),
        "\"(a, b, c)\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringRiffle[{{\"a\", \"b\"}, {\"c\", \"d\"}}, \",\", \";\"]"
        ),
        "\"a;b,c;d\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[\"7\", 3]"),
        "\"  7\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[{\"7\", \"88\"}, 3, \"0\"]"),
        "List[\"007\", \"088\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[\"x\", 4, {\"0\", \"1\"}]"),
        "\"010x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadLeft[\"x\", 5, {\"ab\", \"c\"}]"),
        "\"abcax\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringPadRight[\"x\", 4, {\"ab\"}]"),
        "\"xaba\""
    );
}

#[test]
fn templates_text_words_codes_and_to_expression_extend_cl07() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringTemplate[\"Hello, `name`!\"]"),
        "TemplateObject[\"Hello, `name`!\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TemplateApply[StringTemplate[\"Hello, `name`!\"], <|\"name\" -> \"Alice\"|>]"
        ),
        "\"Hello, Alice!\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TemplateApply[\"x = ``\", 7]"),
        "\"x = 7\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextWords[\"alpha, beta 123 gamma\"]"),
        "List[\"alpha\", \"beta\", \"123\", \"gamma\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToCharacterCode[\"Ab\"]"),
        "List[65, 98]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToCharacterCode[{\"Ab\", \"中\"}]"),
        "List[List[65, 98], List[20013]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{ToCharacterCode[\"é\"], ToCharacterCode[\"é\", \"UTF8\"], ToCharacterCode[\"é\", \"UTF-8\"]}"
        ),
        "List[List[233], List[195, 169], List[195, 169]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToCharacterCode[\"é\", \"Latin1\"]"),
        "ToCharacterCode[\"é\", \"Latin1\"]"
    );
    assert_eq!(eval_to_string(&mut session, "ToExpression[\"1+2\"]"), "3");
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToExpression[\"1+2\", InputForm, HoldAllComplete]"
        ),
        "HoldAllComplete[3]"
    );
}

#[test]
fn syntaxq_textstring_charactercounts_and_removediacritics_extend_cl07() {
    let mut session = core_session();

    assert_eq!(eval_to_string(&mut session, "SyntaxQ[\"1+2\"]"), "True");
    assert_eq!(eval_to_string(&mut session, "SyntaxQ[\"1+\"]"), "False");
    assert_eq!(eval_to_string(&mut session, "TextString[123]"), "\"123\"");
    assert_eq!(
        eval_to_string(&mut session, "TextString[1/2 + x]"),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextString[{1, 2}]"),
        "\"{1, 2}\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextString[<|a -> 1, b -> 2|>]"),
        "\"<|a -> 1, b -> 2|>\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "TextString[StringExpression[\"a\", Pattern[x, BlankSequence[]], EndOfString]]"
        ),
        "\"\\\"a\\\" ~~ x__ ~~ EndOfString\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextString[y_ -> y]"),
        "\"y_ -> y\""
    );
    assert_eq!(
        eval_to_string(&mut session, "TextString[\"🙂 café 中\"]"),
        "\"🙂 café 中\""
    );
    let rendered_expr = Expr::call(
        system_expr("f"),
        vec![
            Expr::call(
                system_expr("PatternTest"),
                vec![
                    Expr::call(
                        system_expr("Pattern"),
                        vec![system_expr("x"), Expr::call(system_expr("Blank"), vec![])],
                    ),
                    system_expr("NumberQ"),
                ],
            ),
            Expr::call(
                system_expr("Optional"),
                vec![Expr::call(
                    system_expr("Pattern"),
                    vec![system_expr("y"), Expr::call(system_expr("Blank"), vec![])],
                )],
            ),
        ],
    );
    assert_eq!(
        crate::strings::render_input_like_expr(&rendered_expr),
        "f[x_?NumberQ, y_.]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterCounts[\"abca\"]"),
        "Association[Rule[\"a\", 2], Rule[\"c\", 1], Rule[\"b\", 1]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "CharacterCounts[\"中中a\"]"),
        "Association[Rule[\"中\", 2], Rule[\"a\", 1]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "RemoveDiacritics[\"café naïve façade\"]"),
        "\"cafe naive facade\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "RemoveDiacritics[{\"café\", \"São\", \"Žižek\"}]"
        ),
        "List[\"cafe\", \"Sao\", \"Zizek\"]"
    );
}
