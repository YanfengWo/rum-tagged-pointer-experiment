use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

use rum_runtime::Session;
use rum_stdlib::install_core;

use super::Repl;

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
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(":inspect NAME"));
    assert!(rendered.contains(":format [--line-width N] EXPR"));
}

#[test]
fn repl_inspect_command_does_not_fall_through_to_evaluation() {
    let input = Cursor::new(":inspect x\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("x = 1").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("x ::"));
    assert!(!rendered.contains("unexpected character"));
}

#[test]
fn repl_bare_inspect_command_shows_usage_instead_of_evaluating() {
    let input = Cursor::new(":inspect\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("! :inspect expects a symbol name"));
    assert!(!rendered.contains("unexpected character"));
}

#[test]
fn repl_inspect_surfaces_message_definition_flags() {
    let input = Cursor::new(":inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("f::argx = \"bad arg\"").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("[msg]"));
    assert!(rendered.contains("messages[1]: argx"));
}

#[test]
fn repl_inspect_surfaces_attribute_flags() {
    let input = Cursor::new(":inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("[attr]"));
    assert!(rendered.contains("attributes[1]: HoldFirst"));
}

#[test]
fn repl_inspect_shows_definition_summaries() {
    let input = Cursor::new(":inspect x\n:inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("x = 1").unwrap();
    session.eval_text("f[z_Integer] := z").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("context: Global`"));
    assert!(rendered.contains("own: x = 1"));
    assert!(rendered.contains("down[1]: f[Pattern[z, Blank[Integer]]] := z"));
}

#[test]
fn repl_messages_lists_catalog_entries_for_a_symbol() {
    let input = Cursor::new(":messages f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("f::argx = \"bad arg\"").unwrap();
    session.eval_text("f::usage := suffix").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("f::argx = \"bad arg\""));
    assert!(rendered.contains("f::usage := suffix"));
}

#[test]
fn repl_messages_reports_when_a_symbol_has_no_messages() {
    let input = Cursor::new(":messages f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("! no messages for `f`"));
}

#[test]
fn repl_renders_runtime_messages_before_results() {
    let input = Cursor::new("FindFile[\"Missing`\"]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("! FindFile::notfound:"));
    assert!(rendered.contains("$Failed"));
}

#[test]
fn repl_reads_multiline_input_until_a_form_is_complete() {
    let input = Cursor::new("Identity[\n123\n]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("rum> ...> ...> 123"));
}

#[test]
fn repl_parse_command_prints_syntax_debug_output() {
    let input = Cursor::new(":parse f[(* note *) x_Integer]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("syntax:"));
    assert!(rendered.contains("Pattern"));
    assert!(rendered.contains("tokens:"));
    assert!(rendered.contains("trivia comment"));
    assert!(rendered.contains("(* note *)"));
}

#[test]
fn repl_parse_command_renders_source_diagnostic_for_bad_input() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let input = Cursor::new(":parse f[1,]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("error: expected expression"));
    assert!(rendered.contains("--> 1:5"));
    assert!(rendered.contains("1 | f[1,]"));
}

#[test]
fn repl_format_command_prints_canonical_surface_form() {
    let input = Cursor::new(":format (* header *) f [ x_Integer , y ]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("(* header *)"));
    assert!(rendered.contains("f[x_Integer, y]"));
}

#[test]
fn repl_format_command_respects_custom_line_width() {
    let input =
        Cursor::new(":format --line-width 60 f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]"));
}

#[test]
fn repl_format_command_accepts_suffix_equals_line_width() {
    let input =
        Cursor::new(":format f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb] --line-width=60\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]"));
}

#[test]
fn repl_format_command_rejects_invalid_line_width() {
    let input = Cursor::new(":format --line-width nope f[x]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("! --line-width expects a positive integer"));
}

#[test]
fn repl_renders_parser_diagnostics_for_regular_input_errors() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let input = Cursor::new("f[1,]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("error: expected expression"));
    assert!(rendered.contains("--> 1:5"));
}

#[test]
fn repl_lower_command_prints_canonical_core_expression() {
    let input = Cursor::new(":lower x_Integer := f[x]\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("SetDelayed[Pattern[x, Blank[Integer]], f[x]]"));
}

#[test]
fn repl_complete_command_lists_builtin_and_user_candidates() {
    let input = Cursor::new(":complete Id\n:complete loc\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    install_core(&mut session);
    session.eval_text("localName = 1").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("Identity"));
    assert!(rendered.contains("localName"));
}

#[test]
fn repl_history_persists_when_a_history_path_is_configured() {
    let history_path = temp_history_path("persist");

    {
        let input = Cursor::new("1 + 2\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();
    }

    let input = Cursor::new(":history\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("1 + 2"));

    let persisted = fs::read_to_string(&history_path).unwrap();
    assert!(persisted.contains("1 + 2"));

    let _ = fs::remove_file(history_path);
}

#[test]
fn repl_history_persists_multiline_inputs_as_single_logical_entries() {
    let history_path = temp_history_path("multiline");

    {
        let input = Cursor::new("Identity[\n123\n]\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();
    }

    let persisted = fs::read_to_string(&history_path).unwrap();
    assert!(persisted.contains("Identity[\\n123\\n]"));
    assert_eq!(persisted.lines().count(), 2);

    let input = Cursor::new(":history\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output).with_history_path(&history_path);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("Identity[\\n123\\n]"));
    assert!(!rendered.contains("   2: 123"));

    let _ = fs::remove_file(history_path);
}

#[test]
fn repl_reports_incomplete_multiline_input_when_input_ends() {
    let input = Cursor::new("Identity[\n123\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("rum> ...> ...> "));
    assert!(rendered.contains("error: incomplete input: expected ]"));
    assert!(rendered.contains("--> 2:4"));
}
