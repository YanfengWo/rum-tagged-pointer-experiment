use hmac::{Hmac, Mac};
use sha2::Sha256;

pub(crate) fn hmac_sha256_hex(key: &str, parts: [&str; 4]) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC accepts keys of any length");
    for part in parts {
        mac.update(part.as_bytes());
    }
    let digest = mac.finalize().into_bytes();

    let mut rendered = String::with_capacity(digest.len() * 2);
    for byte in digest {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => unreachable!("nibbles are always in 0..=15"),
    }
}

#[cfg(test)]
mod tests {
    use super::hmac_sha256_hex;

    #[test]
    fn hmac_sha256_matches_known_test_vector() {
        assert_eq!(
            hmac_sha256_hex(
                "key",
                ["The ", "quick ", "brown fox ", "jumps over the lazy dog"]
            ),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }
}
