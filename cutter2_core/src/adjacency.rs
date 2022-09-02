use crate::corners::{Corner, CornerType};
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

impl Adjacency {
    /// Returns an array of the cardinal directions in the order used by DMI
    pub const fn dmi_cardinals() -> [Adjacency; 4] {
        [Adjacency::S, Adjacency::N, Adjacency::E, Adjacency::W]
    }

    pub const fn corner_sides(corner: &Corner) -> [Adjacency; 2] {
        match corner {
            Corner::NorthEast => [Adjacency::N, Adjacency::E],
            Corner::SouthEast => [Adjacency::S, Adjacency::E],
            Corner::SouthWest => [Adjacency::S, Adjacency::W],
            Corner::NorthWest => [Adjacency::N, Adjacency::W],
        }
    }

    pub const fn get_corner_type(&self, corner: Corner) -> CornerType {
        //I can see a pattern here but extracting the functionality seemed less readable
        match corner {
            Corner::NorthEast => {
                // It should only flat smooth if cardinals are filled too
                if self.contains(Self::N) && self.contains(Self::E) {
                    if self.contains(Self::NE) {
                        CornerType::Flat
                    } else {
                        CornerType::Concave
                    }
                // Since we don't have
                } else if self.contains(Self::N) {
                    CornerType::Horizontal
                } else if self.contains(Self::E) {
                    CornerType::Vertical
                } else {
                    CornerType::Convex
                }
            }
            Corner::SouthEast => {
                todo!()
            }
            Corner::SouthWest => {
                todo!()
            }
            Corner::NorthWest => {
                todo!()
            }
        }
    }
}
