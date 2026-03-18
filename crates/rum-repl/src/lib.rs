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

                    if info.attribute_count > 0 {
                        flags.push("attr");
                    }

                    if info.has_own_value {
                        flags.push("own");
                    }

                    if info.has_delayed_value {
                        flags.push("delayed");
                    }

                    if info.has_up_values {
                        flags.push("up");
                    }

                    if info.has_down_values {
                        flags.push("down");
                    }

                    if info.has_sub_values {
                        flags.push("sub");
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
                        let sub_entries = entries
                            .iter()
                            .filter(|entry| entry.kind == DefinitionKind::SubValue)
                            .collect::<Vec<_>>();
                        let up_entries = entries
                            .iter()
                            .filter(|entry| entry.kind == DefinitionKind::UpValue)
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

                        if !up_entries.is_empty() {
                            writeln!(
                                self.output,
                                "  up[{}]: {}",
                                up_entries.len(),
                                up_entries
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

                        if !sub_entries.is_empty() {
                            writeln!(
                                self.output,
                                "  sub[{}]: {}",
                                sub_entries.len(),
                                sub_entries
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
#[path = "tests.rs"]
mod tests;
