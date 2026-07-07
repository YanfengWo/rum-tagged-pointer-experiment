use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::xml;

pub(crate) const XML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromXml",
        "Parses XML text into a symbolic XmlElement tree, rejecting DTDs and unsupported XML surface.",
        AttributeSet::empty().with(Attribute::Protected),
        xml::eval_from_xml,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToXml",
        "Renders a well-formed XmlElement tree as XML text.",
        AttributeSet::empty().with(Attribute::Protected),
        xml::eval_to_xml,
    ),
    BuiltinSpec::with_attributes(
        "XmlElement",
        "Represents an inert XML element as XmlElement[tag, attrs, children].",
        AttributeSet::empty().with(Attribute::Protected),
    ),
];
