use std::io::{self, BufRead, Write};

use crate::protocol::ser_de::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

macro_rules! num_impl {
    ($($num_type:ident),*$(,)?) => {
        $(
            impl Serialize for $num_type {
                fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
                    writer
                        .write_all(&self.to_be_bytes())
                        .map_err(ser::Error::Io)?;

                    Ok(())
                }
            }

            impl Deserialize for $num_type {
                fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
                    let mut buffer = [0; size_of::<Self>()];
                    reader
                        .read_exact(&mut buffer)
                        .map_err(|err| match err.kind() {
                            io::ErrorKind::UnexpectedEof => de::Error::TooFewBytes {
                                expected: 1,
                                actual: 0,
                            },
                            _ => de::Error::Io(err),
                        })?;

                    Ok(Self::from_be_bytes(buffer))
                }
            }
        )*
    };
}

num_impl!(i8, u8, i16, u16, i32, i64, f32, f64);

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_num_serialization_works() {
        let mut writer = Vec::new();
        16u8.serialize(&mut writer).unwrap();
        assert_eq!(writer, vec![16]);
    }
    #[test]
    fn test_roundtrip_u16() {
        let mut writer = Vec::new();
        500u16.serialize(&mut writer).unwrap();

        let mut bytes = writer.as_slice();
        let value = u16::deserialize(&mut bytes).unwrap();
        assert_eq!(value, 500);
    }

    #[test]
    fn test_roundtrip_i32() {
        let mut writer = Vec::new();
        (-12345i32).serialize(&mut writer).unwrap();

        let mut bytes = writer.as_slice();
        let value = i32::deserialize(&mut bytes).unwrap();
        assert_eq!(value, -12345);
    }

    #[test]
    fn test_big_endian_encoding() {
        let mut writer = Vec::new();
        0x0102u16.serialize(&mut writer).unwrap();
        assert_eq!(writer, vec![0x01, 0x02]);
    }

    #[test]
    fn test_deserialize_updates_slice() {
        let mut data = [0x00, 0x10, 0xFF].as_slice();
        let value = u16::deserialize(&mut data).unwrap();

        assert_eq!(value, 16);
        assert_eq!(data, &[0xFF]);
    }

    #[test]
    fn test_multiple_values() {
        let mut data = [
            0x00, 0x0A, // 10u16
            0x00, 0x14, // 20u16
        ]
        .as_slice();

        let a = u16::deserialize(&mut data).unwrap();
        let b = u16::deserialize(&mut data).unwrap();

        assert_eq!(a, 10);
        assert_eq!(b, 20);
        assert!(data.is_empty());
    }

    #[test]
    fn test_max_min_values() {
        let mut writer = Vec::new();
        i64::MAX.serialize(&mut writer).unwrap();
        let mut max = writer.as_slice();
        assert_eq!(i64::deserialize(&mut max).unwrap(), i64::MAX);

        writer.clear();

        let mut writer = Vec::new();
        i64::MIN.serialize(&mut writer).unwrap();
        let mut min = writer.as_slice();
        assert_eq!(i64::deserialize(&mut min).unwrap(), i64::MIN);

        writer.clear();

        i32::MAX.serialize(&mut writer).unwrap();
        let mut max = writer.as_slice();
        assert_eq!(i32::deserialize(&mut max).unwrap(), i32::MAX);

        writer.clear();

        i32::MIN.serialize(&mut writer).unwrap();
        let mut min = writer.as_slice();
        assert_eq!(i32::deserialize(&mut min).unwrap(), i32::MIN);
    }

    #[test]
    #[should_panic]
    fn test_insufficient_bytes_panics() {
        let mut data = [0x01].as_slice(); // too short for u16
        let _ = u16::deserialize(&mut data).unwrap();
    }

    #[test]
    fn test_incorrect_size_usage() {
        let mut writer = Vec::new();
        0x0102u16.serialize(&mut writer).unwrap();
        let mut bytes = writer.as_slice();
        let value = u16::deserialize(&mut bytes).unwrap();

        // This will fail with current implementation because only 1 byte is read
        assert_eq!(value, 0x0102);
    }

    #[test]
    fn test_consecutive_writes() {
        let mut writer = Vec::new();
        0x12u16.serialize(&mut writer).unwrap();
        0x13u16.serialize(&mut writer).unwrap();
        assert_eq!(writer, vec![0x00, 0x12, 0x00, 0x13]);
    }

    #[test]
    fn test_consecutive_reads() {
        let data = [0x00, 0x12, 0x00, 0x13];
        let mut reader = BufReader::new(&data[..]);

        let first = u16::deserialize(&mut reader).unwrap();
        assert_eq!(first, 0x12);

        let second = u16::deserialize(&mut reader).unwrap();
        assert_eq!(second, 0x13);
    }
}
