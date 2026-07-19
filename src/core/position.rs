#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Position {
    const I26_MIN: i32 = -33_554_432;
    const I26_MAX: i32 = 33_554_431;
    const I12_MIN: i16 = -2048;
    const I12_MAX: i16 = 2047;

    /// Creates a new position from x, y and z.
    ///
    /// ## Panics
    ///
    /// Panics if x or z are longer than 26 bytes or y is longer than 12 bytes
    pub fn new(x: i32, y: i16, z: i32) -> Self {
        assert!(x >= Self::I26_MIN);
        assert!(x <= Self::I26_MAX);

        assert!(y >= Self::I12_MIN);
        assert!(y <= Self::I12_MAX);

        assert!(z >= Self::I26_MIN);
        assert!(z <= Self::I26_MAX);

        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coordinates {
    pub kind: CoordinatesKind,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoordinatesKind {
    Absolute,
    Relative,
    Local,
}
