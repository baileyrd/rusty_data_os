//! Four-byte little-endian payload length followed by exactly that payload.
use std::io::{self, Read, Write};
pub const MAX_FRAME_BYTES: u32 = 16 * 1024 * 1024;
pub fn write_message<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_FRAME_BYTES as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds 16 MiB",
        ));
    }
    writer.write_all(&(payload.len() as u32).to_le_bytes())?;
    writer.write_all(payload)
}
pub fn read_message<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix)?;
    let count = u32::from_le_bytes(prefix);
    if count > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds 16 MiB",
        ));
    }
    let mut payload = vec![0; count as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}
