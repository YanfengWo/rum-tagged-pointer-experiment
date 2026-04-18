use super::*;
use std::fs;

#[test]
fn string_construction_and_measurement_use_code_point_semantics() {
    let mut session = Session::new();

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
}

#[test]
fn characters_character_range_and_tostring_cover_basic_text_flows() {
    let mut session = Session::new();

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
        eval_to_string(&mut session, "ToString[Span[All, 3]]"),
        "\";; 3\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Span[All, All, 2]]"),
        "\";; ;; 2\""
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
        crate::runtime::strings::render_input_like_expr(&part),
        "f[[1, 2 ;;]]"
    );

    let map = Expr::call(
        system_expr("Map"),
        vec![system_expr("g"), system_expr("xs")],
    );
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&map),
        "g /@ xs"
    );

    let apply = Expr::call(
        system_expr("Apply"),
        vec![system_expr("h"), system_expr("xs")],
    );
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&apply),
        "h @@ xs"
    );

    let map_apply = Expr::call(
        system_expr("MapApply"),
        vec![system_expr("k"), system_expr("xss")],
    );
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&map_apply),
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
        crate::runtime::strings::render_input_like_expr(&pattern_test),
        "x_?NumberQ"
    );

    let optional = Expr::call(
        system_expr("Optional"),
        vec![Expr::call(
            system_expr("Pattern"),
            vec![system_expr("y"), Expr::call(system_expr("Blank"), vec![])],
        )],
    );
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&optional),
        "y_."
    );

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
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&held),
        "1 + 2"
    );

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
        crate::runtime::strings::render_input_like_expr(&deferred),
        "f /@ {\"🙂\", \"中\"}"
    );

    let repeated = Expr::call(
        system_expr("Repeated"),
        vec![Expr::call(
            system_expr("Pattern"),
            vec![system_expr("x"), Expr::call(system_expr("Blank"), vec![])],
        )],
    );
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&repeated),
        "x_.."
    );

    let repeated_null = Expr::call(system_expr("RepeatedNull"), vec![Expr::string("🙂")]);
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&repeated_null),
        "\"🙂\"..."
    );
}

#[test]
fn string_character_classes_and_case_mapping_extend_the_string_subset() {
    let mut session = Session::new();

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
fn string_formatting_padding_and_riffle_cover_the_next_surface_wave() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"a``b``\", 1, x]"),
        "\"a1bx\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringRiffle[{\"a\", \"b\", \"c\"}]"),
        "\"a b c\""
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
}

#[test]
fn templates_text_words_codes_and_to_expression_extend_cl07() {
    let mut session = Session::new();

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
fn syntaxq_textstring_filetemplateapply_charactercounts_and_removediacritics_extend_cl07() {
    let mut session = Session::new();

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
        crate::runtime::strings::render_input_like_expr(&rendered_expr),
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

    let source_path =
        std::env::temp_dir().join(format!("rum-file-template-{}.txt", std::process::id()));
    fs::write(&source_path, "Hello, `name`!").unwrap();

    let expr = format!(
        "FileTemplateApply[\"{}\", <|\"name\" -> \"Alice\"|>]",
        source_path.display()
    );
    let output = session.eval_text(&expr).unwrap().value;
    let Expr::Atom(Atom::String(output_path)) = output else {
        panic!("FileTemplateApply should return a path string");
    };

    assert_eq!(fs::read_to_string(&output_path).unwrap(), "Hello, Alice!");

    fs::remove_file(&source_path).unwrap();
    fs::remove_file(&output_path).unwrap();
}
