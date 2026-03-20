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
            let (input, options) = parse_format_args(args.collect())?;

            match parse_with_diagnostics(&input) {
                Ok(parsed) => {
                    let formatted = format_parsed_source(&parsed, options);
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
        Some(flag) if flag.starts_with("--") => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown flag `{flag}`"),
            )));
        }
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
    println!("rum [--format <expr> [--line-width <columns>]]");
    println!("rum [--eval <expr>]");
    println!("rum [--file <path>]");
    println!("rum [--help]");
    println!("rum [<path>]");
    println!("rum");
}

fn parse_format_args(args: Vec<String>) -> io::Result<(String, FormatterOptions)> {
    let mut options = FormatterOptions::default();
    let mut input = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if let Some(raw_width) = arg.strip_prefix("--line-width=") {
            options.line_width = parse_line_width(raw_width, "--line-width")?;
            index += 1;
            continue;
        }

        match arg.as_str() {
            "--line-width" => {
                let raw_width = args.get(index + 1).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--line-width expects a positive integer",
                    )
                })?;
                options.line_width = parse_line_width(raw_width, "--line-width")?;
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown flag `{flag}`"),
                ));
            }
            value => {
                if input.replace(value.to_owned()).is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unexpected extra argument `{value}`"),
                    ));
                }
                index += 1;
            }
        }
    }

    let input = input.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "--format expects an expression",
        )
    })?;

    Ok((input, options))
}

fn parse_line_width(raw: &str, flag: &str) -> io::Result<usize> {
    match raw.parse::<usize>() {
        Ok(0) | Err(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{flag} expects a positive integer"),
        )),
        Ok(width) => Ok(width),
    }
}
