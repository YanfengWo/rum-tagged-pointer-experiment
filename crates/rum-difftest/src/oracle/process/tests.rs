use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::*;

#[cfg(unix)]
#[test]
fn captures_successful_process_output() {
    let executable = Path::new("/bin/sh");
    let mut command = Command::new(executable);
    command.args(["-c", "printf stdout-text; printf stderr-text >&2"]);

    let output = run_with_timeout(&mut command, executable, Duration::from_secs(1), 1024).unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"stdout-text");
    assert_eq!(output.stderr, b"stderr-text");
}

#[cfg(unix)]
#[test]
fn terminates_process_after_timeout() {
    let executable = Path::new("/bin/sleep");
    let mut command = Command::new(executable);
    command.arg("1");

    let err =
        run_with_timeout(&mut command, executable, Duration::from_millis(20), 1024).unwrap_err();

    assert!(matches!(
        err,
        OracleError::TimedOut {
            timeout_millis: 20,
            ..
        }
    ));
}

#[cfg(unix)]
#[test]
fn terminates_process_after_output_limit() {
    let executable = Path::new("/usr/bin/yes");
    let mut command = Command::new(executable);

    let err = run_with_timeout(&mut command, executable, Duration::from_secs(1), 1024).unwrap_err();

    match err {
        OracleError::OutputLimitExceeded {
            output_limit_bytes,
            streams,
            stdout,
            stderr,
            ..
        } => {
            assert_eq!(output_limit_bytes, 1024);
            assert_eq!(streams, "stdout");
            assert_eq!(stdout.len(), 1024);
            assert!(stderr.is_empty());
        }
        other => panic!("expected output limit error, got {other:?}"),
    }
}
