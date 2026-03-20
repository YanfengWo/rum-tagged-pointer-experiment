use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_rum-cli")
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(cli_bin())
        .args(args)
        .output()
        .expect("failed to run rum-cli test binary")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be valid utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be valid utf-8")
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(contents: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rum-cli-test-{unique}.rum"));
        fs::write(&path, contents).expect("should write temporary test file");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn parse_flag_prints_debug_dump() {
    let output = run_cli(&["--parse", "f[1]"]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("syntax:"));
    assert!(stdout(&output).contains("tokens:"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn format_flag_prints_canonical_surface_form() {
    let output = run_cli(&["--format", "(* note *) f [ x_Integer , y ]"]);

    assert!(output.status.success());
    let stdout = stdout(&output);
    assert!(stdout.contains("(* note *)"));
    assert!(stdout.contains("f[x_Integer, y]"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn format_flag_respects_custom_line_width() {
    let output = run_cli(&[
        "--format",
        "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]",
        "--line-width",
        "60",
    ]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output).trim(),
        "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]"
    );
    assert!(stderr(&output).is_empty());
}

#[test]
fn format_flag_rejects_invalid_line_width() {
    let output = run_cli(&["--format", "f[x]", "--line-width", "0"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("error: --line-width expects a positive integer"));
}

#[test]
fn eval_flag_renders_source_aware_syntax_diagnostics() {
    let output = run_cli(&["--eval", "f[1 2]"]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("error: expected ,"));
    assert!(stderr.contains("--> 1:5"));
    assert!(stderr.contains("error: evaluation failed"));
}

#[test]
fn file_flag_renders_source_aware_syntax_diagnostics() {
    let file = TempFile::new("f[1 2]");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_cli(&["--file", &path]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("error: expected ,"));
    assert!(stderr.contains("1 | f[1 2]"));
    assert!(stderr.contains("error: file evaluation failed"));
}

#[test]
fn unknown_long_flag_is_rejected() {
    let output = run_cli(&["--typo"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("error: unknown flag `--typo`"));
}
