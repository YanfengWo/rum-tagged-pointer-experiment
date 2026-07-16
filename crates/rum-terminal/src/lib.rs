use std::fmt::Write;

use rum_host::{
    HostDefinition, HostEvalSuccess, HostFailure, HostFailureCode, HostInspectResponse,
    HostMessage, HostMessageTemplate,
};

pub fn render_host_message(message: &HostMessage) -> String {
    format!("! {}::{}: {}", message.symbol, message.tag, message.text)
}

pub fn render_host_messages(messages: &[HostMessage]) -> String {
    messages
        .iter()
        .map(render_host_message)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_host_failure(failure: &HostFailure) -> String {
    let mut output = render_host_messages(&failure.messages);
    if !output.is_empty() {
        output.push('\n');
    }

    if failure.code == HostFailureCode::Syntax {
        output.push_str(
            failure
                .syntax_rendered
                .as_deref()
                .expect("syntax host failures carry a rendered diagnostic"),
        );
    } else {
        output.push_str("! ");
        output.push_str(&failure.message);
    }

    output
}

pub fn render_host_eval_response(response: &HostEvalSuccess) -> String {
    let mut output = String::new();

    if !response.messages.is_empty() {
        output.push_str(&render_host_messages(&response.messages));
        output.push('\n');
    }

    output.push_str(&response.value.text);
    output
}

pub fn render_host_inspect(response: &HostInspectResponse) -> String {
    let mut output = String::new();
    let info = response
        .info
        .as_ref()
        .expect("found host inspection responses carry symbol information");
    let mut flags = Vec::new();

    if info.is_builtin {
        flags.push("builtin");
    }

    if !info.attributes.is_empty() {
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

    if !info.message_tags.is_empty() {
        flags.push("msg");
    }

    writeln!(
        &mut output,
        "{} :: {}{}",
        info.qualified_name,
        info.summary.as_deref().unwrap_or_default(),
        if flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", flags.join(", "))
        }
    )
    .unwrap();
    writeln!(&mut output, "  context: {}", info.context).unwrap();

    if !info.attributes.is_empty() {
        writeln!(
            &mut output,
            "  attributes[{}]: {}",
            info.attributes.len(),
            info.attributes.join(", ")
        )
        .unwrap();
    }

    if !info.message_tags.is_empty() {
        writeln!(
            &mut output,
            "  messages[{}]: {}",
            info.message_tags.len(),
            info.message_tags.join(", ")
        )
        .unwrap();
    }

    let own_entries = response
        .definitions
        .iter()
        .filter(|entry| matches!(entry.kind.as_str(), "OwnValue" | "DelayedOwnValue"))
        .collect::<Vec<_>>();
    let up_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == "UpValue")
        .collect::<Vec<_>>();
    let down_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == "DownValue")
        .collect::<Vec<_>>();
    let sub_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == "SubValue")
        .collect::<Vec<_>>();

    if !own_entries.is_empty() {
        writeln!(
            &mut output,
            "  own: {}",
            own_entries
                .iter()
                .map(|entry| format_definition_entry(entry))
                .collect::<Vec<_>>()
                .join(" | ")
        )
        .unwrap();
    }

    if !up_entries.is_empty() {
        writeln!(
            &mut output,
            "  up[{}]: {}",
            up_entries.len(),
            up_entries
                .iter()
                .map(|entry| format_definition_entry(entry))
                .collect::<Vec<_>>()
                .join(" | ")
        )
        .unwrap();
    }

    if !down_entries.is_empty() {
        writeln!(
            &mut output,
            "  down[{}]: {}",
            down_entries.len(),
            down_entries
                .iter()
                .map(|entry| format_definition_entry(entry))
                .collect::<Vec<_>>()
                .join(" | ")
        )
        .unwrap();
    }

    if !sub_entries.is_empty() {
        writeln!(
            &mut output,
            "  sub[{}]: {}",
            sub_entries.len(),
            sub_entries
                .iter()
                .map(|entry| format_definition_entry(entry))
                .collect::<Vec<_>>()
                .join(" | ")
        )
        .unwrap();
    }

    output.trim_end().to_string()
}

pub fn render_host_message_catalog(entries: &[HostMessageTemplate]) -> String {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{} {} {}",
                entry.message_name,
                if entry.delayed { ":=" } else { "=" },
                entry.template
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_definition_entry(entry: &HostDefinition) -> String {
    format!(
        "{} {} {}",
        entry.lhs,
        if entry.delayed { ":=" } else { "=" },
        entry.rhs
    )
}

#[cfg(test)]
mod tests {
    use rum_host::{
        HostStructuredOutput, HostSymbolInfo, HostTerminationReason, HostTraceEvent, HostTyped,
        HostValue,
    };

    use super::*;

    fn eval_response(text: &str) -> HostEvalSuccess {
        HostEvalSuccess {
            value: HostValue {
                text: text.to_string(),
                typed: HostTyped::Other {
                    head: "Test".to_string(),
                    text: text.to_string(),
                },
            },
            termination: None::<HostTerminationReason>,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::<HostTraceEvent>::new(),
            outputs: Vec::<HostStructuredOutput>::new(),
        }
    }

    #[test]
    fn host_eval_response_uses_hold_transparent_canonical_contract_text() {
        assert_eq!(
            render_host_eval_response(&eval_response("Plus[1, 1]")),
            "Plus[1, 1]"
        );
        assert_eq!(
            render_host_eval_response(&eval_response("List[Plus[1, 1]]")),
            "List[Plus[1, 1]]"
        );
    }

    #[test]
    fn host_eval_response_keeps_canonical_contract_bytes() {
        assert_eq!(
            render_host_eval_response(&eval_response("List[2, f[a]]")),
            "List[2, f[a]]"
        );
    }

    #[test]
    fn host_failure_renders_messages_before_syntax_diagnostic() {
        let failure = HostFailure {
            code: HostFailureCode::Syntax,
            exit_status: None,
            message: "expected ,".to_string(),
            syntax_rendered: Some("error: expected ,\n --> 1:5".to_string()),
            messages: vec![HostMessage {
                symbol: "FindFile".to_string(),
                tag: "notfound".to_string(),
                text: "missing".to_string(),
            }],
            prints: Vec::new(),
            trace: Vec::new(),
            outputs: Vec::new(),
        };

        let rendered = render_host_failure(&failure);
        assert!(rendered.starts_with("! FindFile::notfound: missing\nerror: expected ,"));
    }

    #[test]
    fn host_inspect_renders_summary_and_definition_lines() {
        let response = HostInspectResponse {
            target: Some("f".to_string()),
            found: true,
            info: Some(HostSymbolInfo {
                qualified_name: "Global`f".to_string(),
                context: "Global`".to_string(),
                display_name: "f".to_string(),
                summary: None,
                is_builtin: false,
                attributes: Vec::new(),
                has_own_value: false,
                has_delayed_value: false,
                has_up_values: false,
                has_down_values: true,
                has_sub_values: false,
                message_tags: vec!["usage".to_string()],
            }),
            definitions: vec![HostDefinition {
                kind: "DownValue".to_string(),
                lhs: "f".to_string(),
                rhs: "1".to_string(),
                delayed: true,
            }],
            message_catalog: Vec::new(),
        };

        let rendered = render_host_inspect(&response);
        assert!(rendered.contains("Global`f ::"));
        assert!(rendered.contains("messages[1]: usage"));
        assert!(rendered.contains("down[1]: f := 1"));
    }

    #[test]
    fn host_message_catalog_renders_immediate_and_delayed_templates() {
        let entries = vec![
            HostMessageTemplate {
                message_name: "f::argx".to_string(),
                qualified_symbol_name: "Global`f".to_string(),
                tag: "argx".to_string(),
                template: "\"bad arg\"".to_string(),
                delayed: false,
            },
            HostMessageTemplate {
                message_name: "f::usage".to_string(),
                qualified_symbol_name: "Global`f".to_string(),
                tag: "usage".to_string(),
                template: "suffix".to_string(),
                delayed: true,
            },
        ];

        assert_eq!(
            render_host_message_catalog(&entries),
            "f::argx = \"bad arg\"\nf::usage := suffix"
        );
    }
}
