use std::io::{BufRead, Write};

use crate::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let has_value = self.is_some();
        has_value.serialize(writer)?;

        match self {
            Some(value) => value.serialize(writer),
            None => Ok(()),
        }
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let has_value = bool::deserialize(reader)?;

        if has_value {
            Ok(Some(T::deserialize(reader)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_none_works() {
        let mut writer = Vec::new();
        None::<u8>.serialize(&mut writer).unwrap();

        assert_eq!(writer, vec![0x00]);

        let mut reader = BufReader::new(writer.as_slice());
        let deserialized = Option::<u8>::deserialize(&mut reader).unwrap();
        assert_eq!(deserialized, None);
    }

    #[test]
    fn test_some_works() {
        let mut writer = Vec::new();
        let input = Some(5u8);
        input.serialize(&mut writer).unwrap();

        assert_eq!(writer, vec![0x01, 0x05]);

        let mut reader = BufReader::new(writer.as_slice());
        let deserialized = Option::<u8>::deserialize(&mut reader).unwrap();
        assert_eq!(deserialized, input);
    }
}
