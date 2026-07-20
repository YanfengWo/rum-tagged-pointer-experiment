use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

use rum_host::{HostEvalOutcome, HostSession};

use super::Repl;

fn eval_text(session: &mut HostSession, source: &str) {
    match session.eval(source) {
        HostEvalOutcome::Success(_) => {}
        other => panic!("test setup evaluation failed: {other:?}"),
    }
}

fn temp_history_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("rum-repl-{prefix}-{unique}.history"))
}

#[test]
fn repl_handles_help_and_quit() {
    let input = Cursor::new(":help\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> :help           Show REPL commands.
:parse EXPR     Parse and print the syntax tree.
:format [--line-width N] EXPR  Format syntax into the current canonical style.
:lower EXPR     Lower syntax into core Expr form.
:complete PREF  Show symbol and context completions.
:history [N]    Show recent input history.
:inspect NAME   Show basic symbol metadata.
:messages [NAME]   Show message definitions globally or for a symbol/message.
:quit           Exit the REPL.
rum> "#
    );
}

#[test]
fn repl_inspect_command_does_not_fall_through_to_evaluation() {
    let input = Cursor::new(":inspect x\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "x = 1");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Global`x ::  [own]
  context: Global`
  own: x = 1
rum> "#
    );
}

#[test]
fn repl_bare_inspect_command_shows_usage_instead_of_evaluating() {
    let input = Cursor::new(":inspect\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> ! :inspect expects a symbol name
rum> "#
    );
}

#[test]
fn repl_inspect_surfaces_message_definition_flags() {
    let input = Cursor::new(":inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "f::argx = \"bad arg\"");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Global`f ::  [msg]
  context: Global`
  messages[1]: argx
rum> "#
    );
}

#[test]
fn repl_inspect_surfaces_attribute_flags() {
    let input = Cursor::new(":inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "SetAttributes[f, HoldFirst]");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Global`f ::  [attr]
  context: Global`
  attributes[1]: HoldFirst
rum> "#
    );
}

#[test]
fn repl_inspect_shows_definition_summaries() {
    let input = Cursor::new(":inspect x\n:inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "x = 1");
    eval_text(&mut session, "f[z_Integer] := z");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Global`x ::  [own]
  context: Global`
  own: x = 1
rum> Global`f ::  [down]
  context: Global`
  down[1]: f[Pattern[z, Blank[Integer]]] := z
rum> "#
    );
}

#[test]
fn repl_messages_lists_catalog_entries_for_a_symbol() {
    let input = Cursor::new(":messages f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "f::argx = \"bad arg\"");
    eval_text(&mut session, "f::usage := suffix");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> f::argx = "bad arg"
f::usage := suffix
rum> "#
    );
}

#[test]
fn repl_messages_reports_when_a_symbol_has_no_messages() {
    let input = Cursor::new(":messages f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> ! no messages for `f`
rum> "#
    );
}

#[test]
fn repl_renders_runtime_messages_before_results() {
    let input = Cursor::new("FindFile[\"Missing`\"]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> ! FindFile::notfound: could not locate `Missing`` in ./Missing.rum, ./Missing/Kernel/init.rum, ./Missing/init.rum, ./Missing.wl, ./Missing/Kernel/init.wl, ./Missing/init.wl, ./Missing.m, ./Missing/Kernel/init.m, ./Missing/init.m
$Failed
rum> "#
    );
}

#[test]
fn repl_renders_hold_form_transparently_in_canonical_style() {
    let input = Cursor::new("HoldForm[1 + 1]\n{HoldForm[1 + 1]}\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Plus[1, 1]
rum> List[Plus[1, 1]]
rum> "#
    );
}

#[test]
fn repl_reads_multiline_input_until_a_form_is_complete() {
    let input = Cursor::new("Identity[\n123\n]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> ...> ...> 123
rum> "#
    );
}

#[test]
fn repl_parse_command_prints_syntax_debug_output() {
    let input = Cursor::new(":parse f[(* note *) x_Integer]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> syntax:
SyntaxExpr {
    kind: Call {
        head: SyntaxExpr {
            kind: Symbol(
                "f",
            ),
            span: Span {
                start: 0,
                end: 1,
            },
        },
        args: [
            SyntaxExpr {
                kind: Pattern {
                    name: SyntaxExpr {
                        kind: Symbol(
                            "x",
                        ),
                        span: Span {
                            start: 13,
                            end: 14,
                        },
                    },
                    pattern: SyntaxExpr {
                        kind: Blank {
                            kind: Blank,
                            head: Some(
                                SyntaxExpr {
                                    kind: Symbol(
                                        "Integer",
                                    ),
                                    span: Span {
                                        start: 15,
                                        end: 22,
                                    },
                                },
                            ),
                        },
                        span: Span {
                            start: 14,
                            end: 22,
                        },
                    },
                },
                span: Span {
                    start: 13,
                    end: 22,
                },
            },
        ],
    },
    span: Span {
        start: 0,
        end: 23,
    },
}
tokens:
  [0] symbol @ 0..1 (1:1) text="f"
  [1] [ @ 1..2 (1:2) text="["
  [2] symbol @ 13..14 (1:14) text="x"
       trivia comment @ 2..12 text="(* note *)"
       trivia whitespace @ 12..13 text=" "
  [3] _ @ 14..15 (1:15) text="_"
  [4] symbol @ 15..22 (1:16) text="Integer"
  [5] ] @ 22..23 (1:23) text="]"
  [6] end of input @ 23..23 (1:24) text=""

rum> "#
    );
}

#[test]
fn repl_parse_command_renders_source_diagnostic_for_bad_input() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let input = Cursor::new(":parse f[1,]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> error: expected expression, found ] at bytes 4..5
 --> 1:5
  |
1 | f[1,]
  |     ^
rum> "#
    );
}

#[test]
fn repl_format_command_prints_canonical_surface_form() {
    let input = Cursor::new(":format (* header *) f [ x_Integer , y ]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> (* header *) f[x_Integer, y]
rum> "#
    );
}

#[test]
fn repl_format_command_respects_custom_line_width() {
    let input =
        Cursor::new(":format --line-width 60 f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]
rum> "#
    );
}

#[test]
fn repl_format_command_accepts_suffix_equals_line_width() {
    let input =
        Cursor::new(":format f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb] --line-width=60\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]
rum> "#
    );
}

#[test]
fn repl_format_command_rejects_invalid_line_width() {
    let input = Cursor::new(":format --line-width nope f[x]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> ! --line-width expects a positive integer
rum> "#
    );
}

#[test]
fn repl_renders_parser_diagnostics_for_regular_input_errors() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let input = Cursor::new("f[1,]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> error: expected expression, found ] at bytes 4..5
 --> 1:5
  |
1 | f[1,]
  |     ^
rum> "#
    );
}

#[test]
fn repl_lower_command_prints_canonical_core_expression() {
    let input = Cursor::new(":lower x_Integer := f[x]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> SetDelayed[Pattern[x, Blank[Integer]], f[x]]
rum> "#
    );
}

#[test]
fn repl_complete_command_lists_builtin_and_user_candidates() {
    let input = Cursor::new(":complete Id\n:complete loc\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();
    eval_text(&mut session, "localName = 1");

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum> Identity
rum> localName
rum> "#
    );
}

#[test]
fn repl_history_persists_when_a_history_path_is_configured() {
    let history_path = temp_history_path("persist");

    {
        let input = Cursor::new("1 + 2\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
        let mut session = HostSession::new();

        repl.run(&mut session).unwrap();
    }

    let input = Cursor::new(":history\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum>    1: 1 + 2
   2: :quit
rum> "#
    );

    let persisted = fs::read_to_string(&history_path).unwrap();
    assert_eq!(
        persisted,
        r#"1 + 2
:quit
:history
:quit
"#
    );

    let _ = fs::remove_file(history_path);
}

#[test]
fn repl_history_persists_multiline_inputs_as_single_logical_entries() {
    let history_path = temp_history_path("multiline");

    {
        let input = Cursor::new("Identity[\n123\n]\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
        let mut session = HostSession::new();

        repl.run(&mut session).unwrap();
    }

    let persisted = fs::read_to_string(&history_path).unwrap();
    assert_eq!(
        persisted,
        r#":rum-history-v1:Identity[\n123\n]
:quit
"#
    );
    assert_eq!(persisted.lines().count(), 2);

    let input = Cursor::new(":history\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        r#"rum>    1: Identity[\n123\n]
   2: :quit
rum> "#
    );

    let _ = fs::remove_file(history_path);
}

#[test]
fn repl_reports_incomplete_multiline_input_when_input_ends() {
    let input = Cursor::new("Identity[\n123\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = HostSession::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(
        rendered,
        concat!(
            "rum> ...> ...> \n",
            r#"error: incomplete input: expected ] at bytes 13..13
 --> 2:4
  |
2 | 123
  |    ^
  |
1 | Identity[
  |         ^ opening `[` for expected `]`
"#
        )
    );
}
