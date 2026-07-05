use super::*;

#[test]
fn default_session_configuration_matches_default_session_state() {
    let session = Session::new();

    assert_eq!(
        session.configuration(),
        SessionConfig::default().normalized()
    );
}

#[test]
fn session_with_config_normalizes_contexts_paths_and_limits() {
    let config = SessionConfig::default()
        .with_resource_limits(ResourceLimits {
            recursion_limit: 17,
            native_stack_limit: 19,
            replace_repeated_limit: 23,
            evaluation_pass_limit: 31,
            generated_element_limit: 37,
            integer_work_limit: 41,
            iteration_work_limit: 43,
        })
        .with_current_context("Rules")
        .with_context_path(["System", "Rules", "Global", "Rules"])
        .with_package_search_paths(["./pkg-a", "./pkg-a", "./pkg-b"])
        .with_tracing(true)
        .with_asserts_enabled(true);
    let session = Session::with_config(config);

    assert_eq!(session.current_context(), "Rules`");
    assert_eq!(
        session.context_path(),
        &[
            "System`".to_string(),
            "Rules`".to_string(),
            "Global`".to_string(),
        ]
    );
    assert_eq!(
        session.package_search_paths(),
        &[PathBuf::from("./pkg-a"), PathBuf::from("./pkg-b")]
    );
    assert_eq!(
        session.resource_limits(),
        ResourceLimits {
            recursion_limit: 17,
            native_stack_limit: 19,
            replace_repeated_limit: 23,
            evaluation_pass_limit: 31,
            generated_element_limit: 37,
            integer_work_limit: 41,
            iteration_work_limit: 43,
        }
    );
    assert!(session.configuration().tracing_enabled);
    assert!(session.configuration().asserts_enabled);
}

#[test]
fn apply_config_updates_host_settings_without_clearing_existing_definitions() {
    let mut session = Session::new();
    session.eval_text("x = 1").unwrap();

    session.apply_config(
        SessionConfig::default()
            .with_current_context("Rules")
            .with_context_path(["System", "Global"])
            .with_package_search_paths(["./packages"])
            .with_tracing(true)
            .with_asserts_enabled(true)
            .with_resource_limits(ResourceLimits {
                recursion_limit: 9,
                native_stack_limit: 15,
                replace_repeated_limit: 10,
                evaluation_pass_limit: 11,
                generated_element_limit: 12,
                integer_work_limit: 13,
                iteration_work_limit: 14,
            }),
    );

    assert_eq!(session.current_context(), "Rules`");
    assert_eq!(
        session.context_path(),
        &["System`".to_string(), "Global`".to_string()]
    );
    assert_eq!(
        session.package_search_paths(),
        &[PathBuf::from("./packages")]
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    assert_eq!(
        session.resource_limits(),
        ResourceLimits {
            recursion_limit: 9,
            native_stack_limit: 15,
            replace_repeated_limit: 10,
            evaluation_pass_limit: 11,
            generated_element_limit: 12,
            integer_work_limit: 13,
            iteration_work_limit: 14,
        }
    );
    assert!(session.configuration().tracing_enabled);
    assert!(session.configuration().asserts_enabled);
}
