use super::*;

#[test]
fn string_construction_and_measurement_use_code_point_semantics() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("StringJoin[\"ru\", \"m\"]")
            .unwrap()
            .value
            .to_string(),
        "\"rum\""
    );
    assert_eq!(
        session
            .eval_text("StringJoin[{\"a\", \"🙂\", \"b\"}]")
            .unwrap()
            .value
            .to_string(),
        "\"a🙂b\""
    );
    assert_eq!(
        session
            .eval_text("StringLength[\"猫🙂\"]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("StringLength[{\"a\", \"bc\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
}

#[test]
fn characters_character_range_and_tostring_cover_basic_text_flows() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Characters[\"a🙂\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"a\", \"🙂\"]"
    );
    assert_eq!(
        session
            .eval_text("CharacterRange[\"x\", \"z\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"x\", \"y\", \"z\"]"
    );
    assert_eq!(
        session
            .eval_text("ToString[1/2 + x]")
            .unwrap()
            .value
            .to_string(),
        "\"Plus[1/2, x]\""
    );
    assert_eq!(
        session
            .eval_text("ToString[\"line\\n\"]")
            .unwrap()
            .value
            .to_string(),
        "\"\\\"line\\\\n\\\"\""
    );
}

#[test]
fn string_take_drop_and_part_support_basic_integer_and_range_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("StringTake[\"abcdef\", 3]")
            .unwrap()
            .value
            .to_string(),
        "\"abc\""
    );
    assert_eq!(
        session
            .eval_text("StringTake[\"abcdef\", -2]")
            .unwrap()
            .value
            .to_string(),
        "\"ef\""
    );
    assert_eq!(
        session
            .eval_text("StringTake[\"abcdef\", {2, 4}]")
            .unwrap()
            .value
            .to_string(),
        "\"bcd\""
    );
    assert_eq!(
        session
            .eval_text("StringDrop[\"abcdef\", 2]")
            .unwrap()
            .value
            .to_string(),
        "\"cdef\""
    );
    assert_eq!(
        session
            .eval_text("StringDrop[\"abcdef\", -2]")
            .unwrap()
            .value
            .to_string(),
        "\"abcd\""
    );
    assert_eq!(
        session
            .eval_text("StringDrop[\"abcdef\", {2, 4}]")
            .unwrap()
            .value
            .to_string(),
        "\"aef\""
    );
    assert_eq!(
        session
            .eval_text("StringPart[\"abcdef\", 2]")
            .unwrap()
            .value
            .to_string(),
        "\"b\""
    );
    assert_eq!(
        session
            .eval_text("StringPart[\"abcdef\", {1, 3, -1}]")
            .unwrap()
            .value
            .to_string(),
        "\"acf\""
    );
}

#[test]
fn string_builtins_preserve_current_boundary_for_non_string_or_invalid_specs() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("StringLength[x]")
            .unwrap()
            .value
            .to_string(),
        "StringLength[x]"
    );
    assert_eq!(
        session
            .eval_text("StringTake[\"abc\", x]")
            .unwrap()
            .value
            .to_string(),
        "StringTake[\"abc\", x]"
    );
    assert_eq!(
        session
            .eval_text("StringPart[\"abc\", 0]")
            .unwrap()
            .value
            .to_string(),
        "StringPart[\"abc\", 0]"
    );
    assert_eq!(
        session
            .eval_text("CharacterRange[\"ab\", \"z\"]")
            .unwrap()
            .value
            .to_string(),
        "CharacterRange[\"ab\", \"z\"]"
    );
}
