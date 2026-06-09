use std::fs;
use std::path::Path;
use std::process::Command;

use super::super::process::run_with_timeout;
use super::OracleKernel;
use crate::oracle::{OracleError, OracleEvaluation};

const SESSION_RESULT_MARKER: &str = "__RUM_DIFFTEST_SESSION_RESULT__=";

pub(super) fn evaluate_session(
    kernel: &mut OracleKernel,
    inputs: &[String],
) -> Result<Vec<OracleEvaluation>, OracleError> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }

    kernel.next_case_id += 1;
    let session_dir = kernel
        .temp_dir
        .path()
        .join(format!("session-{:04}", kernel.next_case_id));
    fs::create_dir(&session_dir).map_err(|source| OracleError::Io {
        path: session_dir.clone(),
        source,
    })?;

    let mut input_paths = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        let path = session_dir.join(format!("step-{:04}.wl", index + 1));
        fs::write(&path, input).map_err(|source| OracleError::Io {
            path: path.clone(),
            source,
        })?;
        input_paths.push(path);
    }

    let manifest_path = session_dir.join("manifest.txt");
    let manifest = input_paths
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&manifest_path, manifest).map_err(|source| OracleError::Io {
        path: manifest_path.clone(),
        source,
    })?;

    let mut command = Command::new(&kernel.executable);
    command
        .arg("-noprompt")
        .arg("-script")
        .arg(&kernel.session_wrapper_path)
        .arg(&manifest_path);
    let output = run_with_timeout(
        &mut command,
        &kernel.executable,
        kernel.timeout,
        kernel.output_limit_bytes,
    )?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(OracleError::ExecutionFailed {
            executable: kernel.executable.clone(),
            code: output.status.code(),
            stdout,
            stderr,
        });
    }

    let results = parse_session_results(kernel, inputs.len(), &stdout, &stderr)?;
    build_session_evaluations(&session_dir, results, &stdout, &stderr)
}

fn parse_session_results(
    kernel: &OracleKernel,
    expected_count: usize,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<Option<String>>, OracleError> {
    let mut results = vec![None; expected_count];

    for line in stdout.lines() {
        if let Some(payload) = line.strip_prefix(SESSION_RESULT_MARKER) {
            let Some((index_text, value)) = payload.split_once('\t') else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            let Ok(index) = index_text.parse::<usize>() else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            if index == 0 || index > expected_count {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            }
            results[index - 1] = Some(value.trim().to_owned());
        }
    }

    let actual = results.iter().filter(|result| result.is_some()).count();
    if actual != expected_count {
        return Err(OracleError::SessionResultCountMismatch {
            executable: kernel.executable.clone(),
            expected: expected_count,
            actual,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        });
    }

    Ok(results)
}

fn build_session_evaluations(
    session_dir: &Path,
    results: Vec<Option<String>>,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<OracleEvaluation>, OracleError> {
    let printed_output = collect_process_prints(stdout);
    let process_stderr = stderr.trim().to_owned();
    let mut evaluations = Vec::with_capacity(results.len());
    for (index, result) in results.into_iter().enumerate() {
        let suffix = format!("{:04}", index + 1);
        let messages_path = session_dir.join(format!("messages-{suffix}.txt"));
        let prints_path = session_dir.join(format!("prints-{suffix}.txt"));
        let mut step_messages =
            read_trimmed_text(&messages_path).map_err(|source| OracleError::Io {
                path: messages_path.clone(),
                source,
            })?;
        if !process_stderr.is_empty() {
            step_messages = join_nonempty_lines(&[&step_messages, &process_stderr]);
        }
        let mut step_prints =
            read_trimmed_text(&prints_path).map_err(|source| OracleError::Io {
                path: prints_path.clone(),
                source,
            })?;
        if !printed_output.is_empty() {
            step_prints = join_nonempty_lines(&[&step_prints, &printed_output]);
        }

        evaluations.push(OracleEvaluation {
            result: result.expect("result count was already validated"),
            printed_output: step_prints,
            stderr: step_messages,
        });
    }

    Ok(evaluations)
}

fn collect_process_prints(stdout: &str) -> String {
    stdout
        .lines()
        .filter(|line| !line.starts_with(SESSION_RESULT_MARKER))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn invalid_session_marker(
    kernel: &OracleKernel,
    line: &str,
    stdout: &str,
    stderr: &str,
) -> OracleError {
    OracleError::InvalidSessionResultMarker {
        executable: kernel.executable.clone(),
        line: line.to_owned(),
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    }
}

fn read_trimmed_text(path: &Path) -> Result<String, std::io::Error> {
    Ok(fs::read_to_string(path)?.trim().to_owned())
}

fn join_nonempty_lines(parts: &[&str]) -> String {
    parts
        .iter()
        .copied()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
