use std::sync::OnceLock;

use super::*;

type Spec = BuiltinAttributeSpec;

mod specs_a_e;
mod specs_f_m;
mod specs_n_r;
mod specs_s_z;

const ATTR_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_FLAT_ONE_IDENTITY_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_HOLD_ALL: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldAll);

const ATTR_HOLD_ALL_COMPLETE_PROTECTED: AttributeSet =
    AttributeSet::from_attribute(Attribute::HoldAllComplete).with(Attribute::Protected);

const ATTR_HOLD_ALL_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected);

const ATTR_HOLD_ALL_PROTECTED_READ_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldAll)
        .with(Attribute::Protected)
        .with(Attribute::ReadProtected);

const ATTR_HOLD_FIRST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldFirst);

const ATTR_HOLD_REST: AttributeSet = AttributeSet::from_hold(HoldAttribute::HoldRest);

const ATTR_HOLD_REST_PROTECTED: AttributeSet =
    AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected);

const ATTR_LISTABLE: AttributeSet = AttributeSet::empty().with(Attribute::Listable);

const ATTR_LISTABLE_N_HOLD_ALL_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NHoldAll)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_ORDERLESS_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Orderless)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected);

const ATTR_LISTABLE_NUMERIC_FUNCTION_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_LISTABLE_ORDERLESS_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Orderless)
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_LISTABLE_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Protected);

const ATTR_LISTABLE_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Listable)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const ATTR_NON_THREADABLE_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::NonThreadable)
    .with(Attribute::Protected);

const ATTR_NUMERIC_FUNCTION_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::NumericFunction)
    .with(Attribute::Protected);

const ATTR_ORDERLESS: AttributeSet = AttributeSet::empty().with(Attribute::Orderless);

const ATTR_ORDERLESS_FLAT_ONE_IDENTITY: AttributeSet = AttributeSet::empty()
    .with(Attribute::Orderless)
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity);

const ATTR_PROTECTED: AttributeSet = AttributeSet::empty().with(Attribute::Protected);

const ATTR_PROTECTED_READ_PROTECTED: AttributeSet = AttributeSet::empty()
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

const STDLIB_EMPTY_ATTRIBUTE_NAMES: &[&str] = &["$AssertFunction", "Decrypt", "Encrypt"];

// These helpers are used by the compile-time assertions below. The allows are
// needed because dead-code analysis does not root usage from anonymous consts.
#[allow(dead_code)]
const fn str_ascii_lt(lhs: &str, rhs: &str) -> bool {
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    let shorter = if lhs.len() < rhs.len() {
        lhs.len()
    } else {
        rhs.len()
    };
    let mut index = 0;
    while index < shorter {
        if lhs[index] != rhs[index] {
            return lhs[index] < rhs[index];
        }
        index += 1;
    }
    lhs.len() < rhs.len()
}

#[allow(dead_code)]
const fn attribute_specs_strictly_sorted(specs: &[Spec]) -> bool {
    let mut index = 1;
    while index < specs.len() {
        if !str_ascii_lt(specs[index - 1].name, specs[index].name) {
            return false;
        }
        index += 1;
    }
    true
}

#[allow(dead_code)]
const fn attribute_specs_within_byte_range(specs: &[Spec], lo: u8, hi: u8) -> bool {
    let mut index = 0;
    while index < specs.len() {
        let name = specs[index].name.as_bytes();
        if name.is_empty() || name[0] < lo || name[0] > hi {
            return false;
        }
        index += 1;
    }
    true
}

#[allow(dead_code)]
const fn attribute_names_strictly_sorted(names: &[&str]) -> bool {
    let mut index = 1;
    while index < names.len() {
        if !str_ascii_lt(names[index - 1], names[index]) {
            return false;
        }
        index += 1;
    }
    true
}

// These tables are binary-searched by ASCII byte order (str::cmp), so an
// out-of-place insert silently breaks lookups near it. Strict sorting within
// each non-empty chunk plus membership in disjoint, exhaustive first-byte
// ranges is equivalent to strict global sorting: concatenating the chunks in
// range order is globally sorted, and duplicate names cannot span chunks.
const _: () = assert!(
    attribute_specs_strictly_sorted(specs_a_e::SPECS),
    "specs_a_e::SPECS must stay strictly ASCII-sorted (binary-searched)"
);
const _: () = assert!(
    attribute_specs_strictly_sorted(specs_f_m::SPECS),
    "specs_f_m::SPECS must stay strictly ASCII-sorted (binary-searched)"
);
const _: () = assert!(
    attribute_specs_strictly_sorted(specs_n_r::SPECS),
    "specs_n_r::SPECS must stay strictly ASCII-sorted (binary-searched)"
);
const _: () = assert!(
    attribute_specs_strictly_sorted(specs_s_z::SPECS),
    "specs_s_z::SPECS must stay strictly ASCII-sorted (binary-searched)"
);
const _: () = assert!(
    attribute_specs_within_byte_range(specs_a_e::SPECS, u8::MIN, b'E'),
    "specs_a_e::SPECS names must start below 'F'"
);
const _: () = assert!(
    attribute_specs_within_byte_range(specs_f_m::SPECS, b'F', b'M'),
    "specs_f_m::SPECS names must start in 'F'..='M'"
);
const _: () = assert!(
    attribute_specs_within_byte_range(specs_n_r::SPECS, b'N', b'R'),
    "specs_n_r::SPECS names must start in 'N'..='R'"
);
const _: () = assert!(
    attribute_specs_within_byte_range(specs_s_z::SPECS, b'S', u8::MAX),
    "specs_s_z::SPECS names must start at or above 'S'"
);
const _: () = assert!(
    attribute_names_strictly_sorted(STDLIB_EMPTY_ATTRIBUTE_NAMES),
    "STDLIB_EMPTY_ATTRIBUTE_NAMES must stay strictly ASCII-sorted (binary-searched)"
);

const fn chunk_for_name(name: &str) -> &'static [Spec] {
    match name.as_bytes() {
        [] => &[],
        [first, ..] if *first < b'F' => specs_a_e::SPECS,
        [first, ..] if *first <= b'M' => specs_f_m::SPECS,
        [first, ..] if *first <= b'R' => specs_n_r::SPECS,
        _ => specs_s_z::SPECS,
    }
}

pub(super) fn stdlib_builtin_attribute_specs() -> &'static [BuiltinAttributeSpec] {
    static SPECS: OnceLock<Vec<BuiltinAttributeSpec>> = OnceLock::new();

    SPECS
        .get_or_init(|| {
            let mut specs = Vec::with_capacity(
                specs_a_e::SPECS.len()
                    + specs_f_m::SPECS.len()
                    + specs_n_r::SPECS.len()
                    + specs_s_z::SPECS.len()
                    + STDLIB_EMPTY_ATTRIBUTE_NAMES.len(),
            );
            specs.extend_from_slice(specs_a_e::SPECS);
            specs.extend_from_slice(specs_f_m::SPECS);
            specs.extend_from_slice(specs_n_r::SPECS);
            specs.extend_from_slice(specs_s_z::SPECS);
            specs.extend(
                STDLIB_EMPTY_ATTRIBUTE_NAMES
                    .iter()
                    .copied()
                    .map(|name| Spec::new(name, AttributeSet::empty())),
            );
            specs.sort_unstable_by(|lhs, rhs| lhs.name.cmp(rhs.name));
            specs
        })
        .as_slice()
}

pub(super) fn default_stdlib_attribute_set_for_symbol(symbol: &Symbol) -> Option<AttributeSet> {
    if !symbol.is_system() {
        return None;
    }

    let name = symbol.name();
    let specs = chunk_for_name(name);
    if let Ok(index) = specs.binary_search_by(|spec| spec.name.cmp(name)) {
        return Some(specs[index].attributes);
    }
    if STDLIB_EMPTY_ATTRIBUTE_NAMES
        .binary_search_by(|candidate| candidate.cmp(&name))
        .is_ok()
    {
        return Some(AttributeSet::empty());
    }

    None
}
