use rum_runtime::{
    DefinitionCatalogEntry, SessionDefinitionSummary, SessionInspectResponse, SymbolInfo,
};

/// Response from symbol inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectResponse {
    /// Requested symbol target, when one was identified.
    pub target: Option<String>,
    /// Whether the target resolved to inspectable symbol information.
    pub found: bool,
    /// Symbol metadata when `found` is true.
    pub info: Option<HostSymbolInfo>,
    /// Definitions associated with the symbol.
    pub definitions: Vec<HostDefinition>,
    /// Message templates associated with the symbol.
    pub message_catalog: Vec<HostMessageTemplate>,
}

/// Metadata about a runtime symbol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSymbolInfo {
    /// Fully qualified symbol name.
    pub qualified_name: String,
    /// Context that owns the symbol.
    pub context: String,
    /// Display name suitable for host UI surfaces.
    pub display_name: String,
    /// Optional builtin summary text.
    pub summary: Option<String>,
    /// Whether the symbol is provided by the installed builtin catalog.
    pub is_builtin: bool,
    /// Attribute names currently attached to the symbol.
    pub attributes: Vec<String>,
    /// Whether the symbol has an own value.
    pub has_own_value: bool,
    /// Whether the symbol has a delayed own value.
    pub has_delayed_value: bool,
    /// Whether the symbol has up values.
    pub has_up_values: bool,
    /// Whether the symbol has down values.
    pub has_down_values: bool,
    /// Whether the symbol has sub values.
    pub has_sub_values: bool,
    /// Message tags defined for the symbol.
    pub message_tags: Vec<String>,
}

/// Host-facing rendering of a stored definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDefinition {
    /// Definition kind.
    pub kind: String,
    /// Rendered left-hand side.
    pub lhs: String,
    /// Rendered right-hand side.
    pub rhs: String,
    /// Whether the right-hand side is delayed.
    pub delayed: bool,
}

/// Compact summary of definitions attached to a symbol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDefinitionSummary {
    /// Symbol name.
    pub name: String,
    /// Whether the symbol has an own value.
    pub has_own_value: bool,
    /// Number of down values attached to the symbol.
    pub down_value_count: usize,
    /// Number of up values attached to the symbol.
    pub up_value_count: usize,
    /// Number of sub values attached to the symbol.
    pub sub_value_count: usize,
}

/// Host-facing rendering of a message template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMessageTemplate {
    /// Fully qualified message name.
    pub message_name: String,
    /// Fully qualified symbol name that owns the message.
    pub qualified_symbol_name: String,
    /// Message tag.
    pub tag: String,
    /// Rendered message template.
    pub template: String,
    /// Whether the template is delayed.
    pub delayed: bool,
}

impl From<SymbolInfo> for HostSymbolInfo {
    fn from(info: SymbolInfo) -> Self {
        Self {
            qualified_name: info.qualified_name,
            context: info.context,
            display_name: info.display_name,
            summary: info.summary,
            is_builtin: info.is_builtin,
            attributes: info.attributes,
            has_own_value: info.has_own_value,
            has_delayed_value: info.has_delayed_value,
            has_up_values: info.has_up_values,
            has_down_values: info.has_down_values,
            has_sub_values: info.has_sub_values,
            message_tags: info.message_tags,
        }
    }
}

impl From<DefinitionCatalogEntry> for HostDefinition {
    fn from(entry: DefinitionCatalogEntry) -> Self {
        Self {
            kind: format!("{:?}", entry.kind),
            lhs: entry.lhs.to_string(),
            rhs: entry.rhs.to_string(),
            delayed: entry.delayed,
        }
    }
}

impl From<SessionDefinitionSummary> for HostDefinitionSummary {
    fn from(summary: SessionDefinitionSummary) -> Self {
        Self {
            name: summary.name,
            has_own_value: summary.has_own_value,
            down_value_count: summary.down_value_count,
            up_value_count: summary.up_value_count,
            sub_value_count: summary.sub_value_count,
        }
    }
}

pub(crate) fn inspect_response_from_runtime(
    target: Option<String>,
    response: Option<SessionInspectResponse>,
) -> HostInspectResponse {
    let Some(response) = response else {
        return HostInspectResponse {
            target,
            found: false,
            info: None,
            definitions: Vec::new(),
            message_catalog: Vec::new(),
        };
    };

    HostInspectResponse {
        target,
        found: true,
        info: Some(response.info.into()),
        definitions: response.definitions.into_iter().map(Into::into).collect(),
        message_catalog: response
            .message_catalog
            .into_iter()
            .map(|entry| HostMessageTemplate {
                message_name: entry.message_name,
                qualified_symbol_name: entry.qualified_symbol_name,
                tag: entry.tag,
                template: entry.template.to_string(),
                delayed: entry.delayed,
            })
            .collect(),
    }
}
