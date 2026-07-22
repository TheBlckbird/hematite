use std::io::{BufRead, Write};

use hematite_serialization::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

use crate::core::position::Position;

impl Serialize for Position {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let packed = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);

        packed.serialize(writer)
    }
}

impl Deserialize for Position {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let value = i64::deserialize(reader)?;

        let x = (value >> 38) as i32;
        let y = (value << 52 >> 52) as i16;
        let z = (value << 26 >> 38) as i32;

        Ok(Self::new(x, y, z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_wiki_provided_example() {
        // From the wiki specification:
        // X = 18357644
        // Z = -20882616
        // Y = 831
        let pos = Position::new(18357644, 831, -20882616);
        let mut buffer = Vec::new();

        pos.serialize(&mut buffer).unwrap();

        // Ensure serialization results in exactly 8 bytes
        assert_eq!(buffer.len(), 8);
        let expected = [
            0b01000110, 0b00000111, 0b01100011, 0b00101100, 0b00010101, 0b10110100, 0b10000011,
            0b00111111,
        ];

        assert_eq!(buffer, expected);

        // Convert back and assert structural parity
        let mut reader = Cursor::new(buffer);
        let decoded = Position::deserialize(&mut reader).unwrap();

        assert_eq!(pos.x, decoded.x);
        assert_eq!(pos.y, decoded.y);
        assert_eq!(pos.z, decoded.z);
    }

    #[test]
    fn test_zero_coordinates() {
        let pos = Position::new(0, 0, 0);
        let mut buffer = Vec::new();

        pos.serialize(&mut buffer).unwrap();

        // Zero values across all 3 spatial structures should result in a zeroed out block
        assert_eq!(buffer, vec![0u8; 8]);

        let mut reader = Cursor::new(buffer);
        let decoded = Position::deserialize(&mut reader).unwrap();
        assert_eq!(decoded, pos);
    }

    #[test]
    fn test_negative_sign_extensions() {
        // Test all coordinates containing negative numbers to guarantee arithmetic sign bit replication works
        let pos = Position::new(-100, -5, -2500);
        let mut buffer = Vec::new();

        pos.serialize(&mut buffer).unwrap();

        let mut reader = Cursor::new(buffer);
        let decoded = Position::deserialize(&mut reader).unwrap();

        assert_eq!(decoded.x, -100);
        assert_eq!(decoded.y, -5);
        assert_eq!(decoded.z, -2500);
    }

    #[test]
    fn test_extreme_boundary_limits() {
        // Maximum limits allowed by the bit allocations:
        // X and Z hold 26 bits signed: range is -33,554,432 to 33,554,431
        // Y holds 12 bits signed: range is -2,048 to 2,047
        let max_pos = Position::new(33554431, 2047, 33554431);
        let min_pos = Position::new(-33554432, -2048, -33554432);

        // Roundtrip Max
        let mut max_buf = Vec::new();
        max_pos.serialize(&mut max_buf).unwrap();
        let mut max_reader = Cursor::new(max_buf);
        assert_eq!(Position::deserialize(&mut max_reader).unwrap(), max_pos);

        // Roundtrip Min
        let mut min_buf = Vec::new();
        min_pos.serialize(&mut min_buf).unwrap();
        let mut min_reader = Cursor::new(min_buf);
        assert_eq!(Position::deserialize(&mut min_reader).unwrap(), min_pos);
    }

    #[test]
    #[should_panic]
    fn test_too_high_panics() {
        Position::new(i32::MAX, i16::MAX, i32::MAX);
    }

    #[test]
    #[should_panic]
    fn test_too_low_panics() {
        Position::new(i32::MIN, i16::MIN, i32::MIN);
    }

    #[test]
    fn test_position_deserialization_eof() {
        // An 8-byte chunk is mandatory; check that partial payloads are rejected
        let broken_payload = vec![0u8; 5];
        let mut reader = Cursor::new(broken_payload);

        let result = Position::deserialize(&mut reader);
        assert!(result.is_err());
    }
}
