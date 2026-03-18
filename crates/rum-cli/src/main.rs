use std::env;
use std::io;
use std::process::ExitCode;

use rum_repl::Repl;
use rum_runtime::{EvalError, Session};
use rum_stdlib::install_core;
use rum_syntax::{FormatterOptions, SourceText, format_parsed_source, parse_with_diagnostics};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::new();
    install_core(&mut session);

    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None => run_repl(&mut session)?,
        Some("--parse") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--parse expects an expression")
            })?;

            match parse_with_diagnostics(&input) {
                Ok(parsed) => print!("{}", parsed.debug_dump()),
                Err(diagnostic) => {
                    eprintln!("{}", diagnostic.render());
                    return Err(Box::new(io::Error::other("parse failed")));
                }
            }
        }
        Some("--format") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--format expects an expression",
                )
            })?;

            match parse_with_diagnostics(&input) {
                Ok(parsed) => {
                    let formatted = format_parsed_source(&parsed, FormatterOptions::default());
                    println!("{formatted}");
                }
                Err(diagnostic) => {
                    eprintln!("{}", diagnostic.render());
                    return Err(Box::new(io::Error::other("format failed")));
                }
            }
        }
        Some("--eval") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--eval expects an expression")
            })?;

            match session.eval_text(&input) {
                Ok(outcome) => print_outcome(&outcome),
                Err(EvalError::Syntax(err)) => {
                    eprintln!("{}", SourceText::new(input).render_syntax_error(&err));
                    return Err(Box::new(io::Error::other("evaluation failed")));
                }
                Err(err) => return Err(Box::new(err)),
            }
        }
        Some("--file") => {
            let path = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--file expects a path")
            })?;

            eval_file(&mut session, &path)?;
        }
        Some("--help") | Some("-h") => print_usage(),
        Some(path) => eval_file(&mut session, path)?,
    }

    Ok(())
}

fn run_repl(session: &mut Session) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut repl = Repl::new(stdin.lock(), stdout.lock()).with_default_history();
    repl.run(session)
}

fn eval_file(session: &mut Session, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match session.eval_file(path) {
        Ok(outcome) => {
            print_outcome(&outcome);
            Ok(())
        }
        Err(EvalError::Syntax(err)) => {
            let source = std::fs::read_to_string(path)?;
            eprintln!("{}", SourceText::new(source).render_syntax_error(&err));
            Err(Box::new(io::Error::other("file evaluation failed")))
        }
        Err(err) => Err(Box::new(err)),
    }
}

fn print_outcome(outcome: &rum_runtime::EvalOutcome) {
    for message in &outcome.messages {
        eprintln!("! {message}");
    }

    println!("{}", outcome.value);
}

fn print_usage() {
    println!("rum [--parse <expr>]");
    println!("rum [--format <expr>]");
    println!("rum [--eval <expr>]");
    println!("rum [--file <path>]");
    println!("rum [<path>]");
    println!("rum");
}
