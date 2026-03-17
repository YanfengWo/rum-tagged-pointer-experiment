use std::io::{self, BufRead, Write};

use rum_runtime::{DefinitionCatalogEntry, DefinitionKind, Session};

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
                writeln!(
                    self.output,
                    ":messages [NAME]   Show message definitions globally or for a symbol/message."
                )?;
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

                    if info.message_count > 0 {
                        flags.push("msg");
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

                    if let Some(entries) = session.inspect_definition_catalog(name) {
                        let own_entries = entries
                            .iter()
                            .filter(|entry| {
                                matches!(
                                    entry.kind,
                                    DefinitionKind::OwnValue | DefinitionKind::DelayedOwnValue
                                )
                            })
                            .collect::<Vec<_>>();
                        let down_entries = entries
                            .iter()
                            .filter(|entry| entry.kind == DefinitionKind::DownValue)
                            .collect::<Vec<_>>();

                        if !own_entries.is_empty() {
                            writeln!(
                                self.output,
                                "  own: {}",
                                own_entries
                                    .iter()
                                    .map(|entry| format_definition_entry(entry))
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            )?;
                        }

                        if !down_entries.is_empty() {
                            writeln!(
                                self.output,
                                "  down[{}]: {}",
                                down_entries.len(),
                                down_entries
                                    .iter()
                                    .map(|entry| format_definition_entry(entry))
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            )?;
                        }
                    }
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

                for entry in entries {
                    let operator = if entry.delayed { ":=" } else { "=" };
                    writeln!(
                        self.output,
                        "{} {} {}",
                        entry.message_name, operator, entry.template
                    )?;
                }

                Ok(CommandAction::Handled)
            }
            _ => Ok(CommandAction::NotHandled),
        }
    }
}

fn format_definition_entry(entry: &DefinitionCatalogEntry) -> String {
    format!(
        "{} {} {}",
        entry.lhs,
        if entry.delayed { ":=" } else { "=" },
        entry.rhs
    )
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
    fn repl_inspect_surfaces_message_definition_flags() {
        let input = Cursor::new(":inspect f\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();
        session.eval_text("f::argx = \"bad arg\"").unwrap();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("[msg]"));
    }

    #[test]
    fn repl_inspect_shows_definition_summaries() {
        let input = Cursor::new(":inspect x\n:inspect f\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();
        session.eval_text("x = 1").unwrap();
        session.eval_text("f[z_Integer] := z").unwrap();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("own: x = 1"));
        assert!(rendered.contains("down[1]: f[Pattern[z, Blank[Integer]]] := z"));
    }

    #[test]
    fn repl_messages_lists_catalog_entries_for_a_symbol() {
        let input = Cursor::new(":messages f\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();
        session.eval_text("f::argx = \"bad arg\"").unwrap();
        session.eval_text("f::usage := suffix").unwrap();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("f::argx = \"bad arg\""));
        assert!(rendered.contains("f::usage := suffix"));
    }

    #[test]
    fn repl_messages_reports_when_a_symbol_has_no_messages() {
        let input = Cursor::new(":messages f\n:quit\n");
        let mut output = Vec::new();
        let mut repl = Repl::new(input, &mut output);
        let mut session = Session::new();

        repl.run(&mut session).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        assert!(rendered.contains("! no messages for `f`"));
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
