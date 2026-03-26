const SHA256_BLOCK_LEN: usize = 64;
const SHA256_OUTPUT_LEN: usize = 32;

const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub(crate) fn hmac_sha256_hex(key: &str, parts: [&str; 4]) -> String {
    let mut key_block = [0u8; SHA256_BLOCK_LEN];
    let key_bytes = key.as_bytes();

    if key_bytes.len() > SHA256_BLOCK_LEN {
        key_block[..SHA256_OUTPUT_LEN].copy_from_slice(&sha256_bytes(key_bytes));
    } else {
        key_block[..key_bytes.len()].copy_from_slice(key_bytes);
    }

    let mut inner_prefix = [0u8; SHA256_BLOCK_LEN];
    let mut outer_prefix = [0u8; SHA256_BLOCK_LEN];
    for (index, key_byte) in key_block.into_iter().enumerate() {
        inner_prefix[index] = key_byte ^ 0x36;
        outer_prefix[index] = key_byte ^ 0x5c;
    }

    let inner_hash = sha256_many([
        &inner_prefix[..],
        parts[0].as_bytes(),
        parts[1].as_bytes(),
        parts[2].as_bytes(),
        parts[3].as_bytes(),
    ]);
    let digest = sha256_many([&outer_prefix[..], &inner_hash]);

    let mut rendered = String::with_capacity(SHA256_OUTPUT_LEN * 2);
    for byte in digest {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

fn sha256_bytes(bytes: &[u8]) -> [u8; SHA256_OUTPUT_LEN] {
    sha256_many([bytes])
}

fn sha256_many<const N: usize>(chunks: [&[u8]; N]) -> [u8; SHA256_OUTPUT_LEN] {
    let total_len = chunks.iter().map(|chunk| chunk.len()).sum::<usize>();
    let bit_len = (total_len as u64) * 8;
    let padded_len = ((total_len + 9 + 63) / 64) * 64;
    let mut message = Vec::with_capacity(padded_len);
    for chunk in chunks {
        message.extend_from_slice(chunk);
    }
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = SHA256_INITIAL_STATE;
    for block in message.chunks_exact(64) {
        sha256_compress(&mut state, block);
    }

    let mut digest = [0u8; SHA256_OUTPUT_LEN];
    for (index, word) in state.into_iter().enumerate() {
        digest[index * 4..(index + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8]) {
    let mut schedule = [0u32; 64];
    for (index, chunk) in block.chunks_exact(4).take(16).enumerate() {
        schedule[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    for index in 16..64 {
        let s0 = schedule[index - 15].rotate_right(7)
            ^ schedule[index - 15].rotate_right(18)
            ^ (schedule[index - 15] >> 3);
        let s1 = schedule[index - 2].rotate_right(17)
            ^ schedule[index - 2].rotate_right(19)
            ^ (schedule[index - 2] >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(s0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(s1);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for index in 0..64 {
        let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choice = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(SHA256_ROUND_CONSTANTS[index])
            .wrapping_add(schedule[index]);
        let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = sum0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
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
