use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ToStringFormat {
    FullForm,
    InputForm,
    OutputLike,
}

const PREC_LOWEST: u8 = 0;
const PREC_RULE: u8 = 10;
const PREC_STRING_EXPRESSION: u8 = 15;
const PREC_PLUS: u8 = 20;
const PREC_TIMES: u8 = 30;
const PREC_POWER: u8 = 40;
const PREC_CALL: u8 = 90;

pub(in crate::runtime) fn render_input_like_expr(expr: &Expr) -> String {
    render_to_string_expr(expr, ToStringFormat::OutputLike)
}

pub(super) fn render_to_string_expr(expr: &Expr, format: ToStringFormat) -> String {
    render_to_string_expr_with_precedence(expr, format, PREC_LOWEST)
}

fn render_to_string_expr_with_precedence(
    expr: &Expr,
    format: ToStringFormat,
    parent_precedence: u8,
) -> String {
    if format == ToStringFormat::FullForm {
        return expr.to_string();
    }

    match expr {
        Expr::Atom(_) => expr.to_string(),
        Expr::Call { head, args } => {
            if is_system_symbol_name(head, heads::messages::MESSAGE_NAME) && args.len() == 2 {
                return expr.to_string();
            }

            if format == ToStringFormat::OutputLike
                && is_system_symbol_name(head, "StringForm")
                && !args.is_empty()
            {
                if let Some(template) = expr_as_string(&args[0]) {
                    return string_form_value(template, &args[1..]);
                }
            }

            if let Some(entries) = association_entries_from_expr(expr) {
                let rendered = format!(
                    "<|{}|>",
                    entries
                        .iter()
                        .map(|(key, value)| format!(
                            "{} -> {}",
                            render_to_string_expr_with_precedence(key, format, PREC_RULE + 1),
                            render_to_string_expr_with_precedence(value, format, PREC_RULE + 1)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::structure::LIST) {
                return format!(
                    "{{{}}}",
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            if is_system_symbol_name(head, heads::functions::SLOT) && args.len() == 1 {
                if let Some(index) = expr_as_non_negative_usize(&args[0]) {
                    return if index == 1 {
                        "#".to_string()
                    } else {
                        format!("#{index}")
                    };
                }
            }

            if is_system_symbol_name(head, heads::functions::SLOT_SEQUENCE) && args.len() == 1 {
                if let Some(index) = expr_as_non_negative_usize(&args[0]) {
                    return if index == 1 {
                        "##".to_string()
                    } else {
                        format!("##{index}")
                    };
                }
            }

            if is_system_symbol_name(head, heads::functions::FUNCTION) && args.len() == 1 {
                let rendered = format!(
                    "{} &",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if format == ToStringFormat::OutputLike
                && is_system_symbol_name(head, "HoldForm")
                && args.len() == 1
            {
                return render_to_string_expr_with_precedence(&args[0], format, parent_precedence);
            }

            if (is_system_symbol_name(head, "HoldCompleteForm")
                || is_system_symbol_name(head, "Defer")
                || is_system_symbol_name(head, "Unevaluated"))
                && args.len() == 1
            {
                return render_to_string_expr_with_precedence(&args[0], format, parent_precedence);
            }

            if is_system_symbol_name(head, "Part") && args.len() >= 2 {
                let rendered = format!(
                    "{}[[{}]]",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL),
                    args[1..]
                        .iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::arithmetic::PLUS) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_PLUS))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    PREC_PLUS,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, heads::arithmetic::TIMES) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_TIMES))
                        .collect::<Vec<_>>()
                        .join(" * "),
                    PREC_TIMES,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, heads::arithmetic::POWER) && args.len() == 2 {
                let rendered = format!(
                    "{} ^ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_POWER),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_POWER)
                );
                return wrap_rendered(rendered, PREC_POWER, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::SET) && args.len() == 2 {
                let rendered = format!(
                    "{} = {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::SET_DELAYED) && args.len() == 2 {
                let rendered = format!(
                    "{} := {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::UNSET) && args.len() == 1 {
                let rendered = format!(
                    "{} =.",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
                let rendered = format!(
                    "{} -> {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "Map") && args.len() == 2 {
                let rendered = format!(
                    "{} /@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "Apply") && args.len() == 2 {
                let rendered = format!(
                    "{} @@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "MapApply") && args.len() == 2 {
                let rendered = format!(
                    "{} @@@ {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::REPLACE_ALL) && args.len() == 2 {
                let rendered = format!(
                    "{} /. {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::REPLACE_REPEATED) && args.len() == 2 {
                let rendered = format!(
                    "{} //. {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_RULE))
                        .collect::<Vec<_>>()
                        .join(" | "),
                    PREC_RULE,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| {
                            render_to_string_expr_with_precedence(
                                arg,
                                format,
                                PREC_STRING_EXPRESSION,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" ~~ "),
                    PREC_STRING_EXPRESSION,
                    parent_precedence,
                );
            }

            if is_system_symbol_name(head, "Repeated") && args.len() == 1 {
                let rendered = format!(
                    "{}..",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, "RepeatedNull") && args.len() == 1 {
                let rendered = format!(
                    "{}...",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 {
                let rendered = format!(
                    "{} /; {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 {
                let rendered = format!(
                    "{}?{}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_CALL),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_CALL)
                );
                return wrap_rendered(rendered, PREC_CALL, parent_precedence);
            }

            if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 {
                if let Some(shorthand) = render_named_blank_pattern(expr, format) {
                    return shorthand;
                }

                let rendered = format!(
                    "{} : {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if let Some(blank) = render_blank_pattern(head, args, format) {
                return blank;
            }

            if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() {
                let pattern = render_to_string_expr_with_precedence(&args[0], format, PREC_CALL);
                let rendered = match args.as_slice() {
                    [_] => format!("{pattern}."),
                    [_, default] => format!(
                        "{pattern} : {}",
                        render_to_string_expr_with_precedence(default, format, PREC_RULE + 1)
                    ),
                    _ => expr.to_string(),
                };
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
                let rendered = format!(
                    "{} :> {}",
                    render_to_string_expr_with_precedence(&args[0], format, PREC_RULE + 1),
                    render_to_string_expr_with_precedence(&args[1], format, PREC_RULE + 1)
                );
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, heads::structure::SPAN) {
                let rendered = match args.as_slice() {
                    [start] if is_system_symbol_name(start, heads::structure::ALL) => {
                        ";;".to_string()
                    }
                    [start] => format!(
                        "{} ;;",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1)
                    ),
                    [start, end] if is_system_symbol_name(start, heads::structure::ALL) => format!(
                        ";; {}",
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1)
                    ),
                    [start, end] if is_system_symbol_name(end, heads::structure::ALL) => format!(
                        "{} ;;",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1)
                    ),
                    [start, end] => format!(
                        "{} ;; {}",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1)
                    ),
                    [start, end, step]
                        if is_system_symbol_name(start, heads::structure::ALL)
                            && is_system_symbol_name(end, heads::structure::ALL) =>
                    {
                        format!(
                            ";; ;; {}",
                            render_to_string_expr_with_precedence(step, format, PREC_RULE + 1)
                        )
                    }
                    [start, end, step] => format!(
                        "{} ;; {} ;; {}",
                        render_to_string_expr_with_precedence(start, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(end, format, PREC_RULE + 1),
                        render_to_string_expr_with_precedence(step, format, PREC_RULE + 1)
                    ),
                    _ => expr.to_string(),
                };
                return wrap_rendered(rendered, PREC_RULE, parent_precedence);
            }

            if is_system_symbol_name(head, "CompoundExpression") && !args.is_empty() {
                return wrap_rendered(
                    args.iter()
                        .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_RULE))
                        .collect::<Vec<_>>()
                        .join("; "),
                    PREC_RULE,
                    parent_precedence,
                );
            }

            let rendered_head = render_to_string_expr_with_precedence(head, format, PREC_CALL);
            format!(
                "{}[{}]",
                rendered_head,
                args.iter()
                    .map(|arg| render_to_string_expr_with_precedence(arg, format, PREC_LOWEST))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

fn wrap_rendered(rendered: String, own_precedence: u8, parent_precedence: u8) -> String {
    if own_precedence < parent_precedence {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn render_named_blank_pattern(expr: &Expr, format: ToStringFormat) -> Option<String> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, heads::patterns::PATTERN) || args.len() != 2 {
        return None;
    }

    let name = render_to_string_expr_with_precedence(&args[0], format, PREC_CALL);
    let Expr::Call {
        head: blank_head,
        args: blank_args,
    } = &args[1]
    else {
        return None;
    };

    let prefix = if is_system_symbol_name(blank_head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let suffix = match blank_args.as_slice() {
        [] => String::new(),
        [constraint] => render_to_string_expr_with_precedence(constraint, format, PREC_CALL),
        _ => return None,
    };

    Some(format!("{name}{prefix}{suffix}"))
}

fn render_blank_pattern(head: &Expr, args: &[Expr], format: ToStringFormat) -> Option<String> {
    let prefix = if is_system_symbol_name(head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let suffix = match args {
        [] => String::new(),
        [constraint] => render_to_string_expr_with_precedence(constraint, format, PREC_CALL),
        _ => return None,
    };

    Some(format!("{prefix}{suffix}"))
}
