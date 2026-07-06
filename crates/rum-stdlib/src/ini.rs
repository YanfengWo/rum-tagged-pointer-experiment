use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{
    EvalError, Session, association_entries_from_expr, expr_as_string, system_expr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct IniParseError {
    offset: usize,
    message: String,
}

impl IniParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        format!(
            "Invalid INI at byte offset {}: {}.",
            self.offset, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IniEncodeError {
    message: String,
}

impl IniEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IniSection {
    name: String,
    entries: Vec<(String, String)>,
}

pub(crate) fn eval_from_ini(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [source] = args else {
        return Ok(fail_ini(
            session,
            "FromIni",
            "FromIni expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source) else {
        return Ok(fail_ini(
            session,
            "FromIni",
            "FromIni expects an INI text string.",
        ));
    };

    match from_ini(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_ini(session, "FromIni", error.message())),
    }
}

pub(crate) fn eval_to_ini(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(fail_ini(
            session,
            "ToIni",
            "ToIni expects exactly one Association argument.",
        ));
    };

    match to_ini(expr) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_ini(session, "ToIni", error.message())),
    }
}

fn from_ini(source: &str) -> Result<Expr, IniParseError> {
    IniParser::new(source).parse()
}

struct IniParser<'a> {
    source: &'a str,
    pos: usize,
    sections: Vec<IniSection>,
    current_section: Option<usize>,
}

impl<'a> IniParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            sections: Vec::new(),
            current_section: None,
        }
    }

    fn parse(mut self) -> Result<Expr, IniParseError> {
        while let Some((offset, line)) = self.next_line() {
            self.parse_line(offset, line)?;
        }
        Ok(sections_expr(self.sections))
    }

    fn next_line(&mut self) -> Option<(usize, &'a str)> {
        if self.pos >= self.source.len() {
            return None;
        }

        let start = self.pos;
        let rest = &self.source[start..];
        if let Some(relative_newline) = rest.find('\n') {
            let end = start + relative_newline;
            self.pos = end + 1;
            let mut line = &self.source[start..end];
            if let Some(stripped) = line.strip_suffix('\r') {
                line = stripped;
            }
            Some((start, line))
        } else {
            self.pos = self.source.len();
            Some((start, rest))
        }
    }

    fn parse_line(&mut self, offset: usize, line: &str) -> Result<(), IniParseError> {
        if let Some(relative) = line.find('\r') {
            return Err(IniParseError::new(
                offset + relative,
                "bare carriage return in line",
            ));
        }

        let content_start = trim_start_len(line);
        let content = &line[content_start..];
        let content_offset = offset + content_start;
        if content.trim().is_empty() {
            return Ok(());
        }
        if content.starts_with(';') || content.starts_with('#') {
            return Ok(());
        }

        if content.starts_with('[') {
            return self.parse_section_header(content_offset, content);
        }

        self.parse_key_value(content_offset, content)
    }

    fn parse_section_header(&mut self, offset: usize, content: &str) -> Result<(), IniParseError> {
        let Some(close_relative) = content[1..].find(']') else {
            return Err(IniParseError::new(offset, "unterminated section header"));
        };
        let close = close_relative + 1;
        let name = &content[1..close];
        if !content[close + 1..].trim().is_empty() {
            return Err(IniParseError::new(
                offset + close + 1,
                "characters after section header",
            ));
        }
        if self.sections.iter().any(|section| section.name == name) {
            return Err(IniParseError::new(
                offset,
                format!("duplicate section name {name:?}"),
            ));
        }

        self.sections.push(IniSection {
            name: name.to_string(),
            entries: Vec::new(),
        });
        self.current_section = Some(self.sections.len() - 1);
        Ok(())
    }

    fn parse_key_value(&mut self, offset: usize, content: &str) -> Result<(), IniParseError> {
        let Some(equals) = content.find('=') else {
            return Err(IniParseError::new(
                offset,
                "expected a section header or key = value line",
            ));
        };

        let key_raw = &content[..equals];
        let value_raw = &content[equals + 1..];
        let key = key_raw.trim();
        let key_offset = offset + trim_start_len(key_raw);
        if key.is_empty() {
            return Err(IniParseError::new(key_offset, "empty key"));
        }

        let section_index = self.ensure_current_section();
        let section = &mut self.sections[section_index];
        if section.entries.iter().any(|(seen, _)| seen == key) {
            return Err(IniParseError::new(
                key_offset,
                format!("duplicate key {key:?} in section {:?}", section.name),
            ));
        }
        section
            .entries
            .push((key.to_string(), value_raw.trim().to_string()));
        Ok(())
    }

    fn ensure_current_section(&mut self) -> usize {
        if let Some(index) = self.current_section {
            return index;
        }

        self.sections.push(IniSection {
            name: String::new(),
            entries: Vec::new(),
        });
        self.current_section = Some(self.sections.len() - 1);
        self.sections.len() - 1
    }
}

fn to_ini(expr: &Expr) -> Result<String, IniEncodeError> {
    let Some(sections) = association_entries_from_expr(expr) else {
        return Err(IniEncodeError::new(
            "ToIni expects an Association of section names to Associations.",
        ));
    };

    let mut default_lines: Option<Vec<String>> = None;
    let mut named_sections: Vec<(String, Vec<String>)> = Vec::new();

    for (section_key, section_value) in sections {
        let Expr::Atom(Atom::String(section_name)) = section_key else {
            return Err(IniEncodeError::new("ToIni section names must be Strings."));
        };
        validate_section_name(&section_name)?;

        let Some(entries) = association_entries_from_expr(&section_value) else {
            return Err(IniEncodeError::new(format!(
                "ToIni section {section_name:?} must be an Association."
            )));
        };

        let lines = section_lines(&section_name, &entries)?;
        if section_name.is_empty() {
            if lines.is_empty() {
                return Err(IniEncodeError::new(
                    "ToIni cannot represent an empty default section.",
                ));
            }
            default_lines = Some(lines);
        } else {
            named_sections.push((section_name, lines));
        }
    }

    let mut output = Vec::new();
    if let Some(lines) = default_lines {
        output.extend(lines);
    }
    for (section_name, lines) in named_sections {
        output.push(format!("[{section_name}]"));
        output.extend(lines);
    }

    Ok(output.join("\n"))
}

fn section_lines(
    section_name: &str,
    entries: &[(Expr, Expr)],
) -> Result<Vec<String>, IniEncodeError> {
    entries
        .iter()
        .map(|(key, value)| {
            let Expr::Atom(Atom::String(key)) = key else {
                return Err(IniEncodeError::new(format!(
                    "ToIni key in section {section_name:?} must be a String."
                )));
            };
            validate_key_name(key)?;

            let Expr::Atom(Atom::String(value)) = value else {
                return Err(IniEncodeError::new(format!(
                    "ToIni value for key {key:?} in section {section_name:?} must be a String."
                )));
            };
            validate_value(section_name, key, value)?;
            Ok(format!("{key} = {value}"))
        })
        .collect()
}

fn validate_section_name(name: &str) -> Result<(), IniEncodeError> {
    if name
        .chars()
        .any(|ch| matches!(ch, '[' | ']' | '=' | '\n' | '\r'))
    {
        return Err(IniEncodeError::new(format!(
            "ToIni section name {name:?} is not representable in this INI dialect."
        )));
    }
    if name.starts_with(';') || name.starts_with('#') {
        return Err(IniEncodeError::new(format!(
            "ToIni section name {name:?} cannot start with a comment marker."
        )));
    }
    Ok(())
}

fn validate_key_name(name: &str) -> Result<(), IniEncodeError> {
    if name.is_empty()
        || name.trim() != name
        || name
            .chars()
            .any(|ch| matches!(ch, '[' | ']' | '=' | '\n' | '\r'))
        || name.starts_with(';')
        || name.starts_with('#')
    {
        return Err(IniEncodeError::new(format!(
            "ToIni key name {name:?} is not representable in this INI dialect."
        )));
    }
    Ok(())
}

fn validate_value(section: &str, key: &str, value: &str) -> Result<(), IniEncodeError> {
    if value.contains('\n') || value.contains('\r') || value.trim() != value {
        return Err(IniEncodeError::new(format!(
            "ToIni value for key {key:?} in section {section:?} is not representable in this INI dialect."
        )));
    }
    Ok(())
}

fn sections_expr(sections: Vec<IniSection>) -> Expr {
    Expr::call(
        system_expr("Association"),
        sections
            .into_iter()
            .map(|section| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::string(section.name), section_expr(section.entries)],
                )
            })
            .collect(),
    )
}

fn section_expr(entries: Vec<(String, String)>) -> Expr {
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::string(key), Expr::string(value)],
                )
            })
            .collect(),
    )
}

fn trim_start_len(value: &str) -> usize {
    value.len() - value.trim_start().len()
}

fn fail_ini(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "ini", message);
    system_expr("$Failed")
}
