use enum_iterator::Sequence;
use fixed_map::Key;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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

impl From<&str> for Side {
    fn from(s: &str) -> Self {
        match s {
            "north" => Self::North,
            "south" => Self::South,
            "east" => Self::East,
            "west" => Self::West,
            _ => panic!("Invalid side: {}", s),
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::North => write!(f, "north"),
            Side::South => write!(f, "south"),
            Side::East => write!(f, "east"),
            Side::West => write!(f, "west"),
        }
    }
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
    /// Returns the two sides that make up a given corner
    /// Order is always (horizontal, vertical)
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

impl From<&str> for CornerType {
    fn from(value: &str) -> Self {
        match value {
            "convex" => Self::Convex,
            "concave" => Self::Concave,
            "horizontal" => Self::Horizontal,
            "vertical" => Self::Vertical,
            "flat" => Self::Flat,
            _ => panic!("Invalid String: {}", value),
        }
    }
}

impl Display for CornerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CornerType::Convex => write!(f, "convex"),
            CornerType::Concave => write!(f, "concave"),
            CornerType::Horizontal => write!(f, "horizontal"),
            CornerType::Vertical => write!(f, "vertical"),
            CornerType::Flat => write!(f, "flat"),
        }
    }
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
