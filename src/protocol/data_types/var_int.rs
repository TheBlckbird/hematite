use std::{
    io::{BufRead, Write},
    ops::{Deref, DerefMut},
};

use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::protocol::ser_de::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

macro_rules! deserialize_fn {
    ($buffer:ident, $read_byte:expr) => {{
        let mut value = 0;

        for position in (0..32).step_by(7) {
            let mut $buffer = [0; 1];
            $read_byte.map_err(de::Error::Io)?;

            let current_byte = $buffer[0];

            value |= ((current_byte & 0x7F) as i32) << position;

            if (current_byte & 0x80) == 0 {
                return Ok(VarInt(value));
            }
        }

        Err(de::Error::Message("VarInt incomplete or too large")) // [TODO] convert to custom error
    }};
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct VarInt(pub i32);

impl VarInt {
    pub fn len(&self) -> usize {
        if self.is_positive() {
            let len = 32 - self.leading_zeros();
            (len as f32 / 7.0).ceil() as usize
        } else {
            let len = 32 - self.abs().leading_zeros();
            (len as f32 / 7.0).ceil() as usize
        }
    }

    pub fn into_inner(self) -> i32 {
        self.0
    }

    pub async fn from_socket(socket: &mut TcpStream) -> Result<Self, de::Error> {
        deserialize_fn!(buffer, socket.read_exact(&mut buffer).await)
    }
}

impl Serialize for VarInt {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        let mut value = **self as u32;

        while (value & !0x7F) != 0 {
            let byte = ((value & 0x7F) | 0x80) as u8;
            writer.write_all(&[byte]).map_err(ser::Error::Io)?;
            value >>= 7;
        }
        writer.write_all(&[value as u8]).map_err(ser::Error::Io)?;

        Ok(())
    }
}

impl Deserialize for VarInt {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        deserialize_fn!(buffer, reader.read_exact(&mut buffer))
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_varint_encode() {
        let test_cases = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (value, expected) in test_cases {
            let mut writer = Vec::new();
            VarInt(value).serialize(&mut writer).unwrap();

            assert_eq!(writer, expected, "Failed for value {value}");
        }
    }

    #[test]
    fn test_varint_decode() {
        let test_cases = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (expected, bytes) in test_cases {
            let mut reader = BufReader::new(bytes.as_slice());

            assert_eq!(
                VarInt::deserialize(&mut reader).unwrap(),
                VarInt(expected),
                "Failed for value {expected}"
            );
        }
    }
}
