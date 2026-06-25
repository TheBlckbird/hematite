use std::io::{BufRead, Write};

use derive_more::{Deref, DerefMut};

use crate::protocol::ser_de::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
pub struct Array<T, const LENGTH: usize>(pub [T; LENGTH]);

impl<T, const LENGTH: usize> Array<T, LENGTH> {
    pub fn into_inner(self) -> [T; LENGTH] {
        self.0
    }
}

impl<T, const LENGTH: usize> Serialize for Array<T, LENGTH>
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        for element in &**self {
            element.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T, const LENGTH: usize> Deserialize for Array<T, LENGTH>
where
    T: Deserialize + Default + Copy,
{
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut result = [T::default(); LENGTH];

        for element in &mut result {
            *element = T::deserialize(reader)?;
        }

        Ok(Self(result))
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_standard_case() {
        let input = Array([1u16, 2, 3, 4, 5]);
        let mut writer = Vec::new();
        input.serialize(&mut writer).unwrap();

        assert_eq!(
            writer,
            vec![0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05]
        );

        let mut reader = BufReader::new(writer.as_slice());
        let deserialized = Array::<u16, 5>::deserialize(&mut reader).unwrap();
        assert_eq!(deserialized, input);
    }

    #[test]
    #[should_panic]
    fn test_wrong_length() {
        let input = [0x00u8, 0x05];
        let mut reader = BufReader::new(input.as_slice());

        Array::<u8, 4>::deserialize(&mut reader).unwrap();
    }
}
