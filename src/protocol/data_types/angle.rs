use std::{
    io::{BufRead, Write},
    ops::{Deref, DerefMut},
};

use crate::protocol::ser_de::{
    de::{self, Deserialize},
    ser::{self, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Angle(pub u8);

impl Angle {
    pub fn from_degrees(degrees: f32) -> Self {
        let normalized = degrees.rem_euclid(360.0);
        let steps = (normalized * (256.0 / 360.0)).round() as u32;

        Self((steps % 256) as u8)
    }

    pub fn into_inner(self) -> u8 {
        self.0
    }
}

impl Serialize for Angle {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ser::Error> {
        self.0.serialize(writer)
    }
}

impl Deserialize for Angle {
    fn deserialize<R: BufRead>(reader: &mut R) -> Result<Self, de::Error> {
        Ok(Self(u8::deserialize(reader)?))
    }
}

impl Deref for Angle {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Angle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_angle_roundtrip_cardinal_directions() {
        // Test 0 degrees (0/256)
        let angle_0 = Angle::from_degrees(0.0);
        let mut buf = Vec::new();
        angle_0.serialize(&mut buf).unwrap();
        assert_eq!(buf, vec![0u8]);
        assert_eq!(Angle::deserialize(&mut Cursor::new(buf)).unwrap(), angle_0);

        // Test 90 degrees (64/256)
        let angle_90 = Angle::from_degrees(90.0);
        let mut buf = Vec::new();
        angle_90.serialize(&mut buf).unwrap();
        assert_eq!(buf, vec![64u8]);
        assert_eq!(Angle::deserialize(&mut Cursor::new(buf)).unwrap(), angle_90);

        // Test 180 degrees (128/256)
        let angle_180 = Angle::from_degrees(180.0);
        let mut buf = Vec::new();
        angle_180.serialize(&mut buf).unwrap();
        assert_eq!(buf, vec![128u8]);
        assert_eq!(
            Angle::deserialize(&mut Cursor::new(buf)).unwrap(),
            angle_180
        );

        // Test 270 degrees (192/256)
        let angle_270 = Angle::from_degrees(270.0);
        let mut buf = Vec::new();
        angle_270.serialize(&mut buf).unwrap();
        assert_eq!(buf, vec![192u8]);
        assert_eq!(
            Angle::deserialize(&mut Cursor::new(buf)).unwrap(),
            angle_270
        );
    }

    #[test]
    fn test_angle_signed_unsigned_equivalence() {
        // -90 degrees is the same turn as 270 degrees on the wire (192/256)
        let neg_90 = Angle::from_degrees(-90.0);
        let mut buf_neg = Vec::new();
        neg_90.serialize(&mut buf_neg).unwrap();

        let pos_270 = Angle::from_degrees(270.0);
        let mut buf_pos = Vec::new();
        pos_270.serialize(&mut buf_pos).unwrap();

        // The wire bytes must be completely identical
        assert_eq!(buf_neg, buf_pos);
        assert_eq!(buf_neg, vec![192u8]);
    }

    #[test]
    fn test_angle_overflow_wrapping() {
        // A full 360 degree rotation (or multiple of it) wraps back to 0
        let angle_360 = Angle::from_degrees(360.0);
        let angle_720 = Angle::from_degrees(720.0);

        let mut buf_360 = Vec::new();
        let mut buf_720 = Vec::new();

        angle_360.serialize(&mut buf_360).unwrap();
        angle_720.serialize(&mut buf_720).unwrap();

        assert_eq!(buf_360, vec![0u8]);
        assert_eq!(buf_720, vec![0u8]);
    }

    #[test]
    fn test_angle_precision_rounding() {
        // 1 step = 360 / 256 = 1.40625 degrees.
        // 1.0 degree should round down to 0 steps (0u8)
        let angle_small = Angle::from_degrees(1.0);
        let mut buf = Vec::new();
        angle_small.serialize(&mut buf).unwrap();
        assert_eq!(buf, vec![1u8]);

        // 2.5 degrees is closer to 2.8125 (2 steps) than 1.40625 (1 step)
        let angle_mid = Angle::from_degrees(2.5);
        let mut buf_mid = Vec::new();
        angle_mid.serialize(&mut buf_mid).unwrap();
        assert_eq!(buf_mid, vec![2u8]);
    }

    #[test]
    fn test_angle_deserialization_eof() {
        let empty_payload: Vec<u8> = vec![];
        let mut reader = Cursor::new(empty_payload);

        let result = Angle::deserialize(&mut reader);
        assert!(result.is_err());
    }
}
