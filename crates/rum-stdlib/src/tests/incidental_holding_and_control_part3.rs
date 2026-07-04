use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn module_result_escape_retains_gensym_definition() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("held049 = Module[{x}, x = 7; Hold[x]]; First[held049]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_cross_gensym_definition_retains_transitive_dependency() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("held049 = Module[{a, b}, b = 5; a := b; Hold[a]]; First[held049]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn module_options_and_sow_escapes_retain_gensym_definitions() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "Module[{x}, x = 7; Options[ggOpts049] = {o -> Hold[x]}; Null]; First[OptionValue[ggOpts049, o]]",
            )
            .unwrap()
            .value
            .to_string(),
        "7"
    );

    assert_eq!(
        session
            .eval_text("First[Part[Reap[Module[{x}, x = 7; Sow[Hold[x]]; Null]], 2, 1, 1]]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn module_nested_and_error_paths_remain_conservative() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Module[{r}, r = Module[{t}, t = 1; t]; r]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    assert_eq!(
        session
            .eval_text("First[Catch[Module[{x}, x = 7; Throw[Hold[x]]]]]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}
