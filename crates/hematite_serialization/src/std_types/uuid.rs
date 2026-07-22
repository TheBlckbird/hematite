use std::io::{BufRead, Write};

use uuid::Uuid;

use crate::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

impl Serialize for Uuid {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        writer
            .write_all(&self.as_u128().to_be_bytes())
            .map_err(ser::Error::Io)?;

        Ok(())
    }
}

impl Deserialize for Uuid {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut buffer = [0; 16];
        reader.read_exact(&mut buffer).map_err(de::Error::Io)?;

        Ok(Uuid::from_bytes(buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::str::FromStr;

    #[test]
    fn test_uuid_roundtrip() {
        // Generate a random, valid v4 UUID
        let input = Uuid::new_v4();
        let mut buffer = Vec::new();

        // 1. Serialize
        input.serialize(&mut buffer).unwrap();

        // Assert wire constraint: UUIDs must always take exactly 16 bytes
        assert_eq!(buffer.len(), 16);
        assert_eq!(buffer.as_slice(), input.as_bytes());

        // 2. Deserialize
        let mut reader = Cursor::new(buffer);
        let output = Uuid::deserialize(&mut reader).unwrap();

        // 3. Verify parity
        assert_eq!(input, output);
        assert_eq!(output.get_version_num(), 4); // Verify it maintained its v4 state
    }

    #[test]
    fn test_uuid_known_value() {
        // Nil UUID (all zeros)
        let input = Uuid::nil();
        let mut buffer = Vec::new();

        input.serialize(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0u8; 16]); // Wire format should be 16 zeros

        let mut reader = Cursor::new(buffer);
        let output = Uuid::deserialize(&mut reader).unwrap();
        assert_eq!(output, Uuid::nil());
    }

    #[test]
    fn test_uuid_string_parsing_roundtrip() {
        // Test parsing from a standard textual hex representation (hyphenated)
        let text_repr = "6740b3c3-ee9b-4497-8d07-28564a275f10";
        let input = Uuid::from_str(text_repr).unwrap();

        let mut buffer = Vec::new();
        input.serialize(&mut buffer).unwrap();

        // Check if bytes correctly match the expected big-endian layout hex values
        // "67 40 b3 c3 ..."
        assert_eq!(buffer[0], 0x67);
        assert_eq!(buffer[1], 0x40);
        assert_eq!(buffer[15], 0x10);

        let mut reader = Cursor::new(buffer);
        let output = Uuid::deserialize(&mut reader).unwrap();
        assert_eq!(output.to_string(), text_repr);
    }

    #[test]
    fn test_uuid_deserialization_eof() {
        // Malicious/broken payload: Stream closes early providing only 10 bytes instead of 16
        let broken_payload = vec![0u8; 10];
        let mut reader = Cursor::new(broken_payload);

        let result = Uuid::deserialize(&mut reader);

        // It must return an Error because a full 128-bit integer sequence couldn't be satisfied
        assert!(result.is_err());
    }
}
