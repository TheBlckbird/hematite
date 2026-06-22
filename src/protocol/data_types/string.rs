use std::{
    io::{BufRead, Write},
    ops::{Deref, DerefMut},
};

use crate::protocol::{
    data_types::var_int::VarInt,
    ser_de::{
        de::{self, Deserialize},
        ser::{self, Serialize},
    },
};

/// String type used for the protocol.
///
/// It can additionally store a maximum length in UTF16 chars.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtoString {
    inner: String,
    /// Max length in UTF16 bytes. Defaults to 32767
    pub max_len: usize,
}

impl ProtoString {
    const MAX_LEN: usize = 32_767;

    pub fn new(inner: impl Into<String>, max_len: usize) -> Self {
        Self {
            inner: inner.into(),
            max_len,
        }
    }

    pub fn into_inner(self) -> String {
        self.inner
    }
}

impl From<String> for ProtoString {
    fn from(value: String) -> Self {
        Self {
            inner: value,
            max_len: Self::MAX_LEN,
        }
    }
}

impl From<&str> for ProtoString {
    fn from(value: &str) -> Self {
        Self {
            inner: value.into(),
            max_len: Self::MAX_LEN,
        }
    }
}

impl From<ProtoString> for String {
    fn from(value: ProtoString) -> Self {
        value.into_inner()
    }
}

impl Deref for ProtoString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ProtoString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Serialize for ProtoString {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let utf16_len = self.chars().map(|c| c.len_utf16()).sum::<usize>();

        if utf16_len > self.max_len {
            return Err(ser::Error::TooLong {
                expected: self.max_len,
                actual: utf16_len,
                context: "String length in UTF16",
            });
        }

        let bytes_len = self.len();
        if bytes_len > self.max_len * 3 {
            return Err(ser::Error::TooLong {
                expected: self.max_len * 3,
                actual: bytes_len,
                context: "String length in bytes",
            });
        }

        VarInt(bytes_len as i32).serialize(writer)?;
        writer.write_all(self.as_bytes()).map_err(ser::Error::Io)?;

        Ok(())
    }
}

impl Deserialize for ProtoString {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let max_len = Self::MAX_LEN;
        let bytes_length = *VarInt::deserialize(reader)? as usize;

        if bytes_length > max_len * 3 {
            return Err(de::Error::TooLong {
                expected: max_len * 3,
                actual: bytes_length,
                context: "String deserialization, exceeds max length in bytes (32767 * 3 = 98301)",
            });
        }

        let mut byte_buffer = vec![0; bytes_length];
        reader.read_exact(&mut byte_buffer).map_err(de::Error::Io)?;
        let value = String::from_utf8(byte_buffer).map_err(de::Error::FromUtf8Error)?;

        Ok(Self {
            inner: value,
            max_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip_basic_ascii() {
        let input = ProtoString::new("Hello!", 10);
        let mut buffer = Vec::new();

        // Serialize
        input.serialize(&mut buffer).unwrap();

        // Assert layout: Length 6 fits in 1 VarInt byte -> [6, H, e, l, l, o, !]
        assert_eq!(buffer[0], 6);
        assert_eq!(&buffer[1..], input.as_bytes());

        // Deserialize
        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();

        assert_eq!(input.inner, output.inner);
    }

    #[test]
    fn test_utf16_surrogate_counting() {
        // The crab emoji '🦀' is 4 bytes in UTF-8, but 2 code units in UTF-16
        let input = ProtoString::new("🦀", 1);
        let mut buffer = Vec::new();
        let result = input.serialize(&mut buffer);

        // A max_len of 1 should fail because 🦀 requires 2 UTF-16 code units
        assert!(result.is_err());

        // A max_len of 2 should pass
        let input = ProtoString::new("🦀", 2);
        let mut buffer = Vec::new();
        input.serialize(&mut buffer).unwrap();

        // The wire format byte length prefix should be 4
        assert_eq!(buffer[0], 4);

        // Ensure it roundtrips back perfectly
        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();
        assert_eq!(input.inner, output.inner);
    }

    #[test]
    fn test_byte_limit_expansion() {
        // Devanagari character 'अ' is 3 bytes in UTF-8, 1 code unit in UTF-16.
        let input = ProtoString::new("अ", 1);
        let mut buffer = Vec::new();
        input.serialize(&mut buffer).unwrap();
        assert_eq!(buffer[0], 3); // 3 bytes on the wire

        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();
        assert_eq!(input.inner, output.inner);
    }

    #[test]
    fn test_deserialization_early_abort_protection() {
        // Malicious payload: Claims an impossibly massive size upfront
        let mut malicious_payload = Vec::new();
        VarInt(150_000).serialize(&mut malicious_payload).unwrap();
        malicious_payload.extend_from_slice(b"short");

        let mut reader = Cursor::new(malicious_payload);
        let result = ProtoString::deserialize(&mut reader);

        // It should reject out-of-hand before allocating large vector pools
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8_payload() {
        let mut buffer = Vec::new();
        VarInt(4).serialize(&mut buffer).unwrap();
        buffer.extend_from_slice(&[0, 159, 146, 150]); // Invalid UTF-8 bytes

        let mut reader = Cursor::new(buffer);
        let result = ProtoString::deserialize(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_varint() {
        // An infinite VarInt payload sequence
        let infinite_varint = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let mut reader = Cursor::new(infinite_varint);

        let result = ProtoString::deserialize(&mut reader);
        assert!(result.is_err());
    }
}
