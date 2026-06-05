use super::*;

#[test]
fn message_name_and_message_emit_runtime_messages() {
    let mut session = Session::new();

    let named = session
        .eval_text("MessageName[FindFile, \"notfound\"]")
        .unwrap()
        .value;
    assert_eq!(named.to_string(), "FindFile::notfound");

    let outcome = session.eval_text("Message[FindFile::notfound, 7]").unwrap();
    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
    assert!(outcome.messages.iter().any(|message| message.text == "7"));
}

#[test]
fn message_family_accepts_symbol_name_strings_and_messagename_targets() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::usage = \"usage text\"").unwrap();

    let emitted = session
        .eval_text("Message[MessageName[\"f\", \"argx\"], 42]")
        .unwrap();
    assert_eq!(emitted.value.to_string(), "Null");
    assert!(emitted.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "bad arg 42"
    }));

    assert_eq!(
        session
            .eval_text("Messages[\"f\"]")
            .unwrap()
            .value
            .to_string(),
        "List[f::argx, f::usage]"
    );
    assert_eq!(
        session
            .eval_text("MessageDefinitions[MessageName[\"f\", \"argx\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"]]"
    );
    assert_eq!(
        session
            .eval_text("MessageTags[MessageName[\"f\", \"argx\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\"]"
    );
}

#[test]
fn direct_set_and_setdelayed_calls_share_message_assignment_normalization() {
    let mut session = Session::new();
    session
        .eval_text("Set[MessageName[\"f\", \"argx\"], \"bad arg `1`\"]")
        .unwrap();
    session
        .eval_text("SetDelayed[MessageName[\"f\", \"usage\"], suffix]")
        .unwrap();
    session.eval_text("suffix = 7").unwrap();

    assert_eq!(
        session.eval_text("Message[f::argx, 42]").unwrap().messages[0].text,
        "bad arg 42"
    );
    assert_eq!(
        session.eval_text("Message[f::usage]").unwrap().messages[0].text,
        "7"
    );
}

#[test]
fn message_templates_can_be_defined_and_formatted() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    let outcome = session.eval_text("Message[f::argx, 42]").unwrap();

    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "f" && message.tag == "argx")
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.text == "bad arg 42")
    );
}
#[test]
fn message_templates_support_message_name_and_all_args_placeholders() {
    let mut session = Session::new();

    session
        .eval_text("f::argx = \"`0`: first=`1`; all=`*`; implicit=``/``\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 1, 2]").unwrap();

    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "f"
            && message.tag == "argx"
            && message.text == "f::argx: first=1; all=1, 2; implicit=1/2"
    }));

    session
        .eval_text("f::str = \"string=`1`; all=`*`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::str, \"x\", \"y\"]").unwrap();
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "str" && message.text == "string=x; all=x, y"
    }));
}
#[test]
fn message_templates_support_richer_format_specifiers() {
    let mut session = Session::new();

    session
        .eval_text(
            "f::argx = \"sym=`0:symbol`; tag=`0:tag`; count=`*:count`; head=`1:head`; name=`2:name`; ctx=`2:context`; full=`2:full`; quoted=`3:quoted`; argc=`3:count`\"",
        )
        .unwrap();
    let outcome = session
        .eval_text("Message[f::argx, 7, Global`x, g[1, 2]]")
        .unwrap();

    assert!(outcome.messages.iter().any(|message| {
        message.text
            == "sym=f; tag=argx; count=3; head=Integer; name=x; ctx=Global`; full=Global`x; quoted=\"g[1, 2]\"; argc=2"
    }));
}

#[test]
fn message_templates_leave_unknown_placeholders_unchanged() {
    let mut session = Session::new();

    session
        .eval_text("f::argx = \"missing=`3`; literal=`name`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 1]").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.text == "missing=`3`; literal=`name`" })
    );
}

#[test]
fn message_templates_leave_unsupported_format_specs_unchanged() {
    let mut session = Session::new();

    session
        .eval_text("f::argx = \"ctx=`1:context`; bad=`*:head`; nope=`1:nope`\"")
        .unwrap();
    let outcome = session.eval_text("Message[f::argx, 7]").unwrap();

    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.text == "ctx=`1:context`; bad=`*:head`; nope=`1:nope`" })
    );
}

#[test]
fn messages_builtin_lists_defined_message_names() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage = \"usage text\"").unwrap();
    session.eval_text("g::tag = \"other\"").unwrap();

    assert_eq!(
        session.eval_text("Messages[f]").unwrap().value.to_string(),
        "List[f::argx, f::usage]"
    );
    assert_eq!(
        session.eval_text("Messages[]").unwrap().value.to_string(),
        "List[f::argx, f::usage, g::tag]"
    );
    assert_eq!(
        session
            .eval_text("Messages[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[f::argx]"
    );
}

#[test]
fn message_definitions_builtin_lists_immediate_and_delayed_templates() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg `1`\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();

    assert_eq!(
        session
            .eval_text("MessageDefinitions[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"], RuleDelayed[f::usage, suffix]]"
    );
    assert_eq!(
        session
            .eval_text("MessageDefinitions[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[f::argx, \"bad arg `1`\"]]"
    );
}

#[test]
fn message_tags_builtin_lists_global_symbol_and_exact_message_tags() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();
    session.eval_text("g::argx = \"other\"").unwrap();

    assert_eq!(
        session
            .eval_text("MessageTags[f]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\", \"usage\"]"
    );
    assert_eq!(
        session
            .eval_text("MessageTags[]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\", \"usage\"]"
    );
    assert_eq!(
        session
            .eval_text("MessageTags[f::argx]")
            .unwrap()
            .value
            .to_string(),
        "List[\"argx\"]"
    );
}

#[test]
fn inspect_reports_message_definition_presence() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg\"").unwrap();

    let info = session.inspect("f").unwrap();
    assert_eq!(info.message_count, 1);
    assert_eq!(info.message_tags, vec!["argx".to_string()]);
}

#[test]
fn inspect_message_catalog_exposes_templates_and_delayed_flags() {
    let mut session = Session::new();

    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();

    let entries = session.inspect_message_catalog(Some("f")).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].message_name, "f::argx");
    assert!(!entries[0].delayed);
    assert_eq!(entries[0].template.to_string(), "\"bad arg\"");
    assert_eq!(entries[1].message_name, "f::usage");
    assert!(entries[1].delayed);
    assert_eq!(entries[1].template.to_string(), "suffix");

    let exact = session.inspect_message_catalog(Some("f::usage")).unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].message_name, "f::usage");
}

#[test]
fn delayed_message_templates_are_evaluated_when_emitted() {
    let mut session = Session::new();

    session.eval_text("suffix = 7").unwrap();
    session.eval_text("f::argx := suffix").unwrap();
    let outcome = session.eval_text("Message[f::argx]").unwrap();

    assert!(outcome.messages.iter().any(|message| message.text == "7"));
}

#[test]
fn message_list_reports_last_top_level_messages() {
    let mut session = Session::new();

    session.eval_text("Message[FindFile::notfound]").unwrap();

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[HoldForm[FindFile::notfound]]"
    );

    let mut string_session = Session::new();
    string_session
        .eval_text("Message[FindFile::notfound]")
        .unwrap();
    assert_eq!(
        string_session
            .eval_text("ToString[$MessageList, InputForm]")
            .unwrap()
            .value
            .to_string(),
        "\"{HoldForm[FindFile::notfound]}\""
    );
}

#[test]
fn quiet_keeps_suppressed_messages_out_of_message_list() {
    let mut session = Session::new();

    session
        .eval_text("Quiet[Message[FindFile::notfound]]")
        .unwrap();

    assert_eq!(
        session.eval_text("$MessageList").unwrap().value.to_string(),
        "List[]"
    );
}

#[test]
fn invalid_message_name_form_emits_runtime_message() {
    let mut session = Session::new();

    let outcome = session.eval_text("MessageName[1, 2]").unwrap();

    assert_eq!(outcome.value.to_string(), "MessageName[1, 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "MessageName" && message.tag == "argform")
    );
}

#[test]
fn invalid_builtin_argument_shapes_emit_runtime_messages() {
    let mut session = Session::new();

    let with_outcome = session.eval_text("With[x, x]").unwrap();
    assert_eq!(with_outcome.value.to_string(), "With[x, x]");
    assert!(
        with_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "With" && message.tag == "argform" })
    );

    let replace_outcome = session.eval_text("7 /. 1").unwrap();
    assert_eq!(replace_outcome.value.to_string(), "ReplaceAll[7, 1]");
    assert!(
        replace_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "ReplaceAll" && message.tag == "argform" })
    );

    let get_outcome = session.eval_text("Get[1]").unwrap();
    assert_eq!(get_outcome.value.to_string(), "Get[1]");
    assert!(
        get_outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "Get" && message.tag == "argform" })
    );
}
