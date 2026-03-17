use std::io::{self, BufRead, Write};

use rum_runtime::Session;

pub struct Repl<R, W> {
    input: R,
    output: W,
    prompt: String,
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
        }
    }

    pub fn run(&mut self, session: &mut Session) -> io::Result<()> {
        loop {
            self.output.write_all(self.prompt.as_bytes())?;
            self.output.flush()?;

            let mut line = String::new();
            let bytes_read = self.input.read_line(&mut line)?;

            if bytes_read == 0 {
                writeln!(self.output)?;
                return Ok(());
            }

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            match self.try_handle_command(session, line)? {
                CommandAction::Exit => return Ok(()),
                CommandAction::Handled => continue,
                CommandAction::NotHandled => {}
            }

            match session.eval_text(line) {
                Ok(outcome) => {
                    for message in outcome.messages {
                        writeln!(self.output, "! {message}")?;
                    }

                    writeln!(self.output, "{}", outcome.value)?
                }
                Err(err) => {
                    for message in session.take_messages() {
                        writeln!(self.output, "! {message}")?;
                    }

                    writeln!(self.output, "! {err}")?
                }
            }
        }
    }

    fn try_handle_command(&mut self, session: &Session, line: &str) -> io::Result<CommandAction> {
        match line {
            ":quit" | ":q" => Ok(CommandAction::Exit),
            ":help" => {
                writeln!(self.output, ":help           Show REPL commands.")?;
                writeln!(self.output, ":inspect NAME   Show basic symbol metadata.")?;
                writeln!(self.output, ":quit           Exit the REPL.")?;
                Ok(CommandAction::Handled)
            }
            _ if line.starts_with(":inspect ") => {
                let name = line.trim_start_matches(":inspect ").trim();

                if let Some(info) = session.inspect(name) {
                    let mut flags = Vec::new();

                    if info.is_builtin {
                        flags.push("builtin");
                    }

                    if info.has_own_value {
                        flags.push("own");
                    }

                    if info.has_delayed_value {
                        flags.push("delayed");
                    }

                    if info.has_down_values {
                        flags.push("down");
                    }

                    writeln!(
                        self.output,
                        "{} :: {}{}",
                        info.qualified_name,
                        info.summary.unwrap_or_default(),
                        if flags.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", flags.join(", "))
                        }
                    )?;
                } else {
                    writeln!(self.output, "! unknown symbol `{name}`")?;
                }

                Ok(CommandAction::Handled)
            }
            _ => Ok(CommandAction::NotHandled),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rum_runtime::Session;

    use super::Repl;

    #[test]
    fn repl_handles_help_and_quit() {
        let input = Cursor::new(":help\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains(":inspect NAME"));
    }

    #[test]
    fn repl_inspect_command_does_not_fall_through_to_evaluation() {
        let input = Cursor::new(":inspect x\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();
        session.eval_text("x = 1").unwrap();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("x ::"));
        assert!(!rendered.contains("unexpected character"));
    }

    #[test]
    fn repl_renders_runtime_messages_before_results() {
        let input = Cursor::new("FindFile[\"Missing`\"]\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("! FindFile::notfound:"));
        assert!(rendered.contains("$Failed"));
    }
}
