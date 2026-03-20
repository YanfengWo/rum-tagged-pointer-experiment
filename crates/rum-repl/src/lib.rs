use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use rum_runtime::{Session, SessionRequestError, SessionRequestFailure};
use rum_syntax::{format_parsed_source, parse_format_command};
use rum_terminal::{
    render_eval_response, render_inspect_response, render_message_catalog_entries,
    render_request_failure,
};

pub struct Repl<R, W> {
    input: R,
    output: W,
    prompt: String,
    continuation_prompt: String,
    history: Vec<String>,
    history_path: Option<PathBuf>,
}

enum CommandAction {
    NotHandled,
    Handled,
    Exit,
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
                match self.try_handle_command(session, trimmed)? {
                    CommandAction::Exit => {
                        self.record_history(trimmed);
                        return Ok(());
                    }
                    CommandAction::Handled => {
                        self.record_history(trimmed);
                        continue;
                    }
                    CommandAction::NotHandled => {}
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
    ) -> io::Result<CommandAction> {
        match line {
            ":quit" | ":q" => Ok(CommandAction::Exit),
            ":help" => {
                writeln!(self.output, ":help           Show REPL commands.")?;
                writeln!(
                    self.output,
                    ":parse EXPR     Parse and print the syntax tree."
                )?;
                writeln!(
                    self.output,
                    ":format [--line-width N] EXPR  Format syntax into the current canonical style."
                )?;
                writeln!(
                    self.output,
                    ":lower EXPR     Lower syntax into core Expr form."
                )?;
                writeln!(
                    self.output,
                    ":complete PREF  Show symbol and context completions."
                )?;
                writeln!(self.output, ":history [N]    Show recent input history.")?;
                writeln!(self.output, ":inspect NAME   Show basic symbol metadata.")?;
                writeln!(
                    self.output,
                    ":messages [NAME]   Show message definitions globally or for a symbol/message."
                )?;
                writeln!(self.output, ":quit           Exit the REPL.")?;
                Ok(CommandAction::Handled)
            }
            _ if line == ":parse" || line.starts_with(":parse ") => {
                let input = line.trim_start_matches(":parse").trim();

                if input.is_empty() {
                    writeln!(self.output, "! :parse expects an expression")?;
                } else {
                    match session.parse_request(input) {
                        Ok(response) => write!(self.output, "{}", response.parsed.debug_dump())?,
                        Err(failure) => write_request_failure(&mut self.output, &failure)?,
                    }
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":format" || line.starts_with(":format ") => {
                let input = line.trim_start_matches(":format").trim();

                match parse_format_command(input, ":format") {
                    Err(message) => writeln!(self.output, "! {message}")?,
                    Ok((expr, options)) => match session.parse_request(expr) {
                        Ok(response) => {
                            let formatted = format_parsed_source(&response.parsed, options);
                            writeln!(self.output, "{formatted}")?;
                        }
                        Err(failure) => write_request_failure(&mut self.output, &failure)?,
                    },
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":lower" || line.starts_with(":lower ") => {
                let input = line.trim_start_matches(":lower").trim();

                if input.is_empty() {
                    writeln!(self.output, "! :lower expects an expression")?;
                } else {
                    match session.lower_request(input) {
                        Ok(response) => writeln!(self.output, "{}", response.expr)?,
                        Err(failure) => write_request_failure(&mut self.output, &failure)?,
                    }
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":history" || line.starts_with(":history ") => {
                let raw_limit = line
                    .strip_prefix(":history")
                    .map(str::trim)
                    .filter(|raw| !raw.is_empty());

                let limit = match raw_limit {
                    Some(raw) => match raw.parse::<usize>() {
                        Ok(limit) => Some(limit),
                        Err(_) => {
                            writeln!(self.output, "! invalid history count `{raw}`")?;
                            return Ok(CommandAction::Handled);
                        }
                    },
                    None => None,
                };

                if self.history.is_empty() {
                    writeln!(self.output, "(no history)")?;
                    return Ok(CommandAction::Handled);
                }

                let limit = limit.unwrap_or(self.history.len());
                let start = self.history.len().saturating_sub(limit);

                for (index, entry) in self.history[start..].iter().enumerate() {
                    writeln!(
                        self.output,
                        "{:>4}: {}",
                        start + index + 1,
                        entry.replace('\n', "\\n")
                    )?;
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":complete" || line.starts_with(":complete ") => {
                let prefix = line.trim_start_matches(":complete").trim();

                if prefix.is_empty() {
                    writeln!(self.output, "! :complete expects a prefix")?;
                    return Ok(CommandAction::Handled);
                }

                let completions = session.complete_symbol_prefix(prefix);

                if completions.is_empty() {
                    writeln!(self.output, "(no completions)")?;
                } else {
                    for completion in completions {
                        writeln!(self.output, "{completion}")?;
                    }
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":inspect" => {
                writeln!(self.output, "! :inspect expects a symbol name")?;
                Ok(CommandAction::Handled)
            }
            _ if line.starts_with(":inspect ") => {
                let name = line.trim_start_matches(":inspect ").trim();

                if let Some(response) = session.inspect_request(name) {
                    writeln!(self.output, "{}", render_inspect_response(&response))?;
                } else {
                    writeln!(self.output, "! unknown symbol `{name}`")?;
                }

                Ok(CommandAction::Handled)
            }
            _ if line == ":messages" || line.starts_with(":messages ") => {
                let query = line
                    .strip_prefix(":messages")
                    .map(str::trim)
                    .filter(|query| !query.is_empty());

                let Some(entries) = session.inspect_message_catalog(query) else {
                    let name = query.unwrap_or("<invalid>");
                    writeln!(self.output, "! invalid message selector `{name}`")?;
                    return Ok(CommandAction::Handled);
                };

                if entries.is_empty() {
                    if let Some(name) = query {
                        writeln!(self.output, "! no messages for `{name}`")?;
                    } else {
                        writeln!(self.output, "(no message definitions)")?;
                    }
                    return Ok(CommandAction::Handled);
                }

                writeln!(self.output, "{}", render_message_catalog_entries(&entries))?;

                Ok(CommandAction::Handled)
            }
            _ => Ok(CommandAction::NotHandled),
        }
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
