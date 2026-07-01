use rum_runtime::{
    DefinitionCatalogEntry, SessionDefinitionSummary, SessionInspectResponse, SymbolInfo,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectResponse {
    pub target: Option<String>,
    pub found: bool,
    pub info: Option<HostSymbolInfo>,
    pub definitions: Vec<HostDefinition>,
    pub message_catalog: Vec<HostMessageTemplate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSymbolInfo {
    pub qualified_name: String,
    pub context: String,
    pub display_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub attributes: Vec<String>,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_up_values: bool,
    pub has_down_values: bool,
    pub has_sub_values: bool,
    pub message_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDefinition {
    pub kind: String,
    pub lhs: String,
    pub rhs: String,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDefinitionSummary {
    pub name: String,
    pub has_own_value: bool,
    pub down_value_count: usize,
    pub up_value_count: usize,
    pub sub_value_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMessageTemplate {
    pub message_name: String,
    pub qualified_symbol_name: String,
    pub tag: String,
    pub template: String,
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
