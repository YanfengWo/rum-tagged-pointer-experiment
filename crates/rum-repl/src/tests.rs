use std::io::Cursor;

use rum_runtime::Session;

use super::Repl;

#[test]
fn repl_handles_help_and_quit() {
    let input = Cursor::new(":help\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains(":inspect NAME"));
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
fn repl_inspect_surfaces_message_definition_flags() {
    let input = Cursor::new(":inspect f\n:quit\n");
    let mut output = Vec::new();
    let mut repl = Repl::new(input, &mut output);
    let mut session = Session::new();
    session.eval_text("f::argx = \"bad arg\"").unwrap();

    repl.run(&mut session).unwrap();

    let rendered = String::from_utf8(output).unwrap();
    assert!(rendered.contains("[msg]"));
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
