use super::*;

#[test]
fn byte_arrays_hashing_transport_and_nearest_extend_cl07() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "StringToByteArray[\"abc\"]"),
        "ByteArray[\"YWJj\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringToByteArray[\"abc\", \"UTF8\"]"),
        "ByteArray[\"YWJj\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "StringToByteArray[\"🙂\"]"),
        "ByteArray[\"8J+Zgg==\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\"]"),
        "84342368487090800366523834928142263660104883695016514377462985829716817089965"
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\", \"SHA256\", \"HexString\"]"),
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\", \"SHA256\", \"ByteArray\"]"),
        "ByteArray[\"ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Hash[\"abc\", \"SHA256\", \"DecimalString\"]"),
        "\"084342368487090800366523834928142263660104883695016514377462985829716817089965\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Hash[StringToByteArray[\"abc\"], \"SHA256\", \"HexString\"]"
        ),
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Compress[1 + 1]"),
        "\"rum:1:e:Mg==\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Compress[\"abc\"]"),
        "\"rum:1:s:YWJj\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Compress[StringToByteArray[\"abc\"]]"),
        "\"rum:1:b:YWJj\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Head[Encrypt[\"secret\", \"abc\"]]"),
        "EncryptedObject"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringContainsQ[ToString[Encrypt[\"secret\", StringToByteArray[\"abc\"]]], \"ByteArray\"]"
        ),
        "True"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Head[Encrypt[StringToByteArray[\"secret\"], StringToByteArray[\"abc\"]]]"
        ),
        "EncryptedObject"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3]"),
        "List[2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, 2]"),
        "List[2, 1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, All]"),
        "List[2, 1, 5]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}][3]"),
        "List[2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5} -> {a, b, c}, 3]"),
        "List[b]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5} -> {a, b, c}][3]"),
        "List[b]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\"]"
        ),
        "List[\"cat\", \"cot\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\", All]"
        ),
        "List[\"cat\", \"cot\", \"dog\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}][\"cut\"]"
        ),
        "List[\"cat\", \"cot\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"cut\"}, \"cut\", DistanceFunction -> HammingDistance]"
        ),
        "List[\"cut\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"cut\"}, \"cut\", 2, DistanceFunction -> HammingDistance]"
        ),
        "List[\"cut\", \"cat\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, {All, 1}]"),
        "List[2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, {All, Infinity}]"),
        "List[2, 1, 5]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\", {All, 1}, DistanceFunction -> EditDistance]"
        ),
        "List[\"cat\", \"cot\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\", {All, Infinity}, DistanceFunction -> EditDistance]"
        ),
        "List[\"cat\", \"cot\", \"dog\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cat\"]"),
        "\"cats\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cat\", 1]"),
        "\"cat\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"city\", 2]"),
        "\"cities\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"box\", 2]"),
        "\"boxes\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"child\", 2]"),
        "\"children\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"person\", 2]"),
        "\"people\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"sheep\", 2]"),
        "\"sheep\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"knife\", 2]"),
        "\"knives\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"hero\", 2]"),
        "\"heroes\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"analysis\", 2]"),
        "\"analyses\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"QUIZ\", 2]"),
        "\"QUIZZES\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Knife\", 2]"),
        "\"Knives\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cactus\", 2]"),
        "\"cacti\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"phenomenon\", 2]"),
        "\"phenomena\""
    );
}
