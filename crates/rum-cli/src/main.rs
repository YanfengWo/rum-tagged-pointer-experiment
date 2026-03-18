use std::env;
use std::io;
use std::process::ExitCode;

use rum_repl::Repl;
use rum_runtime::Session;
use rum_stdlib::install_core;

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
        Some("--eval") => {
            let input = args.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "--eval expects an expression")
            })?;

            let outcome = session.eval_text(&input)?;
            print_outcome(&outcome);
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
    let outcome = session.eval_file(path)?;
    print_outcome(&outcome);
    Ok(())
}

fn print_outcome(outcome: &rum_runtime::EvalOutcome) {
    for message in &outcome.messages {
        eprintln!("! {message}");
    }

    println!("{}", outcome.value);
}

fn print_usage() {
    println!("rum [--eval <expr>]");
    println!("rum [--file <path>]");
    println!("rum [<path>]");
    println!("rum");
}
