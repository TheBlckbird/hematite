use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VarInt(pub i32);

impl VarInt {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut value = **self as u32;

        while (value & !0x7F) != 0 {
            let byte = ((value & 0x7F) | 0x80) as u8;
            bytes.push(byte);
            value >>= 7;
        }
        bytes.push(value as u8);

        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut value = 0;
        let mut bytes = bytes.iter();

        for position in (0..32).step_by(7) {
            let current_byte = *bytes.next().ok_or("Unexpected end of bytes")?;

            value |= ((current_byte & 0x7F) as i32) << position;

            if (current_byte & 0x80) == 0 {
                return Ok(VarInt(value));
            }
        }

        Err("VarInt incomplete or too large".into())
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_bytes::serialize(&self.encode(), serializer)
    }
}

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = serde_bytes::deserialize(deserializer)?;
        VarInt::decode(bytes).map_err(serde::de::Error::custom)
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
            assert_eq!(VarInt(value).encode(), expected, "Failed for value {value}");
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
            assert_eq!(
                VarInt::decode(&bytes),
                Ok(VarInt(expected)),
                "Failed for value {expected}"
            );
        }
    }
}
