use std::collections::BTreeSet;

use rum_runtime::kernel_api::{Attribute, BuiltinSpec, Capability, Session};

use crate::install_core;

pub const REGEN_COMMAND: &str = "cargo run -p rum-stdlib --bin generate-builtin-reference";

pub const LIMITS: &[(&str, &str)] = &[
    (
        "DumpSave",
        "writes a textual dump format; it is not a binary image.",
    ),
    (
        "Factor",
        "handles rational-coefficient polynomial expressions; current factorization extracts monomial factors and recursively splits discovered linear factors.",
    ),
    (
        "Prime",
        "uses a bounded integer-work budget and sieve-based prime generation for supported exact positive integer indices.",
    ),
    (
        "PrimePi",
        "uses a bounded integer-work budget and sieve-based counting after flooring exact rational inputs.",
    ),
    ("PrimeQ", "uses direct trial division over exact integers."),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinReference {
    pub markdown: String,
    pub entry_count: usize,
}

pub fn generate_builtin_reference() -> Result<BuiltinReference, String> {
    let specs = installed_builtin_specs();
    validate_limits(&specs)?;

    let mut markdown = String::new();
    push_header(&mut markdown, specs.len());

    for spec in &specs {
        push_entry(&mut markdown, spec)?;
    }

    Ok(BuiltinReference {
        markdown,
        entry_count: specs.len(),
    })
}

pub fn installed_builtin_specs() -> Vec<BuiltinSpec> {
    let mut session = Session::new();
    install_core(&mut session);

    let mut specs = session
        .registered_builtin_specs()
        .copied()
        .collect::<Vec<_>>();
    specs.sort_by(|left, right| left.name.cmp(right.name));
    specs
}

pub fn validate_limits(specs: &[BuiltinSpec]) -> Result<(), String> {
    let names = specs.iter().map(|spec| spec.name).collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();

    for (name, _) in LIMITS {
        if !seen.insert(*name) {
            return Err(format!("duplicate limits note for builtin `{name}`"));
        }

        if !names.contains(name) {
            return Err(format!(
                "limits note names `{name}`, but no installed builtin has that name"
            ));
        }
    }

    Ok(())
}

fn push_header(markdown: &mut String, entry_count: usize) {
    markdown.push_str("# Builtin Reference\n\n");
    markdown
        .push_str("Generated from the installed `rum` builtin registry. Do not edit by hand.\n\n");
    markdown.push_str(&format!("Builtin count: {entry_count}\n\n"));
    markdown.push_str(&format!("Regenerate with `{REGEN_COMMAND}`.\n\n"));
}

fn push_entry(markdown: &mut String, spec: &BuiltinSpec) -> Result<(), String> {
    markdown.push_str(&format!("### {}\n\n", spec.name));
    markdown.push_str(spec.summary);
    markdown.push_str("\n\n");
    markdown.push_str(&format!(
        "- Attributes: {}\n",
        render_attributes(spec.attributes)
    ));
    markdown.push_str(&format!(
        "- Capability: `{}`\n",
        render_capability(spec.capability)
    ));

    if let Some((_, note)) = LIMITS.iter().find(|(name, _)| *name == spec.name) {
        markdown.push_str(&format!("- Limits: {note}\n"));
    }

    markdown.push('\n');
    Ok(())
}

fn render_attributes(attributes: rum_runtime::kernel_api::AttributeSet) -> String {
    let names = attributes.iter().map(render_attribute).collect::<Vec<_>>();

    if names.is_empty() {
        "`None`".to_string()
    } else {
        names
            .into_iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn render_attribute(attribute: Attribute) -> &'static str {
    match attribute {
        Attribute::HoldAllComplete => "HoldAllComplete",
        Attribute::HoldAll => "HoldAll",
        Attribute::HoldFirst => "HoldFirst",
        Attribute::HoldRest => "HoldRest",
        Attribute::SequenceHold => "SequenceHold",
        Attribute::Listable => "Listable",
        Attribute::NHoldAll => "NHoldAll",
        Attribute::NumericFunction => "NumericFunction",
        Attribute::Orderless => "Orderless",
        Attribute::Flat => "Flat",
        Attribute::OneIdentity => "OneIdentity",
        Attribute::NonThreadable => "NonThreadable",
        Attribute::Protected => "Protected",
        Attribute::ReadProtected => "ReadProtected",
        Attribute::Temporary => "Temporary",
    }
}

fn render_capability(capability: Capability) -> &'static str {
    match capability {
        Capability::Pure => "Pure",
        Capability::FileRead => "FileRead",
        Capability::FileWrite => "FileWrite",
        Capability::Clock => "Clock",
        Capability::Diagnostic => "Diagnostic",
    }
}
