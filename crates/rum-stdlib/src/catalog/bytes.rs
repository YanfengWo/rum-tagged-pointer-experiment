use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::bytes;

#[cfg(feature = "hash")]
pub(crate) const DIGEST_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Sha256",
        "Computes a SHA-256 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_sha256,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sha512",
        "Computes a SHA-512 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_sha512,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sha1",
        "Computes a SHA-1 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_sha1,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Md5",
        "Computes an MD5 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_md5,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Crc32",
        "Computes the IEEE CRC-32 checksum over a string's UTF-8 bytes or a ByteArray.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_crc32,
    ),
];

pub(crate) const BYTE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "ToHexString",
        "Encodes a string's UTF-8 bytes or a ByteArray as lowercase hexadecimal text.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_to_hex_string,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromHexString",
        "Decodes even-length hexadecimal text into a ByteArray, failing closed on malformed input.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_from_hex_string,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToBase64",
        "Encodes a string's UTF-8 bytes or a ByteArray as standard RFC 4648 base64 with padding.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_to_base64,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromBase64",
        "Decodes standard RFC 4648 base64 with padding into a ByteArray, failing closed on malformed input.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_from_base64,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ByteArrayQ",
        "Tests whether an expression is a ByteArray with a valid base64 payload.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_byte_array_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ByteArrayToString",
        "Converts a ByteArray to a string when its payload is valid UTF-8, failing closed otherwise.",
        AttributeSet::empty().with(Attribute::Protected),
        bytes::eval_byte_array_to_string,
    ),
];
