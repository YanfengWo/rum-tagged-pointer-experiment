use crate::install_core;
use rum_core::Expr;
use rum_runtime::{RuntimeMessage, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_with_messages(session: &mut Session, source: &str) -> (Expr, Vec<RuntimeMessage>) {
    let outcome = session.eval_text(source).unwrap();
    (outcome.value, outcome.messages)
}

#[cfg(not(feature = "hash"))]
#[test]
fn no_default_features_drop_digest_builtins_but_keep_codecs() {
    let session = core_session();

    for name in ["Sha256", "Sha512", "Sha1", "Md5", "Crc32", "Hash"] {
        assert!(
            !session.inspect(name).is_some_and(|info| info.is_builtin),
            "{name} should not be registered without the hash feature"
        );
    }
    for name in ["ToHexString", "FromBase64", "ByteArrayQ"] {
        assert!(
            session.inspect(name).is_some_and(|info| info.is_builtin),
            "{name} should stay registered without the hash feature"
        );
    }
}

fn assert_failed_with_message(source: &str, symbol: &str, tag: &str) {
    let mut session = core_session();
    let (value, messages) = eval_with_messages(&mut session, source);
    assert_eq!(value.to_string(), "$Failed", "{source}");
    assert!(
        messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "missing {symbol}::{tag} for {source}; messages: {messages:?}"
    );
}

#[cfg(feature = "hash")]
fn assert_digest_vectors(algorithm: &str, vectors: &[(&str, &str)]) {
    let mut session = core_session();

    for (input, expected) in vectors {
        let quoted_expected = format!("\"{expected}\"");
        assert_eq!(
            eval_to_string(&mut session, &format!("{algorithm}[\"{input}\"]")),
            quoted_expected,
            "{algorithm} string input {input:?}"
        );
        assert_eq!(
            eval_to_string(
                &mut session,
                &format!("{algorithm}[StringToByteArray[\"{input}\"]]")
            ),
            quoted_expected,
            "{algorithm} ByteArray input {input:?}"
        );
        assert_eq!(
            eval_to_string(
                &mut session,
                &format!("ToHexString[{algorithm}[\"{input}\", \"Bytes\"]]")
            ),
            quoted_expected,
            "{algorithm} byte output {input:?}"
        );
        assert_eq!(
            eval_to_string(
                &mut session,
                &format!("ToHexString[{algorithm}[StringToByteArray[\"{input}\"], \"Bytes\"]]")
            ),
            quoted_expected,
            "{algorithm} ByteArray byte output {input:?}"
        );
    }
}

#[cfg(feature = "hash")]
#[test]
fn digest_known_answer_vectors_cover_string_and_byte_array_inputs() {
    let long = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

    assert_digest_vectors(
        "Sha256",
        &[
            (
                "",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                "abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                long,
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ],
    );
    assert_digest_vectors(
        "Sha512",
        &[
            (
                "",
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce\
47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            ),
            (
                "abc",
                "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
            ),
            (
                long,
                "204a8fc6dda82f0a0ced7beb8e08a41657c16ef468b228a8279be331a703c335\
96fd15c13b1b07f9aa1d3bea57789ca031ad85c7a71dd70354ec631238ca3445",
            ),
        ],
    );
    assert_digest_vectors(
        "Sha1",
        &[
            ("", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
            ("abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
            (long, "84983e441c3bd26ebaae4aa1f95129e5e54670f1"),
        ],
    );
    assert_digest_vectors(
        "Md5",
        &[
            ("", "d41d8cd98f00b204e9800998ecf8427e"),
            ("abc", "900150983cd24fb0d6963f7d28e17f72"),
            (long, "8215ef0796a20bcaaae116d3876c664a"),
        ],
    );
}

#[cfg(feature = "hash")]
#[test]
fn crc32_known_vectors_are_unsigned_integers() {
    let mut session = core_session();

    assert_eq!(eval_to_string(&mut session, "Crc32[\"\"]"), "0");
    assert_eq!(eval_to_string(&mut session, "Crc32[\"abc\"]"), "891568578");
    assert_eq!(
        eval_to_string(&mut session, "Crc32[\"123456789\"]"),
        "3421780262"
    );
    assert_eq!(
        eval_to_string(&mut session, "Crc32[StringToByteArray[\"123456789\"]]"),
        "3421780262"
    );
}

#[test]
fn hex_and_base64_codecs_round_trip_boundaries() {
    let mut session = core_session();
    let all_hex = (0_u8..=255)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    assert_eq!(eval_to_string(&mut session, "ToHexString[\"\"]"), "\"\"");
    assert_eq!(
        eval_to_string(&mut session, "FromHexString[\"\"]"),
        "ByteArray[\"\"]"
    );
    assert_eq!(eval_to_string(&mut session, "ToBase64[\"\"]"), "\"\"");
    assert_eq!(
        eval_to_string(&mut session, "FromBase64[\"\"]"),
        "ByteArray[\"\"]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ToHexString[\"héllo\"]"),
        "\"68c3a96c6c6f\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ByteArrayToString[FromBase64[ToBase64[\"héllo, 世界\"]]]"
        ),
        "\"héllo, 世界\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            &format!("ToHexString[FromHexString[\"{all_hex}\"]]")
        ),
        format!("\"{all_hex}\"")
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            &format!("ToHexString[FromBase64[ToBase64[FromHexString[\"{all_hex}\"]]]]")
        ),
        format!("\"{all_hex}\"")
    );
    assert_eq!(
        eval_to_string(&mut session, "ByteArrayToString[FromHexString[\"6162\"]]"),
        "\"ab\""
    );
    assert_eq!(
        eval_to_string(&mut session, "FromHexString[\"C3A9\"] // ByteArrayToString"),
        "\"é\""
    );
}

#[test]
fn malformed_codecs_fail_closed() {
    assert_failed_with_message("FromHexString[\"6\"]", "FromHexString", "hex");
    assert_failed_with_message("FromHexString[\"zz\"]", "FromHexString", "hex");
    assert_failed_with_message("FromBase64[\"%%%=\"]", "FromBase64", "base64");
    assert_failed_with_message("FromBase64[\"YQ=\"]", "FromBase64", "base64");
    assert_failed_with_message("FromBase64[\"Y===\"]", "FromBase64", "base64");
    assert_failed_with_message("FromBase64[\"YR==\"]", "FromBase64", "base64");
    assert_failed_with_message("FromBase64[\"YQ==AA==\"]", "FromBase64", "base64");
}

#[test]
fn byte_array_to_string_and_predicate_cover_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{ByteArrayQ[StringToByteArray[\"abc\"]], ByteArrayQ[ByteArray[\"%%%\"]], ByteArrayQ[\"abc\"], ByteArrayQ[ByteArray[1]]}"
        ),
        "List[True, False, False, False]"
    );
    assert_eq!(
        eval_to_string(&mut session, "ByteArrayToString[StringToByteArray[\"🙂\"]]"),
        "\"🙂\""
    );
    assert_failed_with_message(
        "ByteArrayToString[FromHexString[\"ff\"]]",
        "ByteArrayToString",
        "utf8",
    );
    assert_failed_with_message(
        "ByteArrayToString[ByteArray[\"%%%\"]]",
        "ByteArrayToString",
        "bytearray",
    );
}

#[test]
fn unsupported_argument_types_remain_inert() {
    let mut session = core_session();

    for source in [
        #[cfg(feature = "hash")]
        "Sha256[123]",
        #[cfg(feature = "hash")]
        "Sha256[\"abc\", \"HexString\"]",
        #[cfg(feature = "hash")]
        "Crc32[123]",
        "ToHexString[123]",
        "FromHexString[123]",
        "ToBase64[123]",
        "FromBase64[123]",
        "ByteArrayToString[\"abc\"]",
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), source, "{source}");
        assert!(
            outcome.messages.is_empty(),
            "{source}: {:?}",
            outcome.messages
        );
    }
}

#[cfg(feature = "hash")]
#[test]
fn digest_json_end_to_end_is_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Sha256[ToJson[<|\"a\" -> 1|>]]"),
        "\"015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862\""
    );
}

#[test]
fn byte_catalog_entries_are_pure() {
    #[cfg(feature = "hash")]
    for spec in crate::catalog::DIGEST_BUILTINS {
        assert_eq!(
            spec.capability,
            rum_runtime::Capability::Pure,
            "{}",
            spec.name
        );
    }
    for spec in crate::catalog::BYTE_BUILTINS {
        assert_eq!(
            spec.capability,
            rum_runtime::Capability::Pure,
            "{}",
            spec.name
        );
    }
}
