use crate::corners::{Corner, CornerType, Side};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Adjacency: u8 {
        const N = 0b0000_0001;
        const S = 0b0000_0010;
        const E = 0b0000_0100;
        const W = 0b0000_1000;
        const NE = 0b0001_0000;
        const SE = 0b0010_0000;
        const SW = 0b0100_0000;
        const NW = 0b1000_0000;
        const N_S = Self::N.bits | Self::S.bits;
        const E_W = Self::E.bits | Self::W.bits;
        const CARDINALS = Self::N.bits | Self::S.bits | Self::E.bits | Self::W.bits;
    }
}

impl From<Corner> for Adjacency {
    fn from(corner: Corner) -> Self {
        Adjacency::from_corner(corner)
    }
}

impl From<Side> for Adjacency {
    fn from(side: Side) -> Self {
        match side {
            Side::North => Adjacency::N,
            Side::South => Adjacency::S,
            Side::East => Adjacency::E,
            Side::West => Adjacency::W,
        }
    }
}

impl Adjacency {
    /// Returns an array of the cardinal directions in the order used by DMI
    pub const fn dmi_cardinals() -> [Adjacency; 4] {
        [Adjacency::S, Adjacency::N, Adjacency::E, Adjacency::W]
    }

    pub const fn corner_sides(&self) -> (Adjacency, Adjacency) {
        match *self {
            Adjacency::NE => (Adjacency::N, Adjacency::E),
            Adjacency::SE => (Adjacency::S, Adjacency::E),
            Adjacency::SW => (Adjacency::S, Adjacency::W),
            Adjacency::NW => (Adjacency::N, Adjacency::W),
            _ => panic!("Not a corner!"),
        }
    }

    // implemented as const for usage in get corner type
    const fn from_corner(corner: Corner) -> Self {
        match corner {
            Corner::NorthEast => Adjacency::NE,
            Corner::SouthEast => Adjacency::SE,
            Corner::SouthWest => Adjacency::SW,
            Corner::NorthWest => Adjacency::NW,
        }
    }

    pub const fn get_corner_type(&self, corner: Corner) -> CornerType {
        let adj_corner: Adjacency = Adjacency::from_corner(corner);
        let (vertical, horizontal) = adj_corner.corner_sides();
        // It should only flat smooth if cardinals are filled too
        if self.contains(vertical) && self.contains(horizontal) {
            if self.contains(adj_corner) {
                CornerType::Flat
            } else {
                CornerType::Concave
            }
        } else if self.contains(vertical) {
            // Since we don't have both, it must be exclusive meaning horizontal doesn't need to be checked
            CornerType::Vertical
        } else if self.contains(horizontal) {
            // Ditto as above, but now for horizontal
            CornerType::Horizontal
        } else {
            CornerType::Convex
        }
    }
}
