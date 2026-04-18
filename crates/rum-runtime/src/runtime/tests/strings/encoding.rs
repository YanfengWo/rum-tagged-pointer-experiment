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
        eval_to_string(&mut session, "Hash[1/2 + x, \"SHA256\", \"HexString\"]"),
        "\"e697c4f396641d09562ff62b6d49d4de1d6e852d0101b44f8c2959542984dbb5\""
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
        eval_to_string(&mut session, "Uncompress[Compress[\"abc\"]]"),
        "\"abc\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Uncompress[Compress[StringToByteArray[\"🙂\"]]]"
        ),
        "ByteArray[\"8J+Zgg==\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Uncompress[Compress[\"🙂中\"]]"),
        "\"🙂中\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Uncompress[Compress[f[x_]]]]"),
        "\"f[x_]\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToString[Uncompress[Compress[<|a -> 1, b -> 2|>]]]"
        ),
        "\"<|a -> 1, b -> 2|>\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ToString[Uncompress[Compress[1 + 1], Hold]]"),
        "\"Hold[2]\""
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
        eval_to_string(
            &mut session,
            "Decrypt[\"abc\", Encrypt[\"abc\", \"secret\"]]"
        ),
        "\"secret\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Decrypt[StringToByteArray[\"abc\"], Encrypt[StringToByteArray[\"abc\"], StringToByteArray[\"secret\"]]]"
        ),
        "ByteArray[\"c2VjcmV0\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToString[Decrypt[\"abc\", Encrypt[\"abc\", 1/2 + x]]]"
        ),
        "\"1/2 + x\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToString[Decrypt[\"abc\", Encrypt[\"abc\", f[x_]]]]"
        ),
        "\"f[x_]\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ToString[Decrypt[\"abc\", Encrypt[\"abc\", <|a -> 1, b -> 2|>]]]"
        ),
        "\"<|a -> 1, b -> 2|>\""
    );
    let structured_expr = Expr::call(
        system_expr("f"),
        vec![Expr::call(
            system_expr("Condition"),
            vec![
                Expr::call(
                    system_expr("Pattern"),
                    vec![system_expr("x"), Expr::call(system_expr("Blank"), vec![])],
                ),
                system_expr("True"),
            ],
        )],
    );
    let decrypted = session
        .evaluate(Expr::call(
            system_expr("Decrypt"),
            vec![
                Expr::string("abc"),
                Expr::call(
                    system_expr("Encrypt"),
                    vec![Expr::string("abc"), structured_expr.clone()],
                ),
            ],
        ))
        .unwrap();
    assert_eq!(
        crate::runtime::strings::render_input_like_expr(&decrypted),
        "f[x_ /; True]"
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
        eval_to_string(&mut session, "Nearest[<|1 -> a, 2 -> b, 5 -> c|>, 3]"),
        "List[b]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[<|1 -> a, 2 -> b, 5 -> c|>][3]"),
        "List[b]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[1 -> a, 3]"),
        "List[a]"
    );
    assert_eq!(eval_to_string(&mut session, "Nearest[5, 3]"), "List[5]");
    assert_eq!(eval_to_string(&mut session, "Nearest[5][3]"), "List[5]");
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[{\"cat\", \"cot\", \"dog\"}, \"cut\"]"
        ),
        "List[\"cat\", \"cot\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[\"cat\", \"cut\"]"),
        "List[\"cat\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[\"cat\"][\"cut\"]"),
        "List[\"cat\"]"
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
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, Automatic]"),
        "List[2]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Nearest[{1, 2, 5}, 3, {Automatic, Infinity}]"),
        "List[2]"
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
        eval_to_string(
            &mut session,
            "Nearest[<|\"cat\" -> 1, \"cot\" -> 2, \"dog\" -> 3|>, \"cut\", {All, 1}, DistanceFunction -> EditDistance]"
        ),
        "List[1, 2]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Nearest[\"cat\" -> 1, \"cut\", DistanceFunction -> EditDistance]"
        ),
        "List[1]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"cat\"]"),
        "\"cats\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"red box\"]"),
        "\"red boxes\""
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
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Focus\", 2]"),
        "\"Foci\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"ALUMNUS\", 2]"),
        "\"ALUMNI\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"nucleus\", 2]"),
        "\"nuclei\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Appendix\", 2]"),
        "\"Appendices\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"thesis\", 2]"),
        "\"theses\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Matrix\", 2]"),
        "\"Matrices\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"criterion\", 2]"),
        "\"criteria\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Vertex\", 2]"),
        "\"Vertices\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"happy child\", 2]"),
        "\"happy children\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"red box\", 2]"),
        "\"red boxes\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"city bus\", 2]"),
        "\"city buses\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"radius\", 2]"),
        "\"radii\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"curriculum\", 2]"),
        "\"curricula\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"Ox\", 2]"),
        "\"Oxen\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"happy child!\", 2]"),
        "\"happy children!\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Pluralize[\"city bus?\", 2]"),
        "\"city buses?\""
    );
}
