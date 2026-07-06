use crate::install_core;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::system_expr;
use rum_runtime::{RuntimeMessage, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_string_atom(session: &mut Session, source: &str) -> String {
    match session.eval_text(source).unwrap().value {
        Expr::Atom(Atom::String(value)) => value,
        other => panic!("expected string atom, got {other}"),
    }
}

fn from_yaml_call(source: &str) -> Expr {
    Expr::call(system_expr("FromYaml"), vec![Expr::string(source)])
}

fn from_yaml_to_string(session: &mut Session, source: &str) -> String {
    let value = session.evaluate(from_yaml_call(source)).unwrap();
    let messages = session.take_messages();
    assert!(messages.is_empty(), "{source}: {messages:?}");
    value.to_string()
}

fn assert_failed_with_message(
    value: Expr,
    messages: Vec<RuntimeMessage>,
    symbol: &str,
    tag: &str,
    needle: &str,
) {
    assert_eq!(value.to_string(), "$Failed");
    assert!(
        messages.iter().any(|message| {
            message.symbol == symbol && message.tag == tag && message.text.contains(needle)
        }),
        "missing {symbol}::{tag} containing {needle:?}; messages: {messages:?}"
    );
}

const K8SISH: &str = r#"
# service config
svc:
  name: web # plain scalar comment
  replicas: 2
  enabled: true
  pi: 3.14
  hosts:
    - a
    - b
  containers:
    - name: web
      image: "nginx:1.25"
      env:
        - name: COUNTRY
          value: 'no'
  note: |-
    line1
    line2
"#;

#[test]
fn from_yaml_parses_k8s_flavored_config() {
    let mut session = core_session();

    assert_eq!(
        from_yaml_to_string(&mut session, K8SISH),
        "Association[Rule[\"svc\", Association[Rule[\"name\", \"web\"], Rule[\"replicas\", 2], Rule[\"enabled\", True], Rule[\"pi\", 157/50], Rule[\"hosts\", List[\"a\", \"b\"]], Rule[\"containers\", List[Association[Rule[\"name\", \"web\"], Rule[\"image\", \"nginx:1.25\"], Rule[\"env\", List[Association[Rule[\"name\", \"COUNTRY\"], Rule[\"value\", \"no\"]]]]]]], Rule[\"note\", \"line1\\nline2\"]]]]"
    );
}

#[test]
fn from_yaml_uses_norway_safe_typing() {
    let mut session = core_session();
    let source = "\
country: no
no: 1
yes: yes
on: on
off: off
No: No
";

    assert_eq!(
        from_yaml_to_string(&mut session, source),
        "Association[Rule[\"country\", \"no\"], Rule[\"no\", 1], Rule[\"yes\", \"yes\"], Rule[\"on\", \"on\"], Rule[\"off\", \"off\"], Rule[\"No\", \"No\"]]"
    );
}

#[test]
fn from_yaml_numbers_and_nulls_are_exact() {
    let mut session = core_session();
    let big = "1234567890".repeat(12);
    let source = format!(
        "\
decimal: 0.1
pi: 3.14
exp: 1.5e3
neg: -2
big: {big}
tilde: ~
empty:
"
    );

    assert_eq!(
        from_yaml_to_string(&mut session, &source),
        format!(
            "Association[Rule[\"decimal\", 1/10], Rule[\"pi\", 157/50], Rule[\"exp\", 1500], Rule[\"neg\", -2], Rule[\"big\", {big}], Rule[\"tilde\", Null], Rule[\"empty\", Null]]"
        )
    );
}

#[test]
fn from_yaml_literal_blocks_pin_chomping() {
    let mut session = core_session();
    let source = "\
clip: |
  line1
  line2
strip: |-
  line1
  line2
";

    assert_eq!(
        from_yaml_to_string(&mut session, source),
        "Association[Rule[\"clip\", \"line1\\nline2\\n\"], Rule[\"strip\", \"line1\\nline2\"]]"
    );
}

#[test]
fn from_yaml_rejects_unsupported_yaml_fail_closed() {
    let mut session = core_session();

    for (source, needle) in [
        ("a: &x 1", "anchors"),
        ("a: *x", "aliases"),
        ("a: !str x", "tags"),
        ("%YAML 1.2\n", "directives"),
        ("---\na: 1\n", "document markers"),
        ("<<: 1", "merge keys"),
        ("a: [1, 2]", "non-empty flow"),
        ("a: >\n  x\n", "folded block"),
        ("a:\n\tb: 1\n", "tab indentation"),
        ("a: 1\na: 2\n", "duplicate mapping key"),
        ("- a\nb: c\n", "same indentation"),
        ("a:\n    b: 1\n  c: 2\n", "inconsistent indentation"),
    ] {
        let value = session.evaluate(from_yaml_call(source)).unwrap();
        let messages = session.take_messages();
        assert_failed_with_message(value, messages, "FromYaml", "yaml", needle);
    }
}

#[test]
fn to_yaml_round_trips_adversarial_strings_and_empties() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SameQ[FromYaml[ToYaml[<|\"a\" -> {1, \"no\", \"3.14\", \"null\", \"x y\"}, \"b\" -> <||>, \"c\" -> {}, \"m\" -> \"line1\\nline2\", \"t\" -> \"line1\\n\"|>]], <|\"a\" -> {1, \"no\", \"3.14\", \"null\", \"x y\"}, \"b\" -> <||>, \"c\" -> {}, \"m\" -> \"line1\\nline2\", \"t\" -> \"line1\\n\"|>]"
        ),
        "True"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToYaml[<|\"a\" -> {1, \"no\", \"3.14\"}, \"b\" -> <||>|>]"
        ),
        "a:\n  - 1\n  - \"no\"\n  - \"3.14\"\nb: {}"
    );

    let outcome = session.eval_text("ToYaml[<|x -> 1|>]").unwrap();
    assert_failed_with_message(
        outcome.value,
        outcome.messages,
        "ToYaml",
        "yaml",
        "mapping keys must be Strings",
    );

    let outcome = session.eval_text("ToYaml[<|\"x\" -> 1/3|>]").unwrap();
    assert_failed_with_message(
        outcome.value,
        outcome.messages,
        "ToYaml",
        "yaml",
        "non-terminating Rational",
    );
}

#[test]
fn yaml_codecs_compose_with_json() {
    let mut session = core_session();

    assert_eq!(
        eval_string_atom(&mut session, &format!("ToJson[FromYaml[{:?}]]", K8SISH)),
        "{\"svc\":{\"name\":\"web\",\"replicas\":2,\"enabled\":true,\"pi\":3.14,\"hosts\":[\"a\",\"b\"],\"containers\":[{\"name\":\"web\",\"image\":\"nginx:1.25\",\"env\":[{\"name\":\"COUNTRY\",\"value\":\"no\"}]}],\"note\":\"line1\\nline2\"}}"
    );
    assert_eq!(
        eval_string_atom(
            &mut session,
            "ToYaml[FromJson[\"{\\\"srv\\\":{\\\"port\\\":8080,\\\"hosts\\\":[\\\"a\\\",\\\"b\\\"]}}\"]]"
        ),
        "srv:\n  port: 8080\n  hosts:\n    - a\n    - b"
    );
}
