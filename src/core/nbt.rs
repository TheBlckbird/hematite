use std::path::PathBuf;

use crab_nbt::NbtTag;
use uuid::Uuid;

pub trait IntoNbtTag {
    fn into_nbt_tag(self) -> NbtTag;
}

pub trait FromNbtTag: Sized {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self>;
}

impl IntoNbtTag for String {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::String(self)
    }
}

impl FromNbtTag for String {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::String(content) => Some(content),
            _ => None,
        }
    }
}

impl IntoNbtTag for &'static str {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::String(self.to_string())
    }
}

impl IntoNbtTag for bool {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::Byte(match self {
            true => 1,
            false => 0,
        })
    }
}

impl FromNbtTag for bool {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::Byte(1) => Some(true),
            NbtTag::Byte(0) => Some(false),
            _ => None,
        }
    }
}

impl<T: IntoNbtTag> IntoNbtTag for Vec<T> {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::List(self.into_iter().map(|item| item.into_nbt_tag()).collect())
    }
}

impl<T: FromNbtTag> FromNbtTag for Vec<T> {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::List(list) => list.into_iter().map(|tag| T::from_nbt_tag(tag)).collect(),
            _ => None,
        }
    }
}

impl IntoNbtTag for u8 {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::Byte(self as i8)
    }
}

impl FromNbtTag for u8 {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::Byte(byte) => Some(byte as u8),
            _ => None,
        }
    }
}

impl IntoNbtTag for i32 {
    fn into_nbt_tag(self) -> NbtTag {
        NbtTag::Int(self)
    }
}

impl FromNbtTag for i32 {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::Int(integer) => Some(integer),
            _ => None,
        }
    }
}

impl IntoNbtTag for Uuid {
    fn into_nbt_tag(self) -> NbtTag {
        let int_array = self
            .as_u128()
            .to_be_bytes()
            .chunks(4)
            .map(|int_chunks| {
                i32::from_be_bytes(
                    int_chunks
                        .to_owned()
                        .try_into()
                        .expect("Created chunks of 4 the step before"),
                )
            })
            .collect();

        NbtTag::IntArray(int_array)
    }
}

impl FromNbtTag for Uuid {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::IntArray(int_array) => {
                let length_constrained_array: [i32; 4] = int_array.try_into().ok()?;
                let mut bytes = Vec::new();

                for integer in length_constrained_array {
                    bytes.extend(integer.to_be_bytes());
                }

                Some(Uuid::from_bytes(bytes.try_into().ok()?))
            }
            NbtTag::String(stringified) => Uuid::parse_str(&stringified).ok(),
            _ => None,
        }
    }
}

impl IntoNbtTag for PathBuf {
    fn into_nbt_tag(self) -> NbtTag {
        self.into_os_string()
            .into_string()
            .expect("Path should be valid UTF-8")
            .into()
    }
}

impl FromNbtTag for PathBuf {
    fn from_nbt_tag(tag: NbtTag) -> Option<Self> {
        match tag {
            NbtTag::String(path_value) => Some(PathBuf::from(path_value)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_works() {
        let uuid = Uuid::parse_str("0b752614-d3a1-4eef-9d8a-00292f9ecdc5").unwrap();

        let as_u128 = 0b00001011011101010010011000010100110100111010000101001110111011111001110110001010000000000010100100101111100111101100110111000101_u128;
        assert_eq!(uuid.as_u128(), as_u128);

        let int_array = NbtTag::IntArray(vec![
            0b00001011011101010010011000010100_u32 as i32,
            0b11010011101000010100111011101111_u32 as i32,
            0b10011101100010100000000000101001_u32 as i32,
            0b00101111100111101100110111000101_u32 as i32,
        ]);
        assert_eq!(uuid.into_nbt_tag(), int_array);

        assert_eq!(Uuid::from_nbt_tag(int_array), Some(uuid));

        let string_tag = NbtTag::String("0b752614-d3a1-4eef-9d8a-00292f9ecdc5".to_string());
        assert_eq!(Uuid::from_nbt_tag(string_tag), Some(uuid));
    }
}
