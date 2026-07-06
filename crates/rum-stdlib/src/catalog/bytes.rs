use rum_runtime::kernel_api::BuiltinSpec;

use crate::bytes;

pub(crate) const BYTE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Sha256",
        "Computes a SHA-256 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        bytes::eval_sha256,
    ),
    BuiltinSpec::ordinary(
        "Sha512",
        "Computes a SHA-512 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        bytes::eval_sha512,
    ),
    BuiltinSpec::ordinary(
        "Sha1",
        "Computes a SHA-1 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        bytes::eval_sha1,
    ),
    BuiltinSpec::ordinary(
        "Md5",
        "Computes an MD5 digest over a string's UTF-8 bytes or a ByteArray, returning lowercase hex by default.",
        bytes::eval_md5,
    ),
    BuiltinSpec::ordinary(
        "Crc32",
        "Computes the IEEE CRC-32 checksum over a string's UTF-8 bytes or a ByteArray.",
        bytes::eval_crc32,
    ),
    BuiltinSpec::ordinary(
        "ToHexString",
        "Encodes a string's UTF-8 bytes or a ByteArray as lowercase hexadecimal text.",
        bytes::eval_to_hex_string,
    ),
    BuiltinSpec::ordinary(
        "FromHexString",
        "Decodes even-length hexadecimal text into a ByteArray, failing closed on malformed input.",
        bytes::eval_from_hex_string,
    ),
    BuiltinSpec::ordinary(
        "ToBase64",
        "Encodes a string's UTF-8 bytes or a ByteArray as standard RFC 4648 base64 with padding.",
        bytes::eval_to_base64,
    ),
    BuiltinSpec::ordinary(
        "FromBase64",
        "Decodes standard RFC 4648 base64 with padding into a ByteArray, failing closed on malformed input.",
        bytes::eval_from_base64,
    ),
    BuiltinSpec::ordinary(
        "ByteArrayQ",
        "Tests whether an expression is a ByteArray with a valid base64 payload.",
        bytes::eval_byte_array_q,
    ),
    BuiltinSpec::ordinary(
        "ByteArrayToString",
        "Converts a ByteArray to a string when its payload is valid UTF-8, failing closed otherwise.",
        bytes::eval_byte_array_to_string,
    ),
];
