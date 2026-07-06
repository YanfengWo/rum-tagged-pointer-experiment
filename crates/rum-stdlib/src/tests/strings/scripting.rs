use super::*;

#[test]
fn text_shaping_utilities_cover_wrap_indent_dedent_tabs_and_tables() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringWrap[\"the quick brown fox\", 10]"),
        "\"the quick\\nbrown fox\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringWrap[\"abcdef xy\", 4]"),
        "\"abcdef\\nxy\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "StringWrap[StringWrap[\"the quick brown fox\", 10], 10]"
        ),
        "\"the quick\\nbrown fox\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringIndent[\"a\\n \\n\\nb\", \"> \"]"),
        "\"> a\\n \\n\\n> b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringIndent[\"a\\n\\nb\", 2]"),
        "\"  a\\n\\n  b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDedent[\"  a\\n    b\"]"),
        "\"a\\n  b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringDedent[StringDedent[\"  a\\n    b\"]]"),
        "\"a\\n  b\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringExpandTabs[\"abc\\tx\", 4]"),
        "\"abc x\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringExpandTabs[\"abcd\\tx\", 4]"),
        "\"abcd    x\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "AlignColumns[{{\"name\", \"qty\"}, {\"apple\", \"3\"}, {\"kiwi\", \"10\", \"extra\"}}]"
        ),
        "\"name   qty\\napple  3\\nkiwi   10   extra\""
    );

    let invalid_width = session.eval_text("StringWrap[\"abc\", 0]").unwrap();
    assert_eq!(invalid_width.value.to_string(), "StringWrap[\"abc\", 0]");
    assert!(
        invalid_width
            .messages
            .iter()
            .any(|message| message.symbol == "StringWrap" && message.tag == "argform")
    );
}

#[test]
fn html_shell_path_url_and_args_utilities_cover_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "HtmlEscape[\"a<b & c\\\"d\\u{0027}\"]"),
        "\"a&lt;b &amp; c&quot;d&apos;\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "HtmlUnescape[\"a&lt;b &amp; &#233; &#x4E16; &unknown;\"]"
        ),
        "\"a<b & é 世 &unknown;\""
    );
    assert_eq!(
        eval_to_string(&mut session, "ShellSplit[\"cc -o \\\"my prog\\\" main.c\"]"),
        "List[\"cc\", \"-o\", \"my prog\", \"main.c\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "ShellSplit[ShellQuote[\"a b \\u{0027}\\\"$\\n\"]]"
        ),
        r#"List["a b '\"$\n"]"#
    );

    let bad_shell = session.eval_text("ShellSplit[\"'unterminated\"]").unwrap();
    assert_eq!(bad_shell.value.to_string(), "$Failed");
    assert!(
        bad_shell
            .messages
            .iter()
            .any(|message| message.symbol == "ShellSplit" && message.tag == "syntax")
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "{FileExtension[\"a/b/c.tar.gz\"], FileBaseName[\"a/b/c.tar.gz\"], FileExtension[\".bashrc\"], FileNameSplit[\"/a/b/c\"]}"
        ),
        "List[\"gz\", \"c.tar\", \"\", List[\"/\", \"a\", \"b\", \"c\"]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "UrlDecode[UrlEncode[\"a b/世界\"]]"),
        "\"a b/世界\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "UrlParse[\"https://example.com:8080/x/y?q=1#top\"]"
        ),
        "Association[Rule[\"Scheme\", \"https\"], Rule[\"Host\", \"example.com\"], Rule[\"Port\", 8080], Rule[\"Path\", \"/x/y\"], Rule[\"Query\", \"q=1\"], Rule[\"Fragment\", \"top\"]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "UrlParse[\"https://example.com\"]"),
        "Association[Rule[\"Scheme\", \"https\"], Rule[\"Host\", \"example.com\"], Rule[\"Port\", Null], Rule[\"Path\", \"\"], Rule[\"Query\", Null], Rule[\"Fragment\", Null]]"
    );

    let bad_decode = session.eval_text("UrlDecode[\"%E4%ZZ\"]").unwrap();
    assert_eq!(bad_decode.value.to_string(), "$Failed");
    assert!(
        bad_decode
            .messages
            .iter()
            .any(|message| message.symbol == "UrlDecode" && message.tag == "escape")
    );

    let bad_parse = session.eval_text("UrlParse[\"example.com/path\"]").unwrap();
    assert_eq!(bad_parse.value.to_string(), "$Failed");
    assert!(
        bad_parse
            .messages
            .iter()
            .any(|message| message.symbol == "UrlParse" && message.tag == "syntax")
    );

    assert_eq!(
        eval_to_string(
            &mut session,
            "ParseArgs[{\"--verbose\", \"--out=x.txt\", \"--\", \"--literal\", \"pos\"}]"
        ),
        "Association[Rule[\"Positional\", List[\"--literal\", \"pos\"]], Rule[\"Options\", Association[Rule[\"out\", \"x.txt\"]]], Rule[\"Flags\", List[\"verbose\"]]]"
    );
}

#[test]
fn scripting_pipeline_combines_args_url_and_aligned_output() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "AlignColumns[{{\"field\", \"value\"}, {\"host\", Lookup[UrlParse[Lookup[Lookup[ParseArgs[{\"--url=https://example.com:443/x?q=1\"}], \"Options\"], \"url\"]], \"Host\"]}, {\"path\", Lookup[UrlParse[Lookup[Lookup[ParseArgs[{\"--url=https://example.com:443/x?q=1\"}], \"Options\"], \"url\"]], \"Path\"]}}]"
        ),
        "\"field  value\\nhost   example.com\\npath   /x\""
    );
}

#[test]
fn string_form_is_eager_and_reports_missing_slots() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"`1` + `2` = `3`\", 1, 2, 3]"),
        "\"1 + 2 = 3\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"a``b``\", 1, x]"),
        "\"a1bx\""
    );
    assert_eq!(
        eval_to_string(&mut session, "StringForm[\"\\\\`literal\", 1]"),
        "\"`literal\""
    );

    let missing = session.eval_text("StringForm[\"`` ``\", 1]").unwrap();
    assert_eq!(missing.value.to_string(), "StringForm[\"`` ``\", 1]");
    assert!(
        missing
            .messages
            .iter()
            .any(|message| message.symbol == "StringForm" && message.tag == "slot")
    );
}
