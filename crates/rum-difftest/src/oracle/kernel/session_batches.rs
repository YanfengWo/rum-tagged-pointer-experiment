use std::fs;
use std::path::Path;
use std::process::Command;

use super::super::process::run_with_timeout;
use super::OracleKernel;
use crate::oracle::{OracleError, OracleEvaluation};

const SESSION_BATCH_RESULT_MARKER: &str = "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=";

pub(super) fn evaluate_session_batches(
    kernel: &mut OracleKernel,
    batches: &[Vec<String>],
) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
    if batches.is_empty() {
        return Ok(Vec::new());
    }
    if batches.iter().all(Vec::is_empty) {
        return Ok(batches.iter().map(|_| Vec::new()).collect());
    }

    kernel.next_case_id += 1;
    let root_dir = kernel
        .temp_dir
        .path()
        .join(format!("session-batches-{:04}", kernel.next_case_id));
    fs::create_dir(&root_dir).map_err(|source| OracleError::Io {
        path: root_dir.clone(),
        source,
    })?;

    let batch_manifest_paths = write_batch_inputs(&root_dir, batches)?;
    let root_manifest_path = root_dir.join("batch-manifest.txt");
    let root_manifest = batch_manifest_paths
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&root_manifest_path, root_manifest).map_err(|source| OracleError::Io {
        path: root_manifest_path.clone(),
        source,
    })?;

    let mut command = Command::new(&kernel.executable);
    command
        .arg("-noprompt")
        .arg("-script")
        .arg(&kernel.session_batch_wrapper_path)
        .arg(&root_manifest_path);
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

    let results = parse_session_batch_results(kernel, batches, &stdout, &stderr)?;
    build_session_batch_evaluations(&root_dir, results, &stdout, &stderr)
}

fn write_batch_inputs(
    root_dir: &Path,
    batches: &[Vec<String>],
) -> Result<Vec<std::path::PathBuf>, OracleError> {
    let mut batch_manifest_paths = Vec::with_capacity(batches.len());
    for (batch_index, batch) in batches.iter().enumerate() {
        let batch_dir = root_dir.join(format!("batch-{:04}", batch_index + 1));
        fs::create_dir(&batch_dir).map_err(|source| OracleError::Io {
            path: batch_dir.clone(),
            source,
        })?;

        let mut input_paths = Vec::with_capacity(batch.len());
        for (step_index, input) in batch.iter().enumerate() {
            let path = batch_dir.join(format!("step-{:04}.wl", step_index + 1));
            fs::write(&path, input).map_err(|source| OracleError::Io {
                path: path.clone(),
                source,
            })?;
            input_paths.push(path);
        }

        let manifest_path = batch_dir.join("manifest.txt");
        let manifest = input_paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&manifest_path, manifest).map_err(|source| OracleError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        batch_manifest_paths.push(manifest_path);
    }
    Ok(batch_manifest_paths)
}

fn parse_session_batch_results(
    kernel: &OracleKernel,
    batches: &[Vec<String>],
    stdout: &str,
    stderr: &str,
) -> Result<Vec<Vec<Option<String>>>, OracleError> {
    let mut results = batches
        .iter()
        .map(|batch| vec![None; batch.len()])
        .collect::<Vec<_>>();

    for line in stdout.lines() {
        if let Some(payload) = line.strip_prefix(SESSION_BATCH_RESULT_MARKER) {
            let Some((batch_text, rest)) = payload.split_once('\t') else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            let Some((step_text, value)) = rest.split_once('\t') else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            let Ok(batch_index) = batch_text.parse::<usize>() else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            let Ok(step_index) = step_text.parse::<usize>() else {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            };
            if batch_index == 0
                || batch_index > results.len()
                || step_index == 0
                || step_index > results[batch_index - 1].len()
            {
                return Err(invalid_session_marker(kernel, line, stdout, stderr));
            }
            results[batch_index - 1][step_index - 1] = Some(value.trim().to_owned());
        }
    }

    let expected = batches.iter().map(Vec::len).sum::<usize>();
    let actual = results
        .iter()
        .flatten()
        .filter(|result| result.is_some())
        .count();
    if actual != expected {
        return Err(OracleError::SessionResultCountMismatch {
            executable: kernel.executable.clone(),
            expected,
            actual,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        });
    }

    Ok(results)
}

fn build_session_batch_evaluations(
    root_dir: &Path,
    results: Vec<Vec<Option<String>>>,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
    let printed_output = collect_process_prints(stdout);
    let process_stderr = stderr.trim().to_owned();
    let mut batches = Vec::with_capacity(results.len());
    for (batch_index, batch_results) in results.into_iter().enumerate() {
        let batch_dir = root_dir.join(format!("batch-{:04}", batch_index + 1));
        let mut batch_evaluations = Vec::with_capacity(batch_results.len());
        for (step_index, result) in batch_results.into_iter().enumerate() {
            let suffix = format!("{:04}-{:04}", batch_index + 1, step_index + 1);
            let messages_path = batch_dir.join(format!("messages-{suffix}.txt"));
            let prints_path = batch_dir.join(format!("prints-{suffix}.txt"));
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

            batch_evaluations.push(OracleEvaluation {
                result: result.expect("batch result count was already validated"),
                printed_output: step_prints,
                stderr: step_messages,
            });
        }
        batches.push(batch_evaluations);
    }
    Ok(batches)
}

fn collect_process_prints(stdout: &str) -> String {
    stdout
        .lines()
        .filter(|line| !line.starts_with(SESSION_BATCH_RESULT_MARKER))
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
