use super::text::render_input_like_expr;
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_string_to_byte_array(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringToByteArray", args, 1, 2));
        }

        if let Some(encoding) = args.get(1).and_then(expr_as_string) {
            if encoding != "UTF8" {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        } else if args.len() == 2 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(byte_array_expr(value.as_bytes()));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| byte_array_expr(value.as_bytes()))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_hash(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Hash", args, 1, 3));
        }

        if let Some(algorithm_expr) = args.get(1) {
            let Some(algorithm) = expr_as_string(algorithm_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if algorithm != "SHA256" {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        }

        let digest = sha256_bytes(&hash_input_bytes(&args[0]));
        let integer_value = sha256_integer(&digest);

        match args.get(2).and_then(|format| expr_as_string(format)) {
            None => Ok(Expr::integer(integer_value)),
            Some("Integer") => Ok(Expr::integer(integer_value)),
            Some("HexString") => Ok(Expr::string(bytes_to_hex(&digest))),
            Some("DecimalString") => Ok(Expr::string(sha256_decimal_string(&integer_value))),
            Some("ByteArray") => Ok(byte_array_expr(&digest)),
            Some(_) => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(in crate::runtime) fn eval_compress(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Compress", args, 1));
        }

        Ok(Expr::string(compress_envelope(&args[0])))
    }

    pub(in crate::runtime) fn eval_encrypt(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Encrypt", args, 2));
        }

        let Some(key) = encryption_key_bytes(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let (payload, original_form) = expr_transport_bytes(&args[1]);
        let ciphertext = xor_with_sha256_keystream(&key, &payload);
        Ok(encrypted_object_expr(
            &ciphertext,
            &sha256_bytes(&key),
            original_form,
        ))
    }
}

fn byte_array_expr(bytes: &[u8]) -> Expr {
    Expr::call(
        system_expr("ByteArray"),
        vec![Expr::string(base64_encode(bytes))],
    )
}

fn hash_input_bytes(expr: &Expr) -> Vec<u8> {
    if let Some(value) = expr_as_string(expr) {
        value.as_bytes().to_vec()
    } else if let Some(bytes) = expr_as_byte_array_bytes(expr) {
        bytes
    } else {
        render_input_like_expr(expr).into_bytes()
    }
}

fn sha256_integer(digest: &[u8; SHA256_OUTPUT_LEN]) -> BigInt {
    BigInt::parse_bytes(bytes_to_hex(digest).as_bytes(), 16)
        .expect("sha256 digests always parse as hexadecimal integers")
}

fn sha256_decimal_string(value: &BigInt) -> String {
    const SHA256_DECIMAL_WIDTH: usize = 78;
    let decimal = value.to_str_radix(10);
    format!("{decimal:0>SHA256_DECIMAL_WIDTH$}")
}

fn compress_envelope(expr: &Expr) -> String {
    let (payload, original_form) = expr_transport_bytes(expr);
    let kind = if is_system_symbol_name(&original_form, "String") {
        "s"
    } else if is_system_symbol_name(&original_form, "ByteArray") {
        "b"
    } else {
        "e"
    };
    format!("rum:1:{kind}:{}", base64_encode(&payload))
}

fn expr_transport_bytes(expr: &Expr) -> (Vec<u8>, Expr) {
    if let Some(value) = expr_as_string(expr) {
        (value.as_bytes().to_vec(), system_expr("String"))
    } else if let Some(bytes) = expr_as_byte_array_bytes(expr) {
        (bytes, system_expr("ByteArray"))
    } else {
        (render_input_like_expr(expr).into_bytes(), system_expr("InputForm"))
    }
}

fn encryption_key_bytes(expr: &Expr) -> Option<Vec<u8>> {
    if let Some(value) = expr_as_string(expr) {
        Some(value.as_bytes().to_vec())
    } else {
        expr_as_byte_array_bytes(expr)
    }
}

fn expr_as_byte_array_bytes(expr: &Expr) -> Option<Vec<u8>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "ByteArray") || args.len() != 1 {
        return None;
    }
    let encoded = expr_as_string(&args[0])?;
    base64_decode(encoded)
}

fn encrypted_object_expr(ciphertext: &[u8], key_hash: &[u8], original_form: Expr) -> Expr {
    Expr::call(
        system_expr("EncryptedObject"),
        vec![render_association(vec![
            (Expr::string("Cipher"), Expr::string("XOR-SHA256")),
            (Expr::string("KeyHash"), byte_array_expr(key_hash)),
            (Expr::string("Data"), byte_array_expr(ciphertext)),
            (Expr::string("OriginalForm"), original_form),
        ])],
    )
}

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

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push('=');
            rendered.push('=');
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
            rendered.push('=');
        }
        _ => {}
    }

    rendered
}

fn base64_decode(encoded: &str) -> Option<Vec<u8>> {
    let bytes = encoded.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }

    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut index = 0;
    while index < bytes.len() {
        let a = decode_base64_nibble(bytes[index])?;
        let b = decode_base64_nibble(bytes[index + 1])?;
        let c = bytes[index + 2];
        let d = bytes[index + 3];

        let c_value = if c == b'=' {
            0
        } else {
            decode_base64_nibble(c)?
        };
        let d_value = if d == b'=' {
            0
        } else {
            decode_base64_nibble(d)?
        };

        let chunk =
            ((a as u32) << 18) | ((b as u32) << 12) | ((c_value as u32) << 6) | (d_value as u32);

        output.push(((chunk >> 16) & 0xff) as u8);
        if c != b'=' {
            output.push(((chunk >> 8) & 0xff) as u8);
        }
        if d != b'=' {
            output.push((chunk & 0xff) as u8);
        }

        index += 4;
    }

    Some(output)
}

fn decode_base64_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

fn xor_with_sha256_keystream(key: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(payload.len());
    let mut counter = 0u64;
    while output.len() < payload.len() {
        let mut counter_bytes = [0u8; 8];
        counter_bytes.copy_from_slice(&counter.to_be_bytes());
        let block = sha256_many([key, &counter_bytes]);
        for byte in block {
            if output.len() == payload.len() {
                break;
            }
            let index = output.len();
            output.push(payload[index] ^ byte);
        }
        counter += 1;
    }
    output
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
