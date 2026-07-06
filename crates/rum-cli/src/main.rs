use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rum_host::{StdEntropySource, StdFileIo};
use rum_repl::Repl;
use rum_runtime::{Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure};
use rum_stdlib::install_core;
use rum_syntax::{format_parsed_source, parse_format_cli_args};
use rum_terminal::{render_request_failure, render_runtime_message};

// The allocation-counter shim (Layer B B0 measurement) is compiled in ONLY under
// the `alloc-count` feature. The default build uses bare mimalloc with no wrapper
// so there is zero per-allocation cost on the production path (a global-allocator
// wrapper regresses every allocation 4-19% even when its flag is off, because it
// blocks the allocator fast-path inlining — measured 2026-06-22).
#[cfg(not(feature = "alloc-count"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "alloc-count")]
mod alloc_count;

fn main() -> ExitCode {
    #[cfg(feature = "alloc-count")]
    alloc_count::init();

    let code = match run() {
        Ok(code) => code,
        Err(err) if is_broken_pipe_error(err.as_ref()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    };

    #[cfg(feature = "alloc-count")]
    print_alloc_profile();
    code
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut session = Session::new();
    session.set_file_io(Some(StdFileIo::provider()));
    session.set_entropy_source(Some(StdEntropySource::provider()));
    install_core(&mut session);

    let invocation = parse_invocation(env::args().skip(1).collect())?;
    let stdin_is_tty = io::stdin().is_terminal();

    match invocation.command {
        CommandMode::ReplOrFilter if stdin_is_tty => run_repl(&mut session)?,
        CommandMode::ReplOrFilter => {
            bind_script_args_if_present(&mut session, invocation.script_args)?;
            let input = read_stdin_to_string()?;
            return eval_script_source(&mut session, &input);
        }
        CommandMode::StdinProgram => {
            bind_script_args_if_present(&mut session, invocation.script_args)?;
            let input = read_stdin_to_string()?;
            return eval_script_source(&mut session, &input);
        }
        CommandMode::Parse(input) => match session.parse_request(&input) {
            Ok(response) => write_stdout(response.parsed.debug_dump().as_bytes())?,
            Err(failure) => return Err(request_failure_to_cli_error(failure, "parse failed")),
        },
        CommandMode::Format(raw_args) => {
            let (input, options) = parse_format_cli_args(&raw_args, "--format")
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;

            match session.parse_request(&input) {
                Ok(response) => {
                    let formatted = format_parsed_source(&response.parsed, options);
                    write_stdout_line(&formatted)?;
                }
                Err(failure) => return Err(request_failure_to_cli_error(failure, "format failed")),
            }
        }
        CommandMode::Eval(input) => {
            if !stdin_is_tty {
                bind_script_input(&mut session, read_stdin_to_string()?)?;
            }
            bind_script_args_if_present(&mut session, invocation.script_args)?;
            return eval_source(&mut session, &input);
        }
        CommandMode::File(path) => {
            if !stdin_is_tty {
                bind_script_input(&mut session, read_stdin_to_string()?)?;
            }
            bind_script_args_if_present(&mut session, invocation.script_args)?;
            return eval_file(&mut session, &path);
        }
        CommandMode::Help => print_usage()?,
    }

    Ok(ExitCode::SUCCESS)
}

struct Invocation {
    command: CommandMode,
    script_args: Option<Vec<String>>,
}

enum CommandMode {
    ReplOrFilter,
    StdinProgram,
    Parse(String),
    Format(Vec<String>),
    Eval(String),
    File(String),
    Help,
}

fn parse_invocation(raw_args: Vec<String>) -> Result<Invocation, Box<dyn std::error::Error>> {
    match raw_args.first().map(String::as_str) {
        Some("--parse") => {
            let input = raw_args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--parse expects an expression")
            })?;
            Ok(Invocation {
                command: CommandMode::Parse(input.clone()),
                script_args: None,
            })
        }
        Some("--format") => Ok(Invocation {
            command: CommandMode::Format(raw_args.into_iter().skip(1).collect()),
            script_args: None,
        }),
        _ => parse_program_invocation(raw_args),
    }
}

fn parse_program_invocation(
    raw_args: Vec<String>,
) -> Result<Invocation, Box<dyn std::error::Error>> {
    let (args, script_args) = split_script_args(raw_args);
    let command = match args.first().map(String::as_str) {
        None => CommandMode::ReplOrFilter,
        Some("--eval") => {
            let input = args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--eval expects an expression")
            })?;
            CommandMode::Eval(input.clone())
        }
        Some("--file") => {
            let path = args.get(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--file expects a path")
            })?;
            CommandMode::File(path.clone())
        }
        Some("--help") | Some("-h") => CommandMode::Help,
        Some(flag) if flag.starts_with("--") => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown flag `{flag}`"),
            )));
        }
        Some("-") => CommandMode::StdinProgram,
        Some(path) => CommandMode::File(path.to_string()),
    };

    Ok(Invocation {
        command,
        script_args,
    })
}

fn split_script_args(raw_args: Vec<String>) -> (Vec<String>, Option<Vec<String>>) {
    let Some(separator) = raw_args.iter().position(|arg| arg == "--") else {
        return (raw_args, None);
    };

    let mut args = raw_args;
    let script_args = args.split_off(separator + 1);
    args.pop();
    (args, Some(script_args))
}

fn run_repl(session: &mut Session) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut repl = Repl::new(stdin.lock(), stdout.lock()).with_default_history();
    repl.run(session)
}

fn eval_source(session: &mut Session, input: &str) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(session, "evaluation failed", |session| {
        session.eval_request(input)
    })
}

fn eval_script_source(
    session: &mut Session,
    input: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(session, "evaluation failed", |session| {
        session.eval_script_request(input)
    })
}

fn eval_file(session: &mut Session, path: &str) -> Result<ExitCode, Box<dyn std::error::Error>> {
    eval_request(session, "file evaluation failed", |session| {
        session.eval_file_request(path)
    })
}

fn eval_request(
    session: &mut Session,
    error_context: &str,
    request: impl FnOnce(&mut Session) -> Result<SessionEvalResponse, SessionRequestFailure>,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let broken_pipe = install_stdout_print_sink(session);

    match request(session) {
        Ok(response) => {
            if broken_pipe.load(Ordering::Relaxed) {
                return Ok(ExitCode::SUCCESS);
            }
            print_outcome(&response)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(failure) => {
            if broken_pipe.load(Ordering::Relaxed) {
                return Ok(ExitCode::SUCCESS);
            }
            if let Some(code) = exit_code_from_failure(&failure) {
                print_failure_side_effects(&failure);
                return Ok(code);
            }

            Err(request_failure_to_cli_error(failure, error_context))
        }
    }
}

fn bind_script_input(
    session: &mut Session,
    input: String,
) -> Result<(), Box<dyn std::error::Error>> {
    session.queue_input_response(input);
    eval_setup_binding(session, "Global`$ScriptInput = InputString[]")
}

fn bind_script_args_if_present(
    session: &mut Session,
    args: Option<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(args) = args else {
        return Ok(());
    };

    for arg in args.iter().cloned() {
        session.queue_input_response(arg);
    }

    let placeholders = std::iter::repeat_n("InputString[]", args.len())
        .collect::<Vec<_>>()
        .join(", ");
    eval_setup_binding(session, &format!("Global`$ScriptArgs = {{{placeholders}}}"))
}

fn eval_setup_binding(
    session: &mut Session,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    session.eval_request(source).map(|_| ()).map_err(|failure| {
        io::Error::other(format!(
            "failed to bind script variable: {}",
            render_request_failure(&failure)
        ))
        .into()
    })
}

fn read_stdin_to_string() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn install_stdout_print_sink(session: &mut Session) -> Arc<AtomicBool> {
    let broken_pipe = Arc::new(AtomicBool::new(false));
    let sink_broken_pipe = Arc::clone(&broken_pipe);
    let interrupt = session.interrupt_handle();

    session.set_print_sink(Box::new(move |text| {
        if sink_broken_pipe.load(Ordering::Relaxed) {
            return;
        }

        if let Err(err) = write_stdout_line(text) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                sink_broken_pipe.store(true, Ordering::Relaxed);
                interrupt.store(true, Ordering::Relaxed);
            }
        }
    }));

    broken_pipe
}

fn print_outcome(outcome: &SessionEvalResponse) -> io::Result<()> {
    for message in &outcome.messages {
        eprintln!("{}", render_runtime_message(message));
    }

    write_stdout_line(&outcome.value.to_string())
}

fn print_failure_side_effects(failure: &SessionRequestFailure) {
    for message in &failure.messages {
        eprintln!("{}", render_runtime_message(message));
    }
}

#[cfg(feature = "alloc-count")]
fn print_alloc_profile() {
    if !alloc_count::enabled() {
        return;
    }

    let profile = alloc_count::snapshot();
    eprintln!(
        "rum-profile alloc count={} bytes_total={} peak_live_bytes={}",
        profile.count, profile.bytes_total, profile.peak_live_bytes
    );
}

fn request_failure_to_cli_error(
    failure: SessionRequestFailure,
    syntax_context: &str,
) -> Box<dyn std::error::Error> {
    let rendered = render_request_failure(&failure);

    match &failure.error {
        SessionRequestError::Syntax(_) => {
            eprintln!("{rendered}");
            Box::new(io::Error::other(syntax_context.to_string()))
        }
        error => {
            eprintln!("{rendered}");
            Box::new(io::Error::other(error.to_string()))
        }
    }
}

fn exit_code_from_failure(failure: &SessionRequestFailure) -> Option<ExitCode> {
    match &failure.error {
        SessionRequestError::Exit { code } => Some(ExitCode::from(*code as u8)),
        _ => None,
    }
}

fn write_stdout(bytes: &[u8]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

fn write_stdout_line(line: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "{line}")?;
    out.flush()
}

fn is_broken_pipe_error(err: &(dyn std::error::Error + 'static)) -> bool {
    err.downcast_ref::<io::Error>()
        .is_some_and(|err| err.kind() == io::ErrorKind::BrokenPipe)
}

fn print_usage() -> io::Result<()> {
    write_stdout(
        b"rum [--parse <expr>]\n\
rum [--format <expr> [--line-width <columns>]]\n\
rum [--eval <expr> [-- <arg>...]]\n\
rum [--file <path> [-- <arg>...]]\n\
rum [- [-- <arg>...]]\n\
rum [--help]\n\
rum [<path> [-- <arg>...]]\n\
rum\n\n\
With non-TTY stdin and no program argument, stdin is evaluated as a script.\n\
With a program argument and non-TTY stdin, stdin is bound as Global`$ScriptInput.\n\
Arguments after -- are bound as Global`$ScriptArgs.\n\
Exit codes: 0 on success, requested Exit[n] status for Exit, 1 on syntax/load/evaluation failure.\n",
    )
}
