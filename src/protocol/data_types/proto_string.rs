use std::io::{BufRead, Write};

use derive_more::{Deref, DerefMut};

use crate::protocol::{
    data_types::var_int::VarInt,
    ser_de::{
        de::{self, Deserialize},
        ser::{self, Serialize},
    },
};

const DEFAULT_MAX_LEN: usize = 32_767;

/// String type used for the protocol.
///
/// It can additionally store a maximum length in UTF16 chars.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
pub struct ProtoString<const MAX_LEN: usize = DEFAULT_MAX_LEN>(String);

impl<const MAX_LEN: usize> ProtoString<MAX_LEN> {
    const _ASSERT_MAX_LEN: () = {
        assert!(MAX_LEN <= DEFAULT_MAX_LEN, "MAX_LEN cannot exceed 32,767");
    };

    pub fn new(inner: impl Into<String>) -> Self {
        #[allow(clippy::let_unit_value)]
        let _ = Self::_ASSERT_MAX_LEN;
        Self(inner.into())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<const MAX_LEN: usize> From<String> for ProtoString<MAX_LEN> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<const MAX_LEN: usize> From<&str> for ProtoString<MAX_LEN> {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl<const MAX_LEN: usize> From<ProtoString<MAX_LEN>> for String {
    fn from(value: ProtoString<MAX_LEN>) -> Self {
        value.into_inner()
    }
}

impl<const MAX_LEN: usize> Serialize for ProtoString<MAX_LEN> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        assert!(MAX_LEN <= DEFAULT_MAX_LEN, "MAX_LEN cannot exceed 32,767");

        let utf16_len = self.chars().map(|c| c.len_utf16()).sum::<usize>();

        if utf16_len > MAX_LEN {
            return Err(ser::Error::TooLong {
                expected: MAX_LEN,
                actual: utf16_len,
                context: "String length in UTF16",
            });
        }

        let bytes_len = self.len();
        if bytes_len > MAX_LEN * 3 {
            return Err(ser::Error::TooLong {
                expected: MAX_LEN * 3,
                actual: bytes_len,
                context: "String length in bytes",
            });
        }

        VarInt(bytes_len as i32).serialize(writer)?;
        writer.write_all(self.as_bytes()).map_err(ser::Error::Io)?;

        Ok(())
    }
}

impl<const MAX_LEN: usize> Deserialize for ProtoString<MAX_LEN> {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let bytes_length = *VarInt::deserialize(reader)? as usize;

        if bytes_length > MAX_LEN * 3 {
            return Err(de::Error::TooLong {
                expected: MAX_LEN * 3,
                actual: bytes_length,
                context: "String deserialization, exceeds max length in bytes (32767 * 3 = 98301)",
            });
        }

        let mut byte_buffer = vec![0; bytes_length];
        reader.read_exact(&mut byte_buffer).map_err(de::Error::Io)?;
        let value = String::from_utf8(byte_buffer).map_err(de::Error::FromUtf8Error)?;

        Ok(Self::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip_basic_ascii() {
        let input = ProtoString::<10>::new("Hello!");
        let mut buffer = Vec::new();

        // Serialize
        input.serialize(&mut buffer).unwrap();

        // Assert layout: Length 6 fits in 1 VarInt byte -> [6, H, e, l, l, o, !]
        assert_eq!(buffer[0], 6);
        assert_eq!(&buffer[1..], input.as_bytes());

        // Deserialize
        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_utf16_surrogate_counting() {
        // The crab emoji '🦀' is 4 bytes in UTF-8, but 2 code units in UTF-16
        let input = ProtoString::<1>::new("🦀");
        let mut buffer = Vec::new();
        let result = input.serialize(&mut buffer);

        // A max_len of 1 should fail because 🦀 requires 2 UTF-16 code units
        assert!(result.is_err());

        // A max_len of 2 should pass
        let input = ProtoString::<2>::new("🦀");
        let mut buffer = Vec::new();
        input.serialize(&mut buffer).unwrap();

        // The wire format byte length prefix should be 4
        assert_eq!(buffer[0], 4);

        // Ensure it roundtrips back perfectly
        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_byte_limit_expansion() {
        // Devanagari character 'अ' is 3 bytes in UTF-8, 1 code unit in UTF-16.
        let input = ProtoString::<1>::new("अ");
        let mut buffer = Vec::new();
        input.serialize(&mut buffer).unwrap();
        assert_eq!(buffer[0], 3); // 3 bytes on the wire

        let mut reader = Cursor::new(buffer);
        let output = ProtoString::deserialize(&mut reader).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_deserialization_early_abort_protection() {
        // Malicious payload: Claims an impossibly massive size upfront
        let mut malicious_payload = Vec::new();
        VarInt(150_000).serialize(&mut malicious_payload).unwrap();
        malicious_payload.extend_from_slice(b"short");

        let mut reader = Cursor::new(malicious_payload);
        let result = ProtoString::<32_767>::deserialize(&mut reader);

        // It should reject out-of-hand before allocating large vector pools
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8_payload() {
        let mut buffer = Vec::new();
        VarInt(4).serialize(&mut buffer).unwrap();
        buffer.extend_from_slice(&[0, 159, 146, 150]); // Invalid UTF-8 bytes

        let mut reader = Cursor::new(buffer);
        let result = ProtoString::<32_767>::deserialize(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_varint() {
        // An infinite VarInt payload sequence
        let infinite_varint = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let mut reader = Cursor::new(infinite_varint);

        let result = ProtoString::<32_767>::deserialize(&mut reader);
        assert!(result.is_err());
    }
}
