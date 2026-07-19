use std::io::{BufRead, Write};

use crab_nbt::{Nbt, NbtTag};

use crate::protocol::ser_de::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

impl Serialize for Nbt {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        self.write_unnamed_to_writer(writer)
            .map_err(ser::Error::Nbt)
    }
}

// IMPORTANT: Deserialize for Nbt always reads the whole buffer to the end

impl Deserialize for Nbt {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).map_err(de::Error::Io)?;

        Self::read_unnamed(&mut buffer.as_slice()).map_err(de::Error::Nbt)
    }
}

impl Deserialize for NbtTag {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).map_err(de::Error::Io)?;

        Self::deserialize(&mut buffer.as_slice()).map_err(de::Error::Nbt)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crab_nbt::NbtTag;

    use crate::protocol::data_types::text_component::TextComponent;

    use super::*;

    #[test]
    fn test_nbt_tag_works() {
        let tag = TextComponent::literal("test");
        let mut buffer = Vec::new();
        tag.serialize(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let tag: NbtTag = Deserialize::deserialize(&mut cursor).unwrap();

        assert_eq!(NbtTag::String("test".into()), tag);
    }
}
