use crate::{Durability, LogError, MAX_PAYLOAD, Transaction, Uuid};

pub(crate) fn normalize(txn: &Transaction) -> Result<Vec<u8>, LogError> {
    if txn.payload.len() > MAX_PAYLOAD {
        return Err(LogError::Limit);
    }
    if txn.payload.windows(4).any(|w| w == b"RDE1") {
        return Err(LogError::Invalid("payload contains the RF1 magic".into()));
    }
    let mut out = b"UCR1\0".to_vec();
    out.push(txn.durability as u8);
    out.extend((txn.payload.len() as u64).to_le_bytes());
    out.extend(&txn.payload);
    Ok(out)
}
pub(crate) fn request(bytes: &[u8]) -> Result<(Durability, &[u8]), String> {
    let mut c = Cursor(bytes);
    if c.take(5)? != b"UCR1\0" {
        return Err("normalized magic".into());
    }
    let durability = durability(c.byte()?)?;
    let length = usize::try_from(c.u64()?).map_err(|_| "request length")?;
    if length > MAX_PAYLOAD {
        return Err("request limit".into());
    }
    let payload = c.take(length)?;
    c.end()?;
    if payload.windows(4).any(|w| w == b"RDE1") {
        return Err("payload contains the RF1 magic".into());
    }
    Ok((durability, payload))
}
pub(crate) fn durability(b: u8) -> Result<Durability, String> {
    match b {
        1 => Ok(Durability::D1),
        2 => Ok(Durability::D2),
        _ => Err("invalid durability".into()),
    }
}
pub(crate) fn complete(
    txn: &Transaction,
    normalized: &[u8],
    event: Uuid,
    sequence: u64,
    time: i64,
) -> Vec<u8> {
    let mut out = b"UCE1\0".to_vec();
    out.extend(txn.request_id.0);
    out.extend(event.0);
    out.extend(sequence.to_le_bytes());
    out.extend(time.to_le_bytes());
    out.push(txn.durability as u8);
    out.extend(0u16.to_le_bytes());
    out.extend(0u16.to_le_bytes());
    out.extend(normalized);
    out
}
pub(crate) fn validate(
    bytes: &[u8],
    request_id: Uuid,
    event: Uuid,
    sequence: u64,
    time: i64,
    normalized: &[u8],
) -> Result<(), String> {
    let mut c = Cursor(bytes);
    if c.take(5)? != b"UCE1\0"
        || c.take(16)? != request_id.0
        || c.take(16)? != event.0
        || c.u64()? != sequence
        || c.i64()? != time
    {
        return Err("envelope identity".into());
    }
    let d = durability(c.byte()?)?;
    if c.u16()? != 0 || c.u16()? != 0 {
        return Err("provenance/references outside bounded contract".into());
    }
    if c.0 != normalized || request(c.0)?.0 != d {
        return Err("envelope normalized request".into());
    }
    Ok(())
}
pub(crate) struct Cursor<'a>(pub &'a [u8]);
impl<'a> Cursor<'a> {
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let (value, rest) = self.0.split_at_checked(n).ok_or("truncated encoding")?;
        self.0 = rest;
        Ok(value)
    }
    pub fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    pub fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn i64(&mut self) -> Result<i64, String> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn blob(&mut self) -> Result<&'a [u8], String> {
        let n = usize::try_from(self.u64()?).map_err(|_| "length overflow")?;
        self.take(n)
    }
    pub fn end(&self) -> Result<(), String> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err("trailing bytes".into())
        }
    }
}
pub(crate) fn blob(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend((bytes.len() as u64).to_le_bytes());
    out.extend(bytes);
}
