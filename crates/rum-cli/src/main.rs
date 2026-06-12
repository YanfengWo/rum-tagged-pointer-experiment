use std::env;
use std::io;
use std::process::ExitCode;

use mimalloc::MiMalloc;
use rum_repl::Repl;
use rum_runtime::{Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure};
use rum_stdlib::install_core;
use rum_syntax::{format_parsed_source, parse_format_cli_args};
use rum_terminal::{render_request_failure, render_runtime_message};

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
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

fn print_outcome(outcome: &SessionEvalResponse) {
    for message in &outcome.messages {
        eprintln!("{}", render_runtime_message(message));
    }

    for printed in &outcome.prints {
        println!("{printed}");
    }

    println!("{}", outcome.value);
}

fn print_failure_side_effects(failure: &SessionRequestFailure) {
    for message in &failure.messages {
        eprintln!("{}", render_runtime_message(message));
    }

    for printed in &failure.prints {
        println!("{printed}");
    }
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
