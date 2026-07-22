use std::io::{BufRead, Write};

use derive_more::{Deref, DerefMut};

use crate::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deref, DerefMut)]
pub struct VarLong(pub i64);

impl Serialize for VarLong {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let mut value = **self as u64;

        while (value & !0x7F) != 0 {
            let byte = ((value & 0x7F) | 0x80) as u8;
            writer.write_all(&[byte]).map_err(ser::Error::Io)?;
            value >>= 7;
        }
        writer.write_all(&[value as u8]).map_err(ser::Error::Io)?;

        Ok(())
    }
}

impl Deserialize for VarLong {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut value = 0;

        for position in (0..64).step_by(7) {
            let mut buffer = [0; 1];
            reader.read_exact(&mut buffer).map_err(de::Error::Io)?;
            let current_byte = buffer[0];

            value |= ((current_byte & 0x7F) as i64) << position;

            if (current_byte & 0x80) == 0 {
                return Ok(VarLong(value));
            }
        }

        Err(de::Error::Message("VarInt incomplete or too large")) // [TODO] convert to custom error
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_varlong_encode() {
        let test_cases = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (
                9223372036854775807,
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            ),
            (
                -1,
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -2147483648,
                vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -9223372036854775808,
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ];

        for (value, expected) in test_cases {
            let mut writer = Vec::new();
            VarLong(value).serialize(&mut writer).unwrap();

            assert_eq!(writer, expected, "Failed for value {value}");
        }
    }

    #[test]
    fn test_varlong_decode() {
        let test_cases = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (
                9223372036854775807,
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            ),
            (
                -1,
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -2147483648,
                vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -9223372036854775808,
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ];

        for (expected, bytes) in test_cases {
            let mut reader = BufReader::new(bytes.as_slice());

            assert_eq!(
                VarLong::deserialize(&mut reader).unwrap(),
                VarLong(expected),
                "Failed for value {expected}"
            );
        }
    }
}
