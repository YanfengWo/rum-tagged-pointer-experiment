use std::fs;
use std::process::Command;

use super::super::process::run_with_timeout;
use super::OracleKernel;
use crate::oracle::{OracleError, OracleEvaluation};

const RESULT_MARKER: &str = "__RUM_DIFFTEST_RESULT__=";

pub(super) fn evaluate(
    kernel: &mut OracleKernel,
    input: &str,
) -> Result<OracleEvaluation, OracleError> {
    kernel.next_case_id += 1;
    let input_path = kernel
        .temp_dir
        .path()
        .join(format!("case-{:04}.wl", kernel.next_case_id));
    fs::write(&input_path, input).map_err(|source| OracleError::Io {
        path: input_path.clone(),
        source,
    })?;

    let mut command = Command::new(&kernel.executable);
    command
        .arg("-noprompt")
        .arg("-script")
        .arg(&kernel.wrapper_path)
        .arg(&input_path);
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

    let mut result = None;
    let mut printed_lines = Vec::new();

    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix(RESULT_MARKER) {
            result = Some(value.trim().to_owned());
        } else {
            printed_lines.push(line.to_owned());
        }
    }

    let Some(result) = result else {
        return Err(OracleError::MissingResultMarker {
            executable: kernel.executable.clone(),
            stdout,
            stderr,
        });
    };

    Ok(OracleEvaluation {
        result,
        printed_output: printed_lines.join("\n").trim().to_owned(),
        stderr: stderr.trim().to_owned(),
    })
}
