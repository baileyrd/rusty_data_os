//! Fixed-width little-endian protocol-22 payload codec, independent of bincode/serde.
//! usize is the protocol's 64-bit sequence/index representation on every host.
use crate::{Request, Response};

/// A wire value; implementations are provided for every protocol type.
pub trait Wire: Sized {
    fn put(&self, out: &mut Vec<u8>);
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String>;
}
pub struct Reader<'a> {
    remaining: &'a [u8],
}
impl<'a> Reader<'a> {
    pub fn bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        if count > self.remaining.len() {
            return Err("Malformed: truncated payload".into());
        }
        let (head, tail) = self.remaining.split_at(count);
        self.remaining = tail;
        Ok(head)
    }
}
pub fn encode<T: Wire>(value: &T) -> Vec<u8> {
    let mut out = Vec::new();
    value.put(&mut out);
    out
}
pub fn decode<T: Wire>(bytes: &[u8]) -> Result<T, String> {
    let mut input = Reader { remaining: bytes };
    let value = T::read_wire(&mut input)?;
    if !input.remaining.is_empty() {
        return Err("Malformed: trailing bytes".into());
    }
    Ok(value)
}
pub fn encode_request(value: &Request) -> Vec<u8> {
    encode(value)
}
pub fn decode_request(bytes: &[u8]) -> Result<Request, String> {
    decode(bytes)
}
pub fn encode_response(value: &Response) -> Vec<u8> {
    encode(value)
}
pub fn decode_response(bytes: &[u8]) -> Result<Response, String> {
    decode(bytes)
}
macro_rules! integer {
    ($($ty:ty),*) => { $(impl Wire for $ty {
        fn put(&self, out: &mut Vec<u8>) { out.extend_from_slice(&self.to_le_bytes()); }
        fn read_wire(input: &mut Reader<'_>) -> Result<Self,String> {
            let bytes = input.bytes(std::mem::size_of::<Self>())?;
            Ok(Self::from_le_bytes(bytes.try_into().map_err(|_| "Malformed: integer")?))
        }
    })* }
}
integer!(u8, u16, u32, u64, i64, f64);
impl Wire for usize {
    fn put(&self, out: &mut Vec<u8>) {
        (*self as u64).put(out);
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        u64::read_wire(input)?
            .try_into()
            .map_err(|_| "Malformed: usize overflow".into())
    }
}
impl Wire for bool {
    fn put(&self, out: &mut Vec<u8>) {
        u8::from(*self).put(out);
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u8::read_wire(input)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("Malformed: bool tag".into()),
        }
    }
}
impl<T: Wire> Wire for Option<T> {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            None => 0u8.put(out),
            Some(value) => {
                1u8.put(out);
                value.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u8::read_wire(input)? {
            0 => Ok(None),
            1 => Ok(Some(T::read_wire(input)?)),
            _ => Err("Malformed: option tag".into()),
        }
    }
}
impl Wire for String {
    fn put(&self, out: &mut Vec<u8>) {
        self.len().put(out);
        out.extend_from_slice(self.as_bytes());
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        let count = usize::read_wire(input)?;
        String::from_utf8(input.bytes(count)?.to_vec()).map_err(|_| "Malformed: UTF-8".into())
    }
}
impl<T: Wire> Wire for Vec<T> {
    fn put(&self, out: &mut Vec<u8>) {
        self.len().put(out);
        for item in self {
            item.put(out);
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        let count = usize::read_wire(input)?;
        // Every protocol element consumes at least one byte. Never allocate from an unchecked count.
        if count > input.remaining.len() {
            return Err("Malformed: vector count exceeds payload".into());
        }
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(T::read_wire(input)?);
        }
        Ok(values)
    }
}
impl<A: Wire, B: Wire> Wire for (A, B) {
    fn put(&self, out: &mut Vec<u8>) {
        self.0.put(out);
        self.1.put(out);
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        Ok((A::read_wire(input)?, B::read_wire(input)?))
    }
}
impl<A: Wire, B: Wire, C: Wire> Wire for (A, B, C) {
    fn put(&self, out: &mut Vec<u8>) {
        self.0.put(out);
        self.1.put(out);
        self.2.put(out);
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        Ok((
            A::read_wire(input)?,
            B::read_wire(input)?,
            C::read_wire(input)?,
        ))
    }
}
