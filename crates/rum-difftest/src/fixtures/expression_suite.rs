#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionSuiteEntry {
    pub category: Option<String>,
    pub threshold: Option<String>,
    pub expr: String,
}

pub fn parse_expression_suite(text: &str) -> Vec<ExpressionSuiteEntry> {
    let mut entries = Vec::new();
    let mut current_category = None;
    let mut current_threshold = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') && !line.starts_with("# @") {
            continue;
        }

        if let Some(value) = line.strip_prefix("# @category:") {
            current_category = Some(value.trim().to_owned());
            continue;
        }

        if let Some(value) = line.strip_prefix("# @threshold:") {
            current_threshold = Some(value.trim().to_owned());
            continue;
        }

        entries.push(ExpressionSuiteEntry {
            category: current_category.clone(),
            threshold: current_threshold.clone(),
            expr: line.to_owned(),
        });
    }

    entries
}
