use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use rum_core::SharedArgs;
use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::{EvalError, Session, expr_as_string};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use super::codec::{ByteInput, byte_array_expr, bytes_to_hex, fail_bytes, input_bytes};

pub(crate) fn eval_sha256(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_digest::<Sha256>(session, head, args, "Sha256")
}

pub(crate) fn eval_sha512(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_digest::<Sha512>(session, head, args, "Sha512")
}

pub(crate) fn eval_sha1(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_digest::<Sha1>(session, head, args, "Sha1")
}

pub(crate) fn eval_md5(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_digest::<Md5>(session, head, args, "Md5")
}

pub(crate) fn eval_crc32(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
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

fn eval_digest<D: Digest>(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &'static str,
) -> Result<Expr, EvalError> {
    let (arg, format) = match args {
        [arg] => (arg, None),
        [arg, format] => (arg, Some(format)),
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
    let bytes_output = match format.and_then(expr_as_string) {
        None if format.is_none() => false,
        Some("Bytes") => true,
        Some("Hex") => false,
        _ => {
            return Ok(fail_bytes(
                session,
                symbol,
                "format",
                "unknown digest output format; use \"Hex\" (default) or \"Bytes\"",
            ));
        }
    };
    let digest = D::digest(&bytes);

    if bytes_output {
        Ok(byte_array_expr(&digest))
    } else {
        Ok(Expr::string(bytes_to_hex(&digest)))
    }
}
