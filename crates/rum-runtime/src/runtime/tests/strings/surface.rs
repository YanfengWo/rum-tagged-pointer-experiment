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
        "\"Plus[1/2, x]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[\"line\\n\"]"),
        "\"\\\"line\\\\n\\\"\""
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
        eval_to_string(&mut session, "TextString[{1, 2}]"),
        "\"List[1, 2]\""
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
