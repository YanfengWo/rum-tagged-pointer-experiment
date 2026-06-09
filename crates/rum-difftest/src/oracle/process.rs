use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::OracleError;

const POLL_INTERVAL: Duration = Duration::from_millis(10);
const STDOUT_LIMIT_EXCEEDED: u8 = 1;
const STDERR_LIMIT_EXCEEDED: u8 = 2;

pub fn run_with_timeout(
    command: &mut Command,
    executable: &Path,
    timeout: Duration,
    output_limit_bytes: usize,
) -> Result<Output, OracleError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| OracleError::Spawn {
        executable: executable.to_path_buf(),
        source,
    })?;
    let stdout = child
        .stdout
        .take()
        .expect("stdout was configured as a pipe");
    let stderr = child
        .stderr
        .take()
        .expect("stderr was configured as a pipe");
    let output_limit_flags = Arc::new(AtomicU8::new(0));
    let stdout_reader = spawn_reader(
        stdout,
        output_limit_bytes,
        STDOUT_LIMIT_EXCEEDED,
        Arc::clone(&output_limit_flags),
    );
    let stderr_reader = spawn_reader(
        stderr,
        output_limit_bytes,
        STDERR_LIMIT_EXCEEDED,
        Arc::clone(&output_limit_flags),
    );
    let started = Instant::now();

    let status = loop {
        if output_limit_flags.load(Ordering::Relaxed) != 0 {
            terminate_child(&mut child, executable)?;
            return collect_output_limit_error(
                stdout_reader,
                stderr_reader,
                executable,
                output_limit_bytes,
                &output_limit_flags,
            );
        }

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                terminate_child(&mut child, executable)?;
                let stdout = join_reader(stdout_reader, executable, "stdout")?;
                let stderr = join_reader(stderr_reader, executable, "stderr")?;
                let flags = output_limit_flags.load(Ordering::Relaxed);
                if flags != 0 {
                    return Err(output_limit_error(
                        executable,
                        output_limit_bytes,
                        flags,
                        stdout,
                        stderr,
                    ));
                }
                return Err(OracleError::TimedOut {
                    executable: executable.to_path_buf(),
                    timeout_millis: timeout.as_millis(),
                    stdout: String::from_utf8_lossy(&stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&stderr).into_owned(),
                });
            }
            Ok(None) => {
                let remaining = timeout.saturating_sub(started.elapsed());
                thread::sleep(POLL_INTERVAL.min(remaining));
            }
            Err(source) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(OracleError::Wait {
                    executable: executable.to_path_buf(),
                    source,
                });
            }
        }
    };

    collect_output(
        status,
        stdout_reader,
        stderr_reader,
        executable.to_path_buf(),
        output_limit_bytes,
        &output_limit_flags,
    )
}

fn spawn_reader<R>(
    mut reader: R,
    output_limit_bytes: usize,
    limit_flag: u8,
    output_limit_flags: Arc<AtomicU8>,
) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let count = reader.read(&mut chunk)?;
            if count == 0 {
                break;
            }
            let remaining = output_limit_bytes.saturating_sub(bytes.len());
            let retained = count.min(remaining);
            bytes.extend_from_slice(&chunk[..retained]);
            if retained < count {
                output_limit_flags.fetch_or(limit_flag, Ordering::Relaxed);
            }
        }
        Ok(bytes)
    })
}

fn terminate_child(child: &mut std::process::Child, executable: &Path) -> Result<(), OracleError> {
    if let Err(source) = child.kill() {
        if source.kind() != std::io::ErrorKind::InvalidInput {
            return Err(OracleError::Kill {
                executable: executable.to_path_buf(),
                source,
            });
        }
    }
    child.wait().map_err(|source| OracleError::Wait {
        executable: executable.to_path_buf(),
        source,
    })?;
    Ok(())
}

fn collect_output(
    status: ExitStatus,
    stdout_reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stderr_reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    executable: PathBuf,
    output_limit_bytes: usize,
    output_limit_flags: &AtomicU8,
) -> Result<Output, OracleError> {
    let stdout = join_reader(stdout_reader, &executable, "stdout")?;
    let stderr = join_reader(stderr_reader, &executable, "stderr")?;
    let output_limit_flags = output_limit_flags.load(Ordering::Relaxed);
    if output_limit_flags != 0 {
        return Err(output_limit_error(
            &executable,
            output_limit_bytes,
            output_limit_flags,
            stdout,
            stderr,
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn join_reader(
    reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    executable: &Path,
    stream: &'static str,
) -> Result<Vec<u8>, OracleError> {
    match reader.join() {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(source)) => Err(OracleError::OutputCapture {
            executable: executable.to_path_buf(),
            stream,
            source,
        }),
        Err(_) => Err(OracleError::OutputReaderPanicked {
            executable: executable.to_path_buf(),
            stream,
        }),
    }
}

fn collect_output_limit_error(
    stdout_reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stderr_reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    executable: &Path,
    output_limit_bytes: usize,
    output_limit_flags: &AtomicU8,
) -> Result<Output, OracleError> {
    let stdout = join_reader(stdout_reader, executable, "stdout")?;
    let stderr = join_reader(stderr_reader, executable, "stderr")?;
    let output_limit_flags = output_limit_flags.load(Ordering::Relaxed);
    Err(output_limit_error(
        executable,
        output_limit_bytes,
        output_limit_flags,
        stdout,
        stderr,
    ))
}

fn output_limit_error(
    executable: &Path,
    output_limit_bytes: usize,
    output_limit_flags: u8,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> OracleError {
    OracleError::OutputLimitExceeded {
        executable: executable.to_path_buf(),
        output_limit_bytes,
        streams: exceeded_stream_names(output_limit_flags),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    }
}

fn exceeded_stream_names(flags: u8) -> &'static str {
    match flags & (STDOUT_LIMIT_EXCEEDED | STDERR_LIMIT_EXCEEDED) {
        STDOUT_LIMIT_EXCEEDED => "stdout",
        STDERR_LIMIT_EXCEEDED => "stderr",
        _ => "stdout and stderr",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn captures_successful_process_output() {
        let executable = Path::new("/bin/sh");
        let mut command = Command::new(executable);
        command.args(["-c", "printf stdout-text; printf stderr-text >&2"]);

        let output =
            run_with_timeout(&mut command, executable, Duration::from_secs(1), 1024).unwrap();

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

        let err = run_with_timeout(&mut command, executable, Duration::from_millis(20), 1024)
            .unwrap_err();

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

        let err =
            run_with_timeout(&mut command, executable, Duration::from_secs(1), 1024).unwrap_err();

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
}
