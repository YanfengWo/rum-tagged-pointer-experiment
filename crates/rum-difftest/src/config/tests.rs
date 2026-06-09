use super::*;

#[test]
fn parses_oracle_kernel_from_config() {
    let config = parse_config(
        r#"
            # comment
            oracle_kernel = "/tmp/MathKernel"
        "#,
    )
    .unwrap();

    assert_eq!(config.oracle_kernel, Some(PathBuf::from("/tmp/MathKernel")));
}

#[test]
fn parses_oracle_command_alias_from_config() {
    let config = parse_config("oracle_command = 'WolframKernel'").unwrap();
    assert_eq!(config.oracle_kernel, Some(PathBuf::from("WolframKernel")));
}

#[test]
fn parses_oracle_timeout_from_config() {
    let config = parse_config("oracle_timeout_seconds = 45").unwrap();
    assert_eq!(config.oracle_timeout_seconds, Some(45));
    assert_eq!(
        resolve_oracle_timeout(None, &config).unwrap(),
        Duration::from_secs(45)
    );
}

#[test]
fn cli_oracle_timeout_overrides_config() {
    let config = DiffTestConfig {
        oracle_kernel: None,
        oracle_timeout_seconds: Some(45),
    };
    assert_eq!(
        resolve_oracle_timeout_with_env(Some(9), &config, Some(std::ffi::OsStr::new("3"))).unwrap(),
        Duration::from_secs(9)
    );
}

#[test]
fn config_oracle_timeout_overrides_environment() {
    let config = DiffTestConfig {
        oracle_kernel: None,
        oracle_timeout_seconds: Some(45),
    };
    assert_eq!(
        resolve_oracle_timeout_with_env(None, &config, Some(std::ffi::OsStr::new("3"))).unwrap(),
        Duration::from_secs(45)
    );
}

#[test]
fn environment_oracle_timeout_overrides_default() {
    assert_eq!(
        resolve_oracle_timeout_with_env(
            None,
            &DiffTestConfig::default(),
            Some(std::ffi::OsStr::new("3"))
        )
        .unwrap(),
        Duration::from_secs(3)
    );
    assert_eq!(
        resolve_oracle_timeout_with_env(None, &DiffTestConfig::default(), None).unwrap(),
        DEFAULT_ORACLE_TIMEOUT
    );
}

#[test]
fn rejects_invalid_oracle_timeouts() {
    for value in ["0", "-1", "later"] {
        let err = parse_config(&format!("oracle_timeout_seconds = {value}")).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidPositiveInteger { .. }));
    }
    let err = resolve_oracle_timeout(Some(0), &DiffTestConfig::default()).unwrap_err();
    assert!(matches!(err, ConfigError::InvalidPositiveInteger { .. }));
    let err = resolve_oracle_timeout_with_env(
        None,
        &DiffTestConfig::default(),
        Some(std::ffi::OsStr::new("never")),
    )
    .unwrap_err();
    assert!(matches!(err, ConfigError::InvalidPositiveInteger { .. }));
}

#[test]
fn common_macos_kernel_paths_include_wolfram_app() {
    assert!(common_macos_kernel_paths().contains(&PathBuf::from(
        "/Applications/Wolfram.app/Contents/MacOS/WolframKernel"
    )));
}

#[test]
fn rejects_unknown_config_keys() {
    let err = parse_config("unexpected = 1").unwrap_err();
    assert!(matches!(err, ConfigError::UnknownKey { .. }));
}

#[test]
fn rejects_oracle_version_pin_config_keys() {
    for key in ["oracle_version", "required_oracle_version"] {
        let err = parse_config(&format!("{key} = '14.3'")).unwrap_err();
        assert!(
            matches!(err, ConfigError::UnknownKey { key: ref unknown } if unknown == key),
            "expected `{key}` to remain unsupported, got {err:?}"
        );
    }
}
