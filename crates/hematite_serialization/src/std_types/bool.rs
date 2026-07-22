use std::io::{BufRead, Write};

use crate::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

impl Serialize for bool {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        match self {
            true => writer.write_all(&[0x01]).map_err(ser::Error::Io)?,
            false => writer.write_all(&[0x00]).map_err(ser::Error::Io)?,
        };

        Ok(())
    }
}

impl Deserialize for bool {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut buffer = [0; 1];
        reader.read_exact(&mut buffer).map_err(de::Error::Io)?;

        match buffer[0] {
            0x00 => Ok(false),
            0x01 => Ok(true),
            _ => Err(de::Error::Syntax),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_serialization() {
        let mut writer = Vec::new();
        true.serialize(&mut writer).unwrap();
        assert_eq!(writer, vec![0x01]);

        let mut writer = Vec::new();
        false.serialize(&mut writer).unwrap();
        assert_eq!(writer, vec![0x00]);
    }

    #[test]
    fn test_deserialization() {
        let input = [0x00, 0x01];
        let mut reader = BufReader::new(&input[..]);

        assert!(!bool::deserialize(&mut reader).unwrap());
        assert!(bool::deserialize(&mut reader).unwrap());
    }
}
