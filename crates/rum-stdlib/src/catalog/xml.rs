use rum_runtime::kernel_api::BuiltinSpec;

use crate::xml;

pub(crate) const XML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FromXml",
        "Parses XML text into a symbolic XmlElement tree, rejecting DTDs and unsupported XML surface.",
        xml::eval_from_xml,
    ),
    BuiltinSpec::ordinary(
        "ToXml",
        "Renders a well-formed XmlElement tree as XML text.",
        xml::eval_to_xml,
    ),
    BuiltinSpec::new(
        "XmlElement",
        "Represents an inert XML element as XmlElement[tag, attrs, children].",
    ),
];
