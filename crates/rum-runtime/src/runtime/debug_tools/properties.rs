use super::*;

impl Session {
    pub(in crate::runtime::debug_tools) fn project_association_properties(
        &mut self,
        head_name: &str,
        call_prefix: &[Expr],
        data: Expr,
        properties: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(properties) = properties else {
            return Ok(data);
        };

        if is_system_symbol_name(&properties, "All") {
            return Ok(data);
        }

        match property_name_string(&properties) {
            Some(key) if key == "Properties" => Ok(render_association_properties(&data)),
            Some(key) => Ok(
                association_property_by_name(&data, &key).unwrap_or_else(|| {
                    Expr::call(
                        system_expr("Missing"),
                        vec![Expr::string("KeyAbsent"), properties.clone()],
                    )
                }),
            ),
            None => {
                let Some(keys) = list_items(&properties) else {
                    let mut call = call_prefix.to_vec();
                    call.push(properties.clone());
                    return Ok(self.invalid_call_with_arg_form(
                        head_name,
                        &call,
                        "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
                    ));
                };
                let mut entries = Vec::new();
                for key_expr in keys {
                    let Some(key) = property_name_string(key_expr) else {
                        let mut call = call_prefix.to_vec();
                        call.push(properties.clone());
                        return Ok(self.invalid_call_with_arg_form(
                            head_name,
                            &call,
                            "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
                        ));
                    };
                    let value = association_property_by_name(&data, &key).unwrap_or_else(|| {
                        Expr::call(
                            system_expr("Missing"),
                            vec![Expr::string("KeyAbsent"), key_expr.clone()],
                        )
                    });
                    entries.push((Expr::string(key.clone()), value));
                }
                Ok(render_association(entries))
            }
        }
    }
}

fn property_name_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(name)) => Some(name.clone()),
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name().to_string()),
        _ => None,
    }
}
fn render_association_properties(expr: &Expr) -> Expr {
    let properties = association_entries_from_expr(expr)
        .map(|entries| {
            entries
                .iter()
                .map(|(key, _)| match key {
                    Expr::Atom(Atom::String(text)) => Expr::string(text.clone()),
                    other => Expr::string(other.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();
    Expr::call(system_expr("List"), properties)
}

pub(super) fn association_property_by_name(expr: &Expr, name: &str) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find_map(|(key, value)| match key {
            Expr::Atom(Atom::String(current)) if current == name => Some(value),
            _ => None,
        })
}
