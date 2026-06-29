use std::env;
use std::io;
use std::process::ExitCode;

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
    install_core(&mut session);

    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None => run_repl(&mut session)?,
        Some("--parse") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--parse expects an expression")
            })?;

            match session.parse_request(&input) {
                Ok(response) => print!("{}", response.parsed.debug_dump()),
                Err(failure) => return Err(request_failure_to_cli_error(failure, "parse failed")),
            }
        }
        Some("--format") => {
            let raw_args = args.collect::<Vec<_>>();
            let (input, options) = parse_format_cli_args(&raw_args, "--format")
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;

            match session.parse_request(&input) {
                Ok(response) => {
                    let formatted = format_parsed_source(&response.parsed, options);
                    println!("{formatted}");
                }
                Err(failure) => return Err(request_failure_to_cli_error(failure, "format failed")),
            }
        }
        Some("--eval") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--eval expects an expression")
            })?;

            install_stdout_print_sink(&mut session);
            match session.eval_request(&input) {
                Ok(response) => print_outcome(&response),
                Err(failure) => {
                    if let Some(code) = exit_code_from_failure(&failure) {
                        print_failure_side_effects(&failure);
                        return Ok(code);
                    }
                    return Err(request_failure_to_cli_error(failure, "evaluation failed"));
                }
            }
        }
        Some("--file") => {
            let path = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--file expects a path")
            })?;

            return eval_file(&mut session, &path);
        }
        Some("--help") | Some("-h") => print_usage(),
        Some(flag) if flag.starts_with("--") => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown flag `{flag}`"),
            )));
        }
        Some(path) => return eval_file(&mut session, path),
    }

    Ok(ExitCode::SUCCESS)
}

fn run_repl(session: &mut Session) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut repl = Repl::new(stdin.lock(), stdout.lock()).with_default_history();
    repl.run(session)
}

fn eval_file(session: &mut Session, path: &str) -> Result<ExitCode, Box<dyn std::error::Error>> {
    install_stdout_print_sink(session);
    match session.eval_file_request(path) {
        Ok(response) => {
            print_outcome(&response);
            Ok(ExitCode::SUCCESS)
        }
        Err(failure) => {
            if let Some(code) = exit_code_from_failure(&failure) {
                print_failure_side_effects(&failure);
                return Ok(code);
            }

            Err(request_failure_to_cli_error(
                failure,
                "file evaluation failed",
            ))
        }
    }
}

fn install_stdout_print_sink(session: &mut Session) {
    session.set_print_sink(Box::new(|text| {
        use std::io::Write;

        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{text}");
        let _ = out.flush();
    }));
}

fn print_outcome(outcome: &SessionEvalResponse) {
    for message in &outcome.messages {
        eprintln!("{}", render_runtime_message(message));
    }

    println!("{}", outcome.value);
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

fn print_usage() {
    println!("rum [--parse <expr>]");
    println!("rum [--format <expr> [--line-width <columns>]]");
    println!("rum [--eval <expr>]");
    println!("rum [--file <path>]");
    println!("rum [--help]");
    println!("rum [<path>]");
    println!("rum");
}
