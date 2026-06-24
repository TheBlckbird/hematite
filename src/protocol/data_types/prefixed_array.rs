use std::io::{BufRead, Write};

use crate::protocol::{
    data_types::var_int::VarInt,
    ser_de::{
        de::{self, Deserialize},
        ser::{self, Serialize},
    },
};

impl<T> Serialize for &[T]
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        VarInt(self.len() as i32).serialize(writer)?;

        for element in self.iter() {
            element.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T> Serialize for Box<[T]>
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        VarInt(self.len() as i32).serialize(writer)?;

        for element in self.iter() {
            element.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T> Deserialize for Box<[T]>
where
    T: Deserialize,
{
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let length = *VarInt::deserialize(reader)? as usize;
        let mut result = Vec::with_capacity(length);

        for _ in 0..length {
            result.push(T::deserialize(reader)?);
        }

        Ok(result.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_it_works() {
        let input = [1u8, 2, 3, 4, 5].as_slice();
        let mut writer = Vec::new();
        input.serialize(&mut writer).unwrap();

        assert_eq!(writer, vec![0x05, 0x01, 0x02, 0x03, 0x04, 0x05]);

        let mut reader = BufReader::new(writer.as_slice());
        let deserialized = Box::<[u8]>::deserialize(&mut reader).unwrap().to_vec();
        assert_eq!(deserialized, input);
    }
}
