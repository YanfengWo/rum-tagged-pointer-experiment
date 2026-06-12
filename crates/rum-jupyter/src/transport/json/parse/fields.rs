use super::*;

pub(super) fn parse_object(
    input: &str,
    context: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    parse_json_object(input).map_err(|message| {
        if let Some(found) = message.strip_prefix("expected top-level object, found ") {
            KernelWireError::InvalidField {
                context,
                field: "<root>",
                value: found.to_string(),
            }
        } else {
            KernelWireError::InvalidJson { message }
        }
    })
}

pub(super) fn string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<String, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(super) fn usize_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<usize, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<usize>()
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(super) fn bool_field_with_default(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
    default: bool,
) -> Result<bool, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(default),
    }
}

pub(super) fn scalar_to_string(
    _context: &'static str,
    value: JsonValue,
) -> Result<String, KernelWireError> {
    Ok(match value {
        JsonValue::String(value) => value,
        JsonValue::Number(value) => value,
        JsonValue::Bool(value) => {
            if value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        JsonValue::Null => "null".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => json_value_to_json(&value),
    })
}

fn json_value_to_json(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(value) => json_bool(*value),
        JsonValue::Number(value) => value.clone(),
        JsonValue::String(value) => json_string(value),
        JsonValue::Array(values) => json_array(values.iter().map(json_value_to_json).collect()),
        JsonValue::Object(values) => json_object(
            values
                .iter()
                .map(|(key, value)| field(key, json_value_to_json(value)))
                .collect(),
        ),
    }
}

pub(super) fn optional_string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

pub(super) fn optional_u64_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<u64>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

pub(super) fn string_array_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                JsonValue::String(value) => Ok(value.clone()),
                other => Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                }),
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(super) fn metadata_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMetadata, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut metadata = KernelMetadata::new();
    for (key, value) in nested {
        metadata.insert(key, scalar_to_string(context, value)?);
    }
    Ok(metadata)
}

pub(super) fn mime_bundle_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMimeBundle, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut bundle = KernelMimeBundle::new();
    for (key, value) in nested {
        bundle.insert(key, scalar_to_string(context, value)?);
    }
    Ok(bundle)
}

pub(super) fn mime_bundle_map_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, KernelMimeBundle>, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut result = BTreeMap::new();
    for (key, value) in nested {
        let inner = match value {
            JsonValue::Object(map) => map,
            other => {
                return Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                });
            }
        };
        let mut bundle = KernelMimeBundle::new();
        for (mime, value) in inner {
            bundle.insert(mime, scalar_to_string(context, value)?);
        }
        result.insert(key, bundle);
    }
    Ok(result)
}

pub(super) fn execute_payloads_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelExecutePayload>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelExecutePayload {
                    source: string_field(item, context, "source")?,
                    data: mime_bundle_field(item, context, "data")?,
                    metadata: metadata_field(item, context, "metadata")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

pub(super) fn history_entries_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelHistoryEntry>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Array(item) if item.len() == 3 => item,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                let (input, output) = parse_history_input_output(&item[2], context, field)?;
                Ok(KernelHistoryEntry {
                    session: u64_json_value(&item[0], context, field)?,
                    line_number: u64_json_value(&item[1], context, field)?,
                    input,
                    output,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

fn parse_history_input_output(
    value: &JsonValue,
    context: &'static str,
    field: &'static str,
) -> Result<(String, Option<String>), KernelWireError> {
    match value {
        JsonValue::String(input) => Ok((input.clone(), None)),
        JsonValue::Array(values) if values.len() == 2 => {
            let JsonValue::String(input) = &values[0] else {
                return Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: values[0].kind_name().to_string(),
                });
            };
            let JsonValue::String(output) = &values[1] else {
                return Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: values[1].kind_name().to_string(),
                });
            };
            Ok((input.clone(), Some(output.clone())))
        }
        other => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
    }
}

fn u64_json_value(
    value: &JsonValue,
    context: &'static str,
    field: &'static str,
) -> Result<u64, KernelWireError> {
    match value {
        JsonValue::Number(value) => {
            value
                .parse::<u64>()
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        other => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
    }
}

pub(super) fn language_info_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelLanguageInfo, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    Ok(KernelLanguageInfo {
        name: string_field(&nested, context, "name")?,
        version: string_field(&nested, context, "version")?,
        mimetype: string_field(&nested, context, "mimetype")?,
        file_extension: string_field(&nested, context, "file_extension")?,
        pygments_lexer: string_field(&nested, context, "pygments_lexer")?,
        codemirror_mode: string_field(&nested, context, "codemirror_mode")?,
    })
}

pub(super) fn help_links_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelHelpLink>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelHelpLink {
                    text: string_field(item, context, "text")?,
                    url: string_field(item, context, "url")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

pub(super) fn parse_error_fields(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
) -> Result<KernelErrorOutput, KernelWireError> {
    let rendered = match object.get("traceback") {
        Some(JsonValue::Array(values)) => values.first().and_then(|value| match value {
            JsonValue::String(value) => Some(value.clone()),
            _ => None,
        }),
        _ => None,
    };

    Ok(KernelErrorOutput {
        error_name: string_field(object, context, "ename")?,
        error_value: string_field(object, context, "evalue")?,
        rendered,
    })
}

fn nested_object_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Object(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}
