use rum_core::Expr;
use rum_runtime::kernel_api::system_expr;

use super::TomlParseError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TableKind {
    Implicit,
    Explicit,
    Inline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TomlTable {
    kind: TableKind,
    entries: Vec<(String, TomlValue)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TomlValue {
    Scalar(Expr),
    Array(Vec<TomlValue>),
    Table(TomlTable),
    ArrayOfTables(Vec<TomlTable>),
}

impl TomlTable {
    pub(super) fn root() -> Self {
        Self {
            kind: TableKind::Explicit,
            entries: Vec::new(),
        }
    }

    pub(super) fn implicit() -> Self {
        Self {
            kind: TableKind::Implicit,
            entries: Vec::new(),
        }
    }

    pub(super) fn explicit() -> Self {
        Self {
            kind: TableKind::Explicit,
            entries: Vec::new(),
        }
    }

    pub(super) fn inline() -> Self {
        Self {
            kind: TableKind::Inline,
            entries: Vec::new(),
        }
    }

    pub(super) fn entry_index(&self, key: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|(existing, _)| existing == key)
    }

    pub(super) fn push_entry(&mut self, key: String, value: TomlValue) {
        self.entries.push((key, value));
    }
}

pub(super) fn define_table_path(
    table: &mut TomlTable,
    path: &[String],
    offset: usize,
) -> Result<(), TomlParseError> {
    let Some((first, rest)) = path.split_first() else {
        return Err(TomlParseError::new(offset, "empty table path"));
    };

    if rest.is_empty() {
        match table.entry_index(first) {
            None => {
                table.push_entry(first.clone(), TomlValue::Table(TomlTable::explicit()));
                Ok(())
            }
            Some(index) => match &mut table.entries[index].1 {
                TomlValue::Table(child) if child.kind == TableKind::Implicit => {
                    child.kind = TableKind::Explicit;
                    Ok(())
                }
                TomlValue::Table(child) if child.kind == TableKind::Inline => Err(
                    TomlParseError::new(offset, format!("cannot redefine inline table {first:?}")),
                ),
                TomlValue::Table(_) => Err(TomlParseError::new(
                    offset,
                    format!("duplicate table {first:?}"),
                )),
                TomlValue::ArrayOfTables(_) => Err(TomlParseError::new(
                    offset,
                    format!("cannot redefine array-of-tables {first:?} as a table"),
                )),
                _ => Err(TomlParseError::new(
                    offset,
                    format!("dotted-key conflict at {first:?}"),
                )),
            },
        }
    } else {
        let child = table_for_path_component_mut(table, first, offset)?;
        define_table_path(child, rest, offset)
    }
}

pub(super) fn add_array_table_path(
    table: &mut TomlTable,
    path: &[String],
    offset: usize,
) -> Result<(), TomlParseError> {
    let Some((first, rest)) = path.split_first() else {
        return Err(TomlParseError::new(offset, "empty array-of-tables path"));
    };

    if rest.is_empty() {
        match table.entry_index(first) {
            None => {
                table.push_entry(
                    first.clone(),
                    TomlValue::ArrayOfTables(vec![TomlTable::explicit()]),
                );
                Ok(())
            }
            Some(index) => match &mut table.entries[index].1 {
                TomlValue::ArrayOfTables(items) => {
                    items.push(TomlTable::explicit());
                    Ok(())
                }
                TomlValue::Table(child) if child.kind == TableKind::Inline => Err(
                    TomlParseError::new(offset, format!("cannot redefine inline table {first:?}")),
                ),
                TomlValue::Table(_) => Err(TomlParseError::new(
                    offset,
                    format!("cannot redefine table {first:?} as an array-of-tables"),
                )),
                _ => Err(TomlParseError::new(
                    offset,
                    format!("dotted-key conflict at {first:?}"),
                )),
            },
        }
    } else {
        let child = table_for_path_component_mut(table, first, offset)?;
        add_array_table_path(child, rest, offset)
    }
}

fn table_for_path_component_mut<'a>(
    table: &'a mut TomlTable,
    key: &str,
    offset: usize,
) -> Result<&'a mut TomlTable, TomlParseError> {
    let index = match table.entry_index(key) {
        Some(index) => index,
        None => {
            table.push_entry(key.to_string(), TomlValue::Table(TomlTable::implicit()));
            table.entries.len() - 1
        }
    };

    match &mut table.entries[index].1 {
        TomlValue::Table(child) if child.kind == TableKind::Inline => Err(TomlParseError::new(
            offset,
            format!("cannot redefine inline table {key:?}"),
        )),
        TomlValue::Table(child) => Ok(child),
        TomlValue::ArrayOfTables(items) => items
            .last_mut()
            .ok_or_else(|| TomlParseError::new(offset, "empty array-of-tables")),
        _ => Err(TomlParseError::new(
            offset,
            format!("dotted-key conflict at {key:?}"),
        )),
    }
}

pub(super) fn table_at_path_mut<'a>(
    table: &'a mut TomlTable,
    path: &[String],
    offset: usize,
) -> Result<&'a mut TomlTable, TomlParseError> {
    let Some((first, rest)) = path.split_first() else {
        return Ok(table);
    };
    let Some(index) = table.entry_index(first) else {
        return Err(TomlParseError::new(
            offset,
            format!("missing current table {first:?}"),
        ));
    };
    match &mut table.entries[index].1 {
        TomlValue::Table(child) => table_at_path_mut(child, rest, offset),
        TomlValue::ArrayOfTables(items) => {
            let child = items
                .last_mut()
                .ok_or_else(|| TomlParseError::new(offset, "empty array-of-tables"))?;
            table_at_path_mut(child, rest, offset)
        }
        _ => Err(TomlParseError::new(
            offset,
            format!("current path component {first:?} is not a table"),
        )),
    }
}

pub(super) fn insert_value_path(
    table: &mut TomlTable,
    path: &[String],
    value: TomlValue,
    offset: usize,
) -> Result<(), TomlParseError> {
    let Some((first, rest)) = path.split_first() else {
        return Err(TomlParseError::new(offset, "empty key path"));
    };

    if rest.is_empty() {
        if table.entry_index(first).is_some() {
            return Err(TomlParseError::new(
                offset,
                format!("duplicate key {first:?}"),
            ));
        }
        table.push_entry(first.clone(), value);
        return Ok(());
    }

    let child = table_for_path_component_mut(table, first, offset)?;
    insert_value_path(child, rest, value, offset)
}

pub(super) fn table_expr(table: TomlTable) -> Expr {
    Expr::call(
        system_expr("Association"),
        table
            .entries
            .into_iter()
            .map(|(key, value)| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::string(key), value_expr(value)],
                )
            })
            .collect(),
    )
}

fn value_expr(value: TomlValue) -> Expr {
    match value {
        TomlValue::Scalar(expr) => expr,
        TomlValue::Array(items) => list_expr(items.into_iter().map(value_expr).collect()),
        TomlValue::Table(table) => table_expr(table),
        TomlValue::ArrayOfTables(tables) => list_expr(tables.into_iter().map(table_expr).collect()),
    }
}

pub(super) fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

pub(super) fn bare_key_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
