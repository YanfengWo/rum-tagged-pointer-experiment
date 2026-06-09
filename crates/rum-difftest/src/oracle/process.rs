use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::OracleError;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub fn run_with_timeout(
    command: &mut Command,
    executable: &Path,
    timeout: Duration,
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
    let stdout_reader = spawn_reader(stdout);
    let stderr_reader = spawn_reader(stderr);
    let started = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                terminate_child(&mut child, executable)?;
                let stdout = join_reader(stdout_reader, executable, "stdout")?;
                let stderr = join_reader(stderr_reader, executable, "stderr")?;
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
    )
}

fn spawn_reader<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
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
) -> Result<Output, OracleError> {
    let stdout = join_reader(stdout_reader, &executable, "stdout")?;
    let stderr = join_reader(stderr_reader, &executable, "stderr")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn captures_successful_process_output() {
        let executable = Path::new("/bin/sh");
        let mut command = Command::new(executable);
        command.args(["-c", "printf stdout-text; printf stderr-text >&2"]);

        let output = run_with_timeout(&mut command, executable, Duration::from_secs(1)).unwrap();

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
            run_with_timeout(&mut command, executable, Duration::from_millis(20)).unwrap_err();

        assert!(matches!(
            err,
            OracleError::TimedOut {
                timeout_millis: 20,
                ..
            }
        ));
    }
}
