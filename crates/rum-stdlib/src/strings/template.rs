use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TemplateSegment {
    Text(String),
    PositionalSlot,
    NamedSlot(String),
}

pub(crate) fn eval_string_form(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(session.invalid_call_with_arg_range("StringForm", args, 1, usize::MAX));
    }

    let Some(template) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::call(
        head.clone(),
        std::iter::once(Expr::string(template))
            .chain(args[1..].iter().cloned())
            .collect(),
    ))
}

pub(crate) fn eval_string_template(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("StringTemplate", args, 1));
    }

    let Some(template) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(render_template_object(template))
}

pub(crate) fn eval_template_apply(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [template] => Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("TemplateApply"),
                vec![template.clone(), string_slot_expr()],
            )],
        )),
        [template, data] => {
            let Some(template) = expr_as_template_source(template) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let Some(rendered) = template_apply_value(template, data) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            Ok(Expr::string(rendered))
        }
        _ => Ok(session.invalid_call_with_arg_range("TemplateApply", args, 1, 2)),
    }
}

pub(crate) fn eval_file_template_apply(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [template_path] => Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("FileTemplateApply"),
                vec![template_path.clone(), string_slot_expr()],
            )],
        )),
        [template_path, data] => {
            let Some(path) = expr_as_string(template_path) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            let template_source = fs::read_to_string(path).map_err(|source| EvalError::Io {
                path: path.to_string(),
                source,
            })?;
            let Some(rendered) = template_apply_value(&template_source, data) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            let output_path = temp_rendered_template_path(Path::new(path));
            fs::write(&output_path, rendered).map_err(|source| EvalError::Io {
                path: output_path.display().to_string(),
                source,
            })?;

            Ok(Expr::string(output_path.display().to_string()))
        }
        _ => Ok(session.invalid_call_with_arg_range("FileTemplateApply", args, 1, 2)),
    }
}

fn render_template_object(template: &str) -> Expr {
    Expr::call(
        system_expr("TemplateObject"),
        vec![Expr::string(template.to_string())],
    )
}

fn expr_as_template_source(expr: &Expr) -> Option<&str> {
    if let Some(template) = expr_as_string(expr) {
        return Some(template);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "TemplateObject") || args.len() != 1 {
        return None;
    }

    expr_as_string(&args[0])
}

fn template_apply_value(template: &str, data: &Expr) -> Option<String> {
    let segments = parse_template_segments(template)?;
    let named_slots = template_named_slot_values(data);
    let positional_slots = template_positional_slot_values(data)?;
    let mut positional_index = 0usize;
    let mut output = String::new();

    for segment in segments {
        match segment {
            TemplateSegment::Text(text) => output.push_str(&text),
            TemplateSegment::PositionalSlot => {
                let value = positional_slots.get(positional_index)?;
                output.push_str(value);
                positional_index += 1;
            }
            TemplateSegment::NamedSlot(name) => {
                let value = named_slots.get(&name)?;
                output.push_str(value);
            }
        }
    }

    Some(output)
}

fn parse_template_segments(template: &str) -> Option<Vec<TemplateSegment>> {
    let chars = template.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '`' {
            index += 1;
            continue;
        }

        let closing = chars[index + 1..]
            .iter()
            .position(|ch| *ch == '`')
            .map(|offset| index + 1 + offset)?;

        if start < index {
            segments.push(TemplateSegment::Text(chars[start..index].iter().collect()));
        }

        let slot_name = chars[index + 1..closing].iter().collect::<String>();
        if slot_name.is_empty() {
            segments.push(TemplateSegment::PositionalSlot);
        } else {
            segments.push(TemplateSegment::NamedSlot(slot_name));
        }

        index = closing + 1;
        start = index;
    }

    if start < chars.len() {
        segments.push(TemplateSegment::Text(chars[start..].iter().collect()));
    }

    Some(segments)
}

fn template_named_slot_values(data: &Expr) -> BTreeMap<String, String> {
    let mut named = BTreeMap::new();

    if let Some(entries) = association_entries_from_expr(data) {
        for (key, value) in entries {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
        return named;
    }

    if let Some(rules) = template_rule_entries(data) {
        for (key, value) in rules {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
    }

    named
}

fn template_positional_slot_values(data: &Expr) -> Option<Vec<String>> {
    if let Some(items) = call_items_with_head(data, &system_expr("List")) {
        if items.iter().all(|item| parse_template_rule(item).is_some()) {
            return Some(Vec::new());
        }
        return Some(items.iter().map(template_fragment_value).collect());
    }

    if association_entries_from_expr(data).is_some() || template_rule_entries(data).is_some() {
        return Some(Vec::new());
    }

    Some(vec![template_fragment_value(data)])
}

fn template_rule_entries(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(rule) = parse_template_rule(expr) {
        return Some(vec![rule]);
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    items.iter().map(parse_template_rule).collect()
}

fn parse_template_rule(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn template_slot_name_from_expr(expr: &Expr) -> Option<String> {
    if let Some(value) = expr_as_string(expr) {
        return Some(value.to_string());
    }

    expr_as_symbol(expr).map(|symbol| symbol.name().to_string())
}

fn template_fragment_value(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        _ => expr.to_string(),
    }
}

fn temp_rendered_template_path(source_path: &Path) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let mut filename = format!("template-{}-{nanos}", std::process::id());
    if let Some(extension) = source_path.extension().and_then(|ext| ext.to_str()) {
        filename.push('.');
        filename.push_str(extension);
    }
    std::env::temp_dir().join(filename)
}
