use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::{EvalError, Session, expr_as_string};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

mod codec;

use codec::{
    ByteInput, base64_decode, base64_encode, bool_expr, byte_array_expr, bytes_to_hex,
    expr_as_byte_array_bytes, fail_bytes, has_byte_array_head, hex_to_bytes, input_bytes,
};

pub(crate) fn eval_sha256(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_digest::<Sha256>(session, head, args, "Sha256")
}

pub(crate) fn eval_sha512(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_digest::<Sha512>(session, head, args, "Sha512")
}

pub(crate) fn eval_sha1(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_digest::<Sha1>(session, head, args, "Sha1")
}

pub(crate) fn eval_md5(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_digest::<Md5>(session, head, args, "Md5")
}

pub(crate) fn eval_crc32(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Crc32", args, 1));
    };
    let bytes = match input_bytes(arg) {
        ByteInput::Bytes(bytes) => bytes,
        ByteInput::InvalidByteArray => {
            return Ok(fail_bytes(
                session,
                "Crc32",
                "bytearray",
                "invalid ByteArray payload",
            ));
        }
        ByteInput::WrongType => return Ok(Expr::call(head.clone(), args.to_vec())),
    };

    let mut hasher = Crc32Hasher::new();
    hasher.update(&bytes);
    Ok(Expr::integer(BigInt::from(hasher.finalize())))
}

pub(crate) fn eval_to_hex_string(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ToHexString", args, 1));
    };
    match input_bytes(arg) {
        ByteInput::Bytes(bytes) => Ok(Expr::string(bytes_to_hex(&bytes))),
        ByteInput::InvalidByteArray => Ok(fail_bytes(
            session,
            "ToHexString",
            "bytearray",
            "invalid ByteArray payload",
        )),
        ByteInput::WrongType => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

pub(crate) fn eval_from_hex_string(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FromHexString", args, 1));
    };
    let Some(source) = expr_as_string(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match hex_to_bytes(source) {
        Some(bytes) => Ok(byte_array_expr(&bytes)),
        None => Ok(fail_bytes(
            session,
            "FromHexString",
            "hex",
            "expected an even-length hexadecimal string",
        )),
    }
}

pub(crate) fn eval_to_base64(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ToBase64", args, 1));
    };
    match input_bytes(arg) {
        ByteInput::Bytes(bytes) => Ok(Expr::string(base64_encode(&bytes))),
        ByteInput::InvalidByteArray => Ok(fail_bytes(
            session,
            "ToBase64",
            "bytearray",
            "invalid ByteArray payload",
        )),
        ByteInput::WrongType => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

pub(crate) fn eval_from_base64(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FromBase64", args, 1));
    };
    let Some(source) = expr_as_string(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match base64_decode(source) {
        Some(bytes) => Ok(byte_array_expr(&bytes)),
        None => Ok(fail_bytes(
            session,
            "FromBase64",
            "base64",
            "expected standard RFC 4648 base64 with valid padding",
        )),
    }
}

pub(crate) fn eval_byte_array_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ByteArrayQ", args, 1));
    };
    Ok(bool_expr(
        has_byte_array_head(arg) && expr_as_byte_array_bytes(arg).is_some(),
    ))
}

pub(crate) fn eval_byte_array_to_string(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ByteArrayToString", args, 1));
    };
    if !has_byte_array_head(arg) {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    let Some(bytes) = expr_as_byte_array_bytes(arg) else {
        return Ok(fail_bytes(
            session,
            "ByteArrayToString",
            "bytearray",
            "invalid ByteArray payload",
        ));
    };

    match String::from_utf8(bytes) {
        Ok(value) => Ok(Expr::string(value)),
        Err(_) => Ok(fail_bytes(
            session,
            "ByteArrayToString",
            "utf8",
            "ByteArray payload is not valid UTF-8",
        )),
    }
}

fn eval_digest<D: Digest>(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &'static str,
) -> Result<Expr, EvalError> {
    let (arg, bytes_output) = match args {
        [arg] => (arg, false),
        [arg, format] if expr_as_string(format).is_some_and(|format| format == "Bytes") => {
            (arg, true)
        }
        [_arg, _format] => return Ok(Expr::call(head.clone(), args.to_vec())),
        _ => return Ok(session.invalid_call_with_arg_range(symbol, args, 1, 2)),
    };

    let bytes = match input_bytes(arg) {
        ByteInput::Bytes(bytes) => bytes,
        ByteInput::InvalidByteArray => {
            return Ok(fail_bytes(
                session,
                symbol,
                "bytearray",
                "invalid ByteArray payload",
            ));
        }
        ByteInput::WrongType => return Ok(Expr::call(head.clone(), args.to_vec())),
    };
    let digest = D::digest(&bytes);

    if bytes_output {
        Ok(byte_array_expr(&digest))
    } else {
        Ok(Expr::string(bytes_to_hex(&digest)))
    }
}
