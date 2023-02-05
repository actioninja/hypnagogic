use enum_iterator::Sequence;
use fixed_map::Key;
use serde::{Deserialize, Serialize};

/// Represents a "side" of a given tile. Directions correspond to unrotated cardinal directions,
/// with "North" pointing "upwards."
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug, Sequence, Serialize, Deserialize, Key,
)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Side {
    /// Matches enum variants to Byond bitfield directions
    #[must_use]
    pub const fn byond_dir(&self) -> u8 {
        match self {
            Side::North => 0b0000_0001,
            Side::South => 0b0000_0010,
            Side::East => 0b0000_0100,
            Side::West => 0b0000_1000,
        }
    }

    /// Returns an array of directions in the order that byond specifies directions.
    /// Yes, it is correct that "South" is done before North
    #[must_use]
    pub const fn dmi_cardinals() -> [Self; 4] {
        [Self::South, Self::North, Self::East, Self::West]
    }

    /// Returns a boolean determining whether a Side is a "vertical" side. "North" and "South"
    /// return true and vice versa. Maybe this is reversed, depends on whether you think of the side
    /// as the line making it up or not.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        match self {
            Self::North | Self::South => true,
            Self::East | Self::West => false,
        }
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug, Sequence, Serialize, Deserialize, Key,
)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

impl Corner {
    #[must_use]
    pub const fn sides_of_corner(self) -> (Side, Side) {
        match self {
            Corner::NorthEast => (Side::East, Side::North),
            Corner::SouthEast => (Side::East, Side::South),
            Corner::SouthWest => (Side::West, Side::South),
            Corner::NorthWest => (Side::West, Side::North),
        }
    }

    /// Generates a Byond bitfield direction given the corner
    #[must_use]
    pub const fn byond_dir(self) -> u8 {
        let (horizontal, vertical) = self.sides_of_corner();
        horizontal.byond_dir() | vertical.byond_dir()
    }
}

/// Represents the five possible given states for a corner to be in when bitmask smoothing
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Debug, Deserialize, Serialize, Key)]
#[serde(rename_all = "snake_case")]
pub enum CornerType {
    Convex,
    Concave,
    Horizontal,
    Vertical,
    Flat,
}

impl CornerType {
    /// When only smoothing along cardinals, the "Flat" corner type is not used. This returns a
    /// Vec of the enum variants except for `Flat`.
    #[must_use]
    pub fn cardinal() -> Vec<Self> {
        vec![
            Self::Convex,
            Self::Concave,
            Self::Horizontal,
            Self::Vertical,
        ]
    }

    /// Returns a Vec of all enum variants
    #[must_use]
    pub fn diagonal() -> Vec<Self> {
        vec![
            Self::Convex,
            Self::Concave,
            Self::Horizontal,
            Self::Vertical,
            Self::Flat,
        ]
    }
}
