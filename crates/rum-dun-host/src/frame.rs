//! Length-prefixed framing for Dun Plugin Protocol streams.

use std::io::{self, Read, Write};

/// Maximum accepted inbound payload size: one mebibyte.
pub const MAX_FRAME_BYTES: usize = 1 << 20;

/// Reads one little-endian length-prefixed frame.
///
/// End-of-stream, including a partial four-byte header, returns `Ok(None)`.
/// A complete header followed by a truncated payload is an error.
pub fn read_frame<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    let mut header_bytes = 0;
    while header_bytes < header.len() {
        match reader.read(&mut header[header_bytes..]) {
            Ok(0) => return Ok(None),
            Ok(count) => header_bytes += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }

    let payload_len = u32::from_le_bytes(header) as usize;
    if payload_len > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Dun protocol frame exceeds the 1 MiB limit",
        ));
    }

    let mut payload = vec![0u8; payload_len];
    reader.read_exact(&mut payload)?;
    Ok(Some(payload))
}

/// Writes and flushes one little-endian length-prefixed frame.
pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Dun protocol frame length does not fit in u32",
        )
    })?;
    writer.write_all(&payload_len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, ErrorKind};

    use super::*;

    #[test]
    fn round_trips_payload() {
        let payload = br#"{"kind":"hello"}"#;
        let mut framed = Vec::new();
        write_frame(&mut framed, payload).unwrap();

        let decoded = read_frame(&mut Cursor::new(framed)).unwrap();
        assert_eq!(decoded.as_deref(), Some(payload.as_slice()));
    }

    #[test]
    fn empty_reader_is_end_of_stream() {
        assert_eq!(
            read_frame(&mut Cursor::new(Vec::<u8>::new())).unwrap(),
            None
        );
    }

    #[test]
    fn partial_header_is_end_of_stream() {
        assert_eq!(read_frame(&mut Cursor::new(vec![1, 2, 3])).unwrap(), None);
    }

    #[test]
    fn rejects_length_above_cap_before_payload_read() {
        let header = ((MAX_FRAME_BYTES as u32) + 1).to_le_bytes();
        let error = read_frame(&mut Cursor::new(header)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn truncated_payload_is_unexpected_eof() {
        let mut framed = 5u32.to_le_bytes().to_vec();
        framed.extend_from_slice(b"abc");

        let error = read_frame(&mut Cursor::new(framed)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    }

    #[test]
    fn accepts_payload_at_cap_boundary() {
        let payload = vec![0x5a; MAX_FRAME_BYTES];
        let mut framed = Vec::new();
        write_frame(&mut framed, &payload).unwrap();

        let decoded = read_frame(&mut Cursor::new(framed)).unwrap();
        assert_eq!(decoded.as_deref(), Some(payload.as_slice()));
    }
}
