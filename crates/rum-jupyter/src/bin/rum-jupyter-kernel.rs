use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

use rum_jupyter::run_native_kernel;

fn main() -> ExitCode {
    match parse_connection_path(env::args_os()) {
        Ok(connection_path) => match run_native_kernel(connection_path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Err(ParseCliError::Help(program_name)) => {
            print_usage(&program_name);
            ExitCode::SUCCESS
        }
        Err(ParseCliError::Usage {
            program_name,
            message,
        }) => {
            eprintln!("{message}");
            print_usage(&program_name);
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
enum ParseCliError {
    Help(String),
    Usage {
        program_name: String,
        message: String,
    },
}

fn parse_connection_path(
    mut args: impl Iterator<Item = OsString>,
) -> Result<PathBuf, ParseCliError> {
    let program_name = args
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "rum-jupyter-kernel".to_string());
    let Some(first) = args.next() else {
        return Err(ParseCliError::Usage {
            program_name,
            message: "missing required <connection-file> argument".to_string(),
        });
    };

    if first == "-h" || first == "--help" {
        return Err(ParseCliError::Help(program_name));
    }

    if args.next().is_some() {
        return Err(ParseCliError::Usage {
            program_name,
            message: "expected exactly one <connection-file> argument".to_string(),
        });
    }

    Ok(PathBuf::from(first))
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {program_name} <connection-file>");
    eprintln!();
    eprintln!("Start the rum Jupyter kernel against the given Jupyter connection file.");
}
