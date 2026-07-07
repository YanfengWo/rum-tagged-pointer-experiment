use std::collections::BTreeSet;

use rum_runtime::kernel_api::{Attribute, BuiltinSpec, Capability, Session};

use crate::install_core;

pub const REGEN_COMMAND: &str = "cargo run -p rum-stdlib --bin generate-builtin-reference";

pub const LIMITS: &[(&str, &str)] = &[
    (
        "AbsoluteTime",
        "the timestamp contract is exact rational seconds since the Unix epoch 1970-01-01T00:00:00Z; date-list conversion uses UTC, the proleptic Gregorian calendar, and no leap seconds.",
    ),
    (
        "DateDifference",
        "supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.",
    ),
    (
        "DateList",
        "uses UTC, the proleptic Gregorian calendar, and no leap seconds; the zero-argument current-time form requires Capability::Clock.",
    ),
    (
        "DatePlus",
        "supports only fixed-duration units Seconds, Minutes, Hours, Days, and Weeks; calendar Months and Years are outside the folk date/time subset.",
    ),
    (
        "DateString",
        "renders ISO 8601 UTC only; non-integer seconds are rendered only when their exact rational value has a terminating decimal expansion, and the zero-argument current-time form requires Capability::Clock.",
    ),
    (
        "DecimalString",
        "uses exact integer/rational arithmetic only, rounds half away from zero, and always emits exactly n fractional digits.",
    ),
    (
        "DeleteDuplicates",
        "the custom binary-test form performs pairwise O(n^2) representative comparisons and keeps the first representative.",
    ),
    (
        "FileExtension",
        "dotfiles such as .bashrc have no extension; only the final slash-separated path component is inspected.",
    ),
    (
        "FileNameSplit",
        "treats / as the documented separator, emits an absolute leading / as its own component, and skips repeated or trailing separators.",
    ),
    (
        "Md5",
        "intended only for interoperability and checksums; it is not collision-resistant.",
    ),
    (
        "DumpSave",
        "writes a textual dump format; it is not a binary image.",
    ),
    (
        "Factor",
        "handles rational-coefficient polynomial expressions; current factorization extracts monomial factors and recursively splits discovered linear factors.",
    ),
    (
        "FromCsv",
        "parses RFC 4180-style text into String fields only; plain rows may be ragged, while Headers -> True rejects duplicate headers and ragged data rows. Delimiter must be one non-quote, non-newline character.",
    ),
    (
        "FromDateString",
        "accepts only the documented ISO 8601 subset with optional Z or fixed +/-HH:MM offsets; there is no time-zone database and no leap-second support.",
    ),
    (
        "FromIni",
        "implements rum's documented INI dialect only: whole-line ;/# comments, [section] headers, key = value lines, trimmed keys/values, verbatim values, no nesting, no multiline values, and no inline comments.",
    ),
    (
        "FromToml",
        "implements rum's documented TOML 1.0 folk subset only: tables, arrays-of-tables, inline tables, strings, exact integers/decimals, booleans, and date/time tokens; inf/nan and unsupported TOML surface fail closed. Offset date-times decode as exact Unix-second instants, while local dates/times decode as Lists.",
    ),
    (
        "FromXml",
        "parses a value-oriented XML element tree only: declaration syntax is validated but not represented, comments are dropped, CDATA becomes text, namespaces remain verbatim, whitespace-only runs around element content are dropped, and DTD/DOCTYPE declarations are deliberately rejected.",
    ),
    (
        "FromYaml",
        "implements rum's documented YAML sane block subset only: block mappings and sequences, plain and quoted scalars, literal | and |- blocks, comments, and empty flow {} / []; anchors, aliases, tags, directives, document markers, merge keys, folded blocks, non-empty flow collections, duplicate keys, tabs in indentation, and ambiguous same-indent collection mixes fail closed.",
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
    (
        "RandomBytes",
        "draws directly from the host entropy provider, not from the deterministic session PRNG.",
    ),
    (
        "RandomChoice",
        "uses the deterministic session PRNG; it is not cryptographic.",
    ),
    (
        "RandomInteger",
        "uses the deterministic session PRNG with rejection sampling for unbiased exact integers; it is not cryptographic. There is no RandomReal in the current exact-number boundary.",
    ),
    (
        "RandomPassword",
        "uses the host entropy provider directly with rejection sampling over the alphabet size to avoid modulo bias; it is not affected by SeedRandom or BlockRandom. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw. RandomPassword[n, opts...] and RandomPassword[alphabet, n, opts...] accept string-keyed policy rules \"Require\" -> {group, ...}, \"NoAdjacentRepeats\" -> True|False, \"Exclude\" -> {substr, ...}, and \"IgnoreCase\" -> True|False. Require groups are exact character sets and must be non-empty subsets of the alphabet; IgnoreCase affects only Exclude matching. NoAdjacentRepeats forbids identical adjacent characters only, not sequential runs. Policy sampling draws whole candidate passwords uniformly from alphabet^n and accepts only candidates satisfying every constraint, so accepted outputs are uniform over the satisfying set. Unsatisfied policies fail closed after 10000 attempts.",
    ),
    (
        "RandomSample",
        "uses the deterministic session PRNG; it is not cryptographic.",
    ),
    (
        "RandomString",
        "uses the deterministic session PRNG with rejection sampling over the alphabet size; it is not cryptographic. The default alphabet is the 62-character A-Za-z0-9 set. Alphabet strings are read as code points, and duplicate code points weight the draw.",
    ),
    (
        "RandomUuid",
        "uses the deterministic session PRNG and is reproducible under the same seed; use an entropy reseed first when global uniqueness matters.",
    ),
    (
        "SeedRandom",
        "uses xoshiro256** with splitmix64 seed expansion; exact integer seeds fold magnitude bytes through splitmix64, with a sign tag for negative seeds. The zero-argument entropy form requires Capability::Random.",
    ),
    (
        "Sha1",
        "intended only for interoperability and checksums; it is not collision-resistant.",
    ),
    (
        "ShellSplit",
        "implements only whitespace splitting, single and double quotes, and backslash escapes outside single quotes; expansion, globbing, command substitution, and short-option parsing are outside this text utility.",
    ),
    (
        "ToCsv",
        "renders only String fields from list rows or String-keyed Association rows; Association rows imply a header row and must share the same keys. Delimiter must be one non-quote, non-newline character.",
    ),
    (
        "ToIni",
        "renders only Association-of-Associations with String section names, keys, and values; unrepresentable names, multiline values, whitespace-trimming value loss, and an empty default section fail closed.",
    ),
    (
        "Tally",
        "the custom binary-test form performs pairwise O(n^2) representative comparisons and uses the first element of each class as representative.",
    ),
    (
        "ToToml",
        "renders only top-level Associations with String keys, exact integers, terminating rationals, booleans, strings, Lists, nested Associations, and Lists of Associations; nested Associations use sub-table headers unless inline form is required to preserve same-level key order. It never synthesizes TOML datetime syntax, so date Lists and timestamps encode as arrays or numbers.",
    ),
    (
        "ToXml",
        "renders only well-formed XmlElement[tag_String, attrs_Association, children_List] trees with String attributes and String/XmlElement children; names use the documented ASCII XML-name subset, compact output is default, and Indent pretty-prints only element-only content.",
    ),
    (
        "ToYaml",
        "renders only Associations with String keys, Lists, exact integers, terminating rationals, True, False, Null, and Strings; safe plain strings use the documented Norway-safe rule, other strings are double-quoted or literal blocks, and unsupported YAML surface is never synthesized.",
    ),
    (
        "StringDedent",
        "computes the common leading whitespace from non-blank lines; blank and whitespace-only lines do not determine the prefix.",
    ),
    (
        "StringForm",
        "returns a String eagerly rather than a lazy display wrapper; a backslash before a backtick emits a literal backtick, and missing or malformed placeholders emit StringForm::slot and leave the call inert.",
    ),
    (
        "StringIndent",
        "does not prefix empty or whitespace-only lines.",
    ),
    (
        "StringWrap",
        "normalizes input whitespace runs to single spaces while wrapping; it does not preserve original spacing inside wrapped text.",
    ),
    (
        "PadLeft",
        "does not support the margin argument; list dimensions apply to rectangular nested lists only.",
    ),
    (
        "PadRight",
        "does not support the margin argument; list dimensions apply to rectangular nested lists only.",
    ),
    (
        "Partition",
        "supports flat positive scalar size/offset forms; aligned forms use positive {kL, kR}, prepend kL - 1 fill positions, stop at the first d-stepped block where the final item appears at position >= kR, use cyclic list padding when padding is omitted, and use ragged no-padding output for {}.",
    ),
    (
        "Riffle",
        "supports one separator with optional positive interval or {imin, imax, di} insertion-position specs; list-separator forms are outside the current contract.",
    ),
    (
        "StringLines",
        "splits only on LF and CRLF; one trailing newline is discarded while interior empty lines are preserved.",
    ),
    (
        "UrlParse",
        "without \"DefaultScheme\", accepts only a strict scheme://host[:port][/path][?query][#fragment] shape with an ASCII scheme and decimal port; with \"DecodeQuery\" -> True, query text is decoded as application/x-www-form-urlencoded data.",
    ),
    (
        "ParseArgs",
        "recognizes --key=value options, bare --flag flags, and -- as the end of option parsing; \"ShortOptions\" -> True recognizes single-dash flag clusters but does not support attached values.",
    ),
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
        Capability::Random => "Random",
        Capability::Diagnostic => "Diagnostic",
        Capability::Process => "Process",
    }
}
