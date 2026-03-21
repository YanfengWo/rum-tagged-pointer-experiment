use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use rum_runtime::{Session, SessionRequestError, SessionRequestFailure};
use rum_terminal::{render_eval_response, render_request_failure};

mod commands;
use self::commands::*;

pub struct Repl<R, W> {
    input: R,
    output: W,
    prompt: String,
    continuation_prompt: String,
    history: Vec<String>,
    history_path: Option<PathBuf>,
}

impl<R, W> Repl<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(input: R, output: W) -> Self {
        Self {
            input,
            output,
            prompt: "rum> ".to_string(),
            continuation_prompt: "...> ".to_string(),
            history: Vec::new(),
            history_path: None,
        }
    }

    pub fn with_default_history(mut self) -> Self {
        if let Some(path) = default_history_path() {
            self.history_path = Some(path);
            self.load_history();
        }

        self
    }

    pub fn with_history_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.history_path = Some(path.into());
        self.load_history();
        self
    }

    pub fn run(&mut self, session: &mut Session) -> io::Result<()> {
        let mut buffer = String::new();

        loop {
            let prompt = if buffer.trim().is_empty() {
                &self.prompt
            } else {
                &self.continuation_prompt
            };

            self.output.write_all(prompt.as_bytes())?;
            self.output.flush()?;

            let mut line = String::new();
            let bytes_read = self.input.read_line(&mut line)?;

            if bytes_read == 0 {
                writeln!(self.output)?;

                let pending = buffer.trim();

                if !pending.is_empty() {
                    if let Err(failure) = session.parse_request(pending) {
                        self.record_history(pending);
                        write_request_failure(&mut self.output, &failure)?;
                        return Ok(());
                    }

                    self.record_history(pending);
                    self.evaluate_input(session, pending)?;
                }

                return Ok(());
            }

            let line = line.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim();

            if buffer.trim().is_empty() && trimmed.is_empty() {
                continue;
            }

            if buffer.trim().is_empty() {
                if let Some(execution) = self.try_handle_command(session, trimmed)? {
                    if matches!(execution, CommandExecution::Exit) {
                        self.record_history(trimmed);
                        return Ok(());
                    }

                    self.record_history(trimmed);
                    continue;
                }
            }

            if !buffer.is_empty() {
                buffer.push('\n');
            }

            buffer.push_str(line);

            let pending = buffer.trim();

            match session.parse_request(pending) {
                Ok(_) => {
                    let input = pending.to_string();
                    buffer.clear();
                    self.record_history(&input);
                    self.evaluate_input(session, &input)?;
                }
                Err(failure) if request_failure_is_incomplete(&failure) => continue,
                Err(failure) => {
                    let input = pending.to_string();
                    buffer.clear();
                    self.record_history(&input);
                    write_request_failure(&mut self.output, &failure)?;
                }
            }
        }
    }

    fn evaluate_input(&mut self, session: &mut Session, input: &str) -> io::Result<()> {
        match session.eval_request(input) {
            Ok(response) => writeln!(self.output, "{}", render_eval_response(&response)),
            Err(failure) => write_request_failure(&mut self.output, &failure),
        }
    }

    fn load_history(&mut self) {
        let Some(path) = &self.history_path else {
            return;
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };

        self.history = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect();
    }

    fn record_history(&mut self, entry: &str) {
        let entry = entry.trim();

        if entry.is_empty() {
            return;
        }

        self.history.push(entry.to_string());
        let _ = self.persist_history();
    }

    fn persist_history(&self) -> io::Result<()> {
        let Some(path) = &self.history_path else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut contents = self.history.join("\n");

        if !contents.is_empty() {
            contents.push('\n');
        }

        fs::write(path, contents)
    }

    fn try_handle_command(
        &mut self,
        session: &mut Session,
        line: &str,
    ) -> io::Result<Option<CommandExecution>> {
        let Some(command) = (match parse_repl_command(line) {
            Ok(command) => command,
            Err(message) => {
                writeln!(self.output, "! {message}")?;
                return Ok(Some(CommandExecution::Output(String::new())));
            }
        }) else {
            return Ok(None);
        };

        let execution = execute_repl_command(session, &self.history, command);

        match &execution {
            CommandExecution::Exit => {}
            CommandExecution::Output(rendered) if rendered.is_empty() => {}
            CommandExecution::Output(rendered) => {
                writeln!(self.output, "{rendered}")?;
            }
        }

        Ok(Some(execution))
    }
}

fn default_history_path() -> Option<PathBuf> {
    env::var_os("RUM_HISTORY_PATH")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok().map(|dir| dir.join(".rum_history")))
}

fn request_failure_is_incomplete(failure: &SessionRequestFailure) -> bool {
    matches!(
        &failure.error,
        SessionRequestError::Syntax(diagnostic) if diagnostic.is_incomplete
    )
}

fn write_request_failure<W: Write>(
    output: &mut W,
    failure: &SessionRequestFailure,
) -> io::Result<()> {
    writeln!(output, "{}", render_request_failure(failure))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
