use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_rum-cli")
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(cli_bin())
        .args(args)
        .output()
        .expect("failed to run rum-cli test binary")
}

fn run_cli_with_stdin(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(cli_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn rum-cli test binary");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("should write rum-cli stdin");

    child
        .wait_with_output()
        .expect("failed to wait for rum-cli test binary")
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
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "rum-cli-test-{}-{unique}-{counter}.rum",
            std::process::id()
        ));
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
fn format_flag_accepts_prefix_equals_line_width() {
    let output = run_cli(&[
        "--format",
        "--line-width=60",
        "f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]",
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
fn no_arg_non_tty_stdin_is_filter_program_without_repl_chatter() {
    let output = run_cli_with_stdin(&[], "Total[{1,2,3}]\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "6\n");
    assert!(!stdout(&output).contains("rum>"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn filter_mode_uses_script_semantics_and_prints_final_result_only() {
    let output = run_cli_with_stdin(&[], "x=1\nx+1\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "2\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn positional_dash_reads_stdin_as_program() {
    let output = run_cli_with_stdin(&["-"], "2 + 3\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "5\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn eval_flag_renders_source_aware_syntax_diagnostics() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let output = run_cli(&["--eval", "f[1,]"]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("error: expected expression"));
    assert!(stderr.contains("--> 1:5"));
    assert!(stderr.contains("error: evaluation failed"));
}

#[test]
fn eval_flag_renders_prints_before_final_value() {
    let output = run_cli(&["--eval", "Print[\"hello\"]; 1 + 1"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "hello\n2\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn eval_flag_renders_hold_form_transparently_in_canonical_style() {
    let direct = run_cli(&["--eval", "HoldForm[1 + 1]"]);
    let nested = run_cli(&["--eval", "{HoldForm[1 + 1]}"]);

    assert!(direct.status.success());
    assert_eq!(stdout(&direct), "Plus[1, 1]\n");
    assert!(stderr(&direct).is_empty());
    assert!(nested.status.success());
    assert_eq!(stdout(&nested), "List[Plus[1, 1]]\n");
    assert!(stderr(&nested).is_empty());
}

#[test]
fn eval_flag_keeps_non_hold_form_canonical_output_unchanged() {
    let output = run_cli(&["--eval", "{Hold[1 + 1], \"rum\"}"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "List[Hold[Plus[1, 1]], \"rum\"]\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn eval_flag_keeps_to_string_form_arms_unchanged() {
    let output = run_cli(&[
        "--eval",
        "{ToString[HoldForm[1 + 1], OutputForm], ToString[HoldForm[1 + 1], StandardForm], ToString[HoldForm[1 + 1], InputForm], ToString[HoldForm[1 + 1], TraditionalForm]}",
    ]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "List[\"1 + 1\", \"1 + 1\", \"HoldForm[1 + 1]\", \"1 + 1\"]\n"
    );
    assert!(stderr(&output).is_empty());
}

#[test]
fn eval_flag_binds_piped_stdin_as_script_input() {
    let output = run_cli_with_stdin(&["--eval", "StringLines[$ScriptInput]"], "a\nb\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "List[\"a\", \"b\"]\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn script_input_preserves_quotes_and_backslashes_verbatim() {
    let output = run_cli_with_stdin(&["--eval", "ToCharacterCode[$ScriptInput]"], "a\"\\b\n");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "List[97, 34, 92, 98, 10]\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn script_args_after_separator_are_bound_as_strings() {
    let output = run_cli(&["--eval", "$ScriptArgs", "--", "one", "two three"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "List[\"one\", \"two three\"]\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn script_args_separator_without_args_binds_empty_list() {
    let output = run_cli(&["--eval", "$ScriptArgs", "--"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "List[]\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn runtime_messages_go_to_stderr_and_values_go_to_stdout() {
    let output = run_cli(&["--eval", "Message[FindFile::notfound]; 42"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "42\n");
    assert!(stderr(&output).contains("! FindFile::notfound:"));
    assert!(!stdout(&output).contains("FindFile::notfound"));
}

#[test]
fn random_default_seed_is_reproducible_across_cli_invocations() {
    let source = "{RandomInteger[100], RandomInteger[100], RandomInteger[{-5, 5}, 4]}";
    let first = run_cli(&["--eval", source]);
    let second = run_cli(&["--eval", source]);

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(stdout(&first), stdout(&second));
    assert_eq!(stdout(&first), "List[52, 42, List[-5, 4, 5, 3]]\n");
    assert!(stderr(&first).is_empty());
    assert!(stderr(&second).is_empty());
}

#[test]
fn eval_flag_treats_replace_repeated_limit_as_recoverable_value() {
    let output = run_cli(&["--eval", "1 //. x_ :> x + 1"]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "TerminatedEvaluation[\"IterationLimit\"]\n"
    );
    assert!(stderr(&output).contains("! General::iterlim:"));
}

#[test]
fn file_flag_renders_source_aware_syntax_diagnostics() {
    // `f[1 2]` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let file = TempFile::new("f[1,]");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_cli(&["--file", &path]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("error: expected expression"));
    assert!(stderr.contains("--> 1:5"));
    assert!(stderr.contains("1 | f[1,]"));
    assert!(stderr.contains("error: file evaluation failed"));
}

#[test]
fn file_and_stdin_scripts_render_later_error_at_the_written_character() {
    let source = "a = 1\nb = 2\nc = 3\nd = 1 ! 2\n";
    let file = TempFile::new(source);
    let path = file.path().to_string_lossy().into_owned();
    let file_output = run_cli(&["--file", &path]);
    let stdin_output = run_cli_with_stdin(&[], source);

    for (path, output) in [("--file", file_output), ("stdin", stdin_output)] {
        assert!(!output.status.success(), "{path} should reject the script");
        let stderr = stderr(&output);
        assert!(
            stderr.contains("error: trailing input at bytes 24..25"),
            "{path}: {stderr}"
        );
        assert!(stderr.contains("--> 4:7"), "{path}: {stderr}");
        assert!(
            stderr.contains("4 | d = 1 ! 2\n  |       ^"),
            "{path}: {stderr}"
        );
    }
}

#[test]
fn eval_exit_uses_requested_status_code_without_error_rendering() {
    let output = run_cli(&["--eval", "Exit[7]"]);

    assert_eq!(output.status.code(), Some(7));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
}

#[test]
fn eval_exit_renders_accumulated_prints_without_error_rendering() {
    let output = run_cli(&["--eval", "Print[\"before exit\"]; Exit[7]"]);

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(stdout(&output), "before exit\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn file_exit_uses_requested_status_code_without_error_rendering() {
    let file = TempFile::new("Exit[6]");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_cli(&["--file", &path]);

    assert_eq!(output.status.code(), Some(6));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
}

#[test]
fn file_exit_renders_accumulated_prints_without_error_rendering() {
    let file = TempFile::new("Print[\"file progress\"]; Exit[6]");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_cli(&["--file", &path]);

    assert_eq!(output.status.code(), Some(6));
    assert_eq!(stdout(&output), "file progress\n");
    assert!(stderr(&output).is_empty());
}

#[test]
fn unknown_long_flag_is_rejected() {
    let output = run_cli(&["--typo"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("error: unknown flag `--typo`"));
}

#[test]
fn closed_stdout_pipe_exits_successfully_without_stderr() {
    let program = std::iter::repeat_n("Print[1]", 500)
        .collect::<Vec<_>>()
        .join("\n");
    let mut child = Command::new(cli_bin())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn rum-cli test binary");

    drop(child.stdout.take().expect("stdout should be piped"));

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(program.as_bytes());
    }

    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("stderr should be readable");
    let status = child.wait().expect("failed to wait for rum-cli");

    assert!(status.success(), "closed-pipe status was {status:?}");
    assert!(stderr.is_empty(), "closed-pipe stderr was {stderr:?}");
}
