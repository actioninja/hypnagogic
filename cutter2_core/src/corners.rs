use enum_iterator::Sequence;
use fixed_map::{Key, Map};
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;

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
    pub const fn sides_of_corner(&self) -> (Side, Side) {
        match self {
            Corner::NorthEast => (Side::East, Side::North),
            Corner::SouthEast => (Side::East, Side::South),
            Corner::SouthWest => (Side::West, Side::South),
            Corner::NorthWest => (Side::West, Side::North),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Shrinkwrap, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CornerData<T>(pub Map<Corner, T>);

impl<T> Default for CornerData<T> {
    fn default() -> Self {
        CornerData(Map::new())
    }
}

impl<T> CornerData<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

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
    pub fn cardinal() -> Vec<Self> {
        vec![
            Self::Convex,
            Self::Concave,
            Self::Horizontal,
            Self::Vertical,
        ]
    }

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
