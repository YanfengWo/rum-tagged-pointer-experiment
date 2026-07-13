use std::fmt::Write;

use rum_runtime::{
    DefinitionCatalogEntry, DefinitionKind, MessageCatalogEntry, RuntimeMessage,
    SessionEvalResponse, SessionInspectResponse, SessionRequestError, SessionRequestFailure,
    kernel_api::{ToStringFormat, render_to_string_expr},
};

pub fn render_runtime_message(message: &RuntimeMessage) -> String {
    format!("! {message}")
}

pub fn render_runtime_messages(messages: &[RuntimeMessage]) -> String {
    messages
        .iter()
        .map(render_runtime_message)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_request_failure(failure: &SessionRequestFailure) -> String {
    let mut output = String::new();

    if !failure.messages.is_empty() {
        output.push_str(&render_runtime_messages(&failure.messages));
        output.push('\n');
    }

    match &failure.error {
        SessionRequestError::Syntax(diagnostic) => output.push_str(&diagnostic.rendered),
        error => {
            write!(&mut output, "! {error}").unwrap();
        }
    }

    output
}

pub fn render_eval_response(response: &SessionEvalResponse) -> String {
    let mut output = String::new();

    if !response.messages.is_empty() {
        output.push_str(&render_runtime_messages(&response.messages));
        output.push('\n');
    }

    output.push_str(&render_eval_value(response));
    output
}

pub fn render_eval_value(response: &SessionEvalResponse) -> String {
    render_to_string_expr(
        &response.value,
        ToStringFormat::CanonicalHoldFormTransparent,
    )
}

pub fn render_inspect_response(response: &SessionInspectResponse) -> String {
    let mut output = String::new();
    let info = &response.info;
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
        &mut output,
        "{} :: {}{}",
        info.qualified_name,
        info.summary.clone().unwrap_or_default(),
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
        .filter(|entry| {
            matches!(
                entry.kind,
                DefinitionKind::OwnValue | DefinitionKind::DelayedOwnValue
            )
        })
        .collect::<Vec<_>>();
    let up_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == DefinitionKind::UpValue)
        .collect::<Vec<_>>();
    let down_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == DefinitionKind::DownValue)
        .collect::<Vec<_>>();
    let sub_entries = response
        .definitions
        .iter()
        .filter(|entry| entry.kind == DefinitionKind::SubValue)
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

pub fn render_message_catalog_entries(entries: &[MessageCatalogEntry]) -> String {
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
    use rum_core::{BigInt, Expr};
    use rum_runtime::{
        RuntimeMessage, SessionInspectResponse, SessionRequestError, SessionRequestFailure,
        SymbolInfo,
    };
    use rum_syntax::StructuredSourceDiagnostic;

    use super::*;

    fn eval_response(value: Expr) -> SessionEvalResponse {
        SessionEvalResponse {
            value,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
        }
    }

    fn call(name: &str, args: Vec<Expr>) -> Expr {
        Expr::call(Expr::system_symbol(name), args)
    }

    #[test]
    fn eval_value_renders_hold_form_transparently_in_canonical_style() {
        let plus = call(
            "Plus",
            vec![
                Expr::integer(BigInt::from(1_i32)),
                Expr::integer(BigInt::from(1_i32)),
            ],
        );
        let held = call("HoldForm", vec![plus.clone()]);

        assert_eq!(
            render_eval_value(&eval_response(held.clone())),
            "Plus[1, 1]"
        );
        assert_eq!(
            render_eval_value(&eval_response(call("List", vec![held]))),
            "List[Plus[1, 1]]"
        );
    }

    #[test]
    fn eval_value_without_hold_form_keeps_canonical_bytes() {
        let value = call(
            "List",
            vec![
                Expr::integer(BigInt::from(2_i32)),
                Expr::call(Expr::symbol("f"), vec![Expr::symbol("a")]),
            ],
        );
        let response = eval_response(value.clone());

        assert_eq!(render_eval_value(&response), value.to_string());
        assert_eq!(render_eval_value(&response), "List[2, f[a]]");
    }

    #[test]
    fn request_failure_renders_messages_before_syntax_diagnostic() {
        let failure = SessionRequestFailure {
            error: SessionRequestError::Syntax(StructuredSourceDiagnostic {
                message: "expected ,".to_string(),
                rendered: "error: expected ,\n --> 1:5".to_string(),
                is_incomplete: false,
                spans: Vec::new(),
            }),
            messages: vec![RuntimeMessage {
                symbol: "FindFile".to_string(),
                tag: "notfound".to_string(),
                text: "missing".to_string(),
            }],
            prints: Vec::new(),
            trace: Vec::new(),
        };

        let rendered = render_request_failure(&failure);
        assert!(rendered.starts_with("! FindFile::notfound: missing\nerror: expected ,"));
    }

    #[test]
    fn inspect_response_renders_summary_and_definition_lines() {
        let response = SessionInspectResponse {
            info: SymbolInfo {
                qualified_name: "Global`f".to_string(),
                context: "Global`".to_string(),
                display_name: "f".to_string(),
                summary: None,
                is_builtin: false,
                attribute_count: 0,
                attributes: Vec::new(),
                has_own_value: false,
                has_delayed_value: false,
                has_up_values: false,
                has_down_values: true,
                has_sub_values: false,
                message_count: 1,
                message_tags: vec!["usage".to_string()],
            },
            definitions: vec![DefinitionCatalogEntry {
                kind: DefinitionKind::DownValue,
                lhs: Expr::symbol("f"),
                rhs: Expr::integer(BigInt::from(1_i32)),
                delayed: true,
            }],
            message_catalog: Vec::new(),
        };

        let rendered = render_inspect_response(&response);
        assert!(rendered.contains("Global`f ::"));
        assert!(rendered.contains("messages[1]: usage"));
        assert!(rendered.contains("down[1]: f := 1"));
    }
}
