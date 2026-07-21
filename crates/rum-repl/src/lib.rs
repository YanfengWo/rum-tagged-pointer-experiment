//! Interactive shell (REPL) layer for `rum`.
//!
//! Drives a read-eval-print loop over a [`rum_host`] session, including the
//! `:`-prefixed shell commands (`:help`, `:trace`, `:parse`, `:format`, and the
//! rest) and resumable input. Like the other front ends, it is a pure
//! `rum-host` consumer with no direct dependency on the kernel or standard
//! library.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use rum_host::{
    HostCompletenessStatus, HostEvalOutcome, HostFailure, HostInputSuspension, HostSession,
};
use rum_terminal::{render_host_eval_response, render_host_failure, render_host_messages};

mod commands;
use self::commands::*;

const HISTORY_ENTRY_PREFIX: &str = ":rum-history-v1:";

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

    pub fn run(&mut self, session: &mut HostSession) -> io::Result<()> {
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
                    let input = pending.to_string();
                    self.record_history(&input);
                    self.evaluate_input(session, &input)?;
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

            match session.is_complete(pending).status {
                HostCompletenessStatus::Complete | HostCompletenessStatus::Invalid => {
                    let input = pending.to_string();
                    buffer.clear();
                    self.record_history(&input);
                    self.evaluate_input(session, &input)?;
                }
                HostCompletenessStatus::Incomplete => continue,
            }
        }
    }

    fn evaluate_input(&mut self, session: &mut HostSession, input: &str) -> io::Result<()> {
        match session.eval(input) {
            HostEvalOutcome::Success(response) => {
                writeln!(self.output, "{}", render_host_eval_response(&response))
            }
            HostEvalOutcome::Failure(failure) => write_host_failure(&mut self.output, &failure),
            HostEvalOutcome::InputRequired(suspension) => {
                writeln!(self.output, "{}", render_input_suspension(&suspension))
            }
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
            .map(decode_history_line)
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

        let mut contents = self
            .history
            .iter()
            .map(|entry| encode_history_entry(entry))
            .collect::<Vec<_>>()
            .join("\n");

        if !contents.is_empty() {
            contents.push('\n');
        }

        fs::write(path, contents)
    }

    fn try_handle_command(
        &mut self,
        session: &mut HostSession,
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

fn encode_history_entry(entry: &str) -> String {
    if !history_entry_needs_encoding(entry) {
        return entry.to_string();
    }

    let mut encoded = String::with_capacity(HISTORY_ENTRY_PREFIX.len() + entry.len());
    encoded.push_str(HISTORY_ENTRY_PREFIX);

    for ch in entry.chars() {
        match ch {
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            other => encoded.push(other),
        }
    }

    encoded
}

fn history_entry_needs_encoding(entry: &str) -> bool {
    entry.starts_with(HISTORY_ENTRY_PREFIX) || entry.contains(['\n', '\r'])
}

fn decode_history_line(line: &str) -> String {
    let Some(encoded) = line.strip_prefix(HISTORY_ENTRY_PREFIX) else {
        return line.to_string();
    };

    let mut decoded = String::new();
    let mut chars = encoded.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        match chars.next() {
            Some('\\') => decoded.push('\\'),
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some(other) => {
                decoded.push('\\');
                decoded.push(other);
            }
            None => decoded.push('\\'),
        }
    }

    decoded
}

fn render_input_suspension(suspension: &HostInputSuspension) -> String {
    let mut output = render_host_messages(&suspension.messages);
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str("! input required");
    if !suspension.request.prompt.is_empty() {
        output.push_str(": ");
        output.push_str(&suspension.request.prompt);
    }
    output
}

fn write_host_failure<W: Write>(output: &mut W, failure: &HostFailure) -> io::Result<()> {
    writeln!(output, "{}", render_host_failure(failure))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
