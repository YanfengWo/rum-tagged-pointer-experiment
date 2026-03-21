use rum_runtime::Session;
use rum_syntax::{FormatterOptions, format_parsed_source, parse_format_command};
use rum_terminal::{
    render_inspect_response, render_message_catalog_entries, render_request_failure,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReplCommand {
    Help,
    Quit,
    Parse {
        expr: Option<String>,
    },
    Format {
        expr: String,
        options: FormatterOptions,
    },
    Lower {
        expr: Option<String>,
    },
    History {
        limit: Option<usize>,
    },
    Complete {
        prefix: Option<String>,
    },
    Inspect {
        name: Option<String>,
    },
    Messages {
        query: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CommandExecution {
    Exit,
    Output(String),
}

pub(super) fn parse_repl_command(line: &str) -> Result<Option<ReplCommand>, String> {
    match line {
        ":quit" | ":q" => Ok(Some(ReplCommand::Quit)),
        ":help" => Ok(Some(ReplCommand::Help)),
        _ if line == ":parse" || line.starts_with(":parse ") => Ok(Some(ReplCommand::Parse {
            expr: parse_trailing_argument(line, ":parse"),
        })),
        _ if line == ":format" || line.starts_with(":format ") => {
            let input = line.trim_start_matches(":format").trim();
            let (expr, options) = parse_format_command(input, ":format")?;
            Ok(Some(ReplCommand::Format {
                expr: expr.to_string(),
                options,
            }))
        }
        _ if line == ":lower" || line.starts_with(":lower ") => Ok(Some(ReplCommand::Lower {
            expr: parse_trailing_argument(line, ":lower"),
        })),
        _ if line == ":history" || line.starts_with(":history ") => {
            Ok(Some(ReplCommand::History {
                limit: parse_history_limit(line)?,
            }))
        }
        _ if line == ":complete" || line.starts_with(":complete ") => {
            Ok(Some(ReplCommand::Complete {
                prefix: parse_trailing_argument(line, ":complete"),
            }))
        }
        _ if line == ":inspect" || line.starts_with(":inspect ") => {
            Ok(Some(ReplCommand::Inspect {
                name: parse_trailing_argument(line, ":inspect"),
            }))
        }
        _ if line == ":messages" || line.starts_with(":messages ") => {
            Ok(Some(ReplCommand::Messages {
                query: parse_trailing_argument(line, ":messages"),
            }))
        }
        _ => Ok(None),
    }
}

pub(super) fn execute_repl_command(
    session: &mut Session,
    history: &[String],
    command: ReplCommand,
) -> CommandExecution {
    match command {
        ReplCommand::Quit => CommandExecution::Exit,
        ReplCommand::Help => CommandExecution::Output(help_text().to_string()),
        ReplCommand::Parse { expr: None } => {
            CommandExecution::Output("! :parse expects an expression".to_string())
        }
        ReplCommand::Parse { expr: Some(expr) } => {
            CommandExecution::Output(match session.parse_request(&expr) {
                Ok(response) => response.parsed.debug_dump(),
                Err(failure) => render_request_failure(&failure),
            })
        }
        ReplCommand::Format { expr, options } => {
            CommandExecution::Output(match session.parse_request(&expr) {
                Ok(response) => format_parsed_source(&response.parsed, options),
                Err(failure) => render_request_failure(&failure),
            })
        }
        ReplCommand::Lower { expr: None } => {
            CommandExecution::Output("! :lower expects an expression".to_string())
        }
        ReplCommand::Lower { expr: Some(expr) } => {
            CommandExecution::Output(match session.lower_request(&expr) {
                Ok(response) => response.expr.to_string(),
                Err(failure) => render_request_failure(&failure),
            })
        }
        ReplCommand::History { limit } => CommandExecution::Output(render_history(history, limit)),
        ReplCommand::Complete { prefix: None } => {
            CommandExecution::Output("! :complete expects a prefix".to_string())
        }
        ReplCommand::Complete {
            prefix: Some(prefix),
        } => {
            let completions = session.complete_symbol_prefix(&prefix);
            let rendered = if completions.is_empty() {
                "(no completions)".to_string()
            } else {
                completions.join("\n")
            };
            CommandExecution::Output(rendered)
        }
        ReplCommand::Inspect { name: None } => {
            CommandExecution::Output("! :inspect expects a symbol name".to_string())
        }
        ReplCommand::Inspect { name: Some(name) } => {
            let rendered = if let Some(response) = session.inspect_request(&name) {
                render_inspect_response(&response)
            } else {
                format!("! unknown symbol `{name}`")
            };
            CommandExecution::Output(rendered)
        }
        ReplCommand::Messages { query } => {
            let rendered = match session.inspect_message_catalog(query.as_deref()) {
                None => {
                    let name = query.as_deref().unwrap_or("<invalid>");
                    format!("! invalid message selector `{name}`")
                }
                Some(entries) if entries.is_empty() => match query {
                    Some(name) => format!("! no messages for `{name}`"),
                    None => "(no message definitions)".to_string(),
                },
                Some(entries) => render_message_catalog_entries(&entries),
            };
            CommandExecution::Output(rendered)
        }
    }
}

fn parse_trailing_argument(line: &str, command: &str) -> Option<String> {
    line.strip_prefix(command)
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_history_limit(line: &str) -> Result<Option<usize>, String> {
    let raw_limit = line
        .strip_prefix(":history")
        .map(str::trim)
        .filter(|raw| !raw.is_empty());

    match raw_limit {
        Some(raw) => raw
            .parse::<usize>()
            .map(Some)
            .map_err(|_| format!("invalid history count `{raw}`")),
        None => Ok(None),
    }
}

fn render_history(history: &[String], limit: Option<usize>) -> String {
    if history.is_empty() {
        return "(no history)".to_string();
    }

    let limit = limit.unwrap_or(history.len());
    let start = history.len().saturating_sub(limit);

    history[start..]
        .iter()
        .enumerate()
        .map(|(index, entry)| format!("{:>4}: {}", start + index + 1, entry.replace('\n', "\\n")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn help_text() -> &'static str {
    ":help           Show REPL commands.\n:parse EXPR     Parse and print the syntax tree.\n:format [--line-width N] EXPR  Format syntax into the current canonical style.\n:lower EXPR     Lower syntax into core Expr form.\n:complete PREF  Show symbol and context completions.\n:history [N]    Show recent input history.\n:inspect NAME   Show basic symbol metadata.\n:messages [NAME]   Show message definitions globally or for a symbol/message.\n:quit           Exit the REPL."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_history_reports_invalid_counts() {
        let error = parse_repl_command(":history nope")
            .expect_err("invalid history counts should surface a parse error");
        assert_eq!(error, "invalid history count `nope`");
    }

    #[test]
    fn parse_format_returns_a_structured_command() {
        let command = parse_repl_command(":format f[x] --line-width=60")
            .expect("format command should parse")
            .expect("format command should be recognized");

        match command {
            ReplCommand::Format { expr, options } => {
                assert_eq!(expr, "f[x]");
                assert_eq!(options.line_width, 60);
            }
            other => panic!("expected format command, got {other:?}"),
        }
    }

    #[test]
    fn execute_history_renders_numbered_entries() {
        let history = vec!["1 + 2".to_string(), "f[\n1\n]".to_string()];
        let rendered = match execute_repl_command(
            &mut Session::new(),
            &history,
            ReplCommand::History { limit: None },
        ) {
            CommandExecution::Output(rendered) => rendered,
            other => panic!("expected command output, got {other:?}"),
        };

        assert!(rendered.contains("   1: 1 + 2"));
        assert!(rendered.contains(r"   2: f[\n1\n]"));
    }
}
