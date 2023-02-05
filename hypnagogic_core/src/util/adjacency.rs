use crate::util::corners::{Corner, CornerType, Side};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[allow(clippy::unsafe_derive_deserialize)]
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
    #[must_use]
    pub const fn dmi_cardinals() -> [Adjacency; 4] {
        [Adjacency::S, Adjacency::N, Adjacency::E, Adjacency::W]
    }

    /// Gets the sides for a given corner adjacency
    /// Adjacency is always returned in the format of `(Vertical, Horizontal)`
    /// # Panics
    /// Panics when a non-corner adjacency is passed in
    #[must_use]
    pub const fn corner_sides(self) -> (Adjacency, Adjacency) {
        match self {
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

    #[must_use]
    pub fn set_flags_vec(self) -> Vec<Self> {
        let full = [
            Adjacency::N,
            Adjacency::S,
            Adjacency::E,
            Adjacency::W,
            Adjacency::NE,
            Adjacency::SE,
            Adjacency::SW,
            Adjacency::NW,
        ];
        full.into_iter().filter(|a| self.contains(*a)).collect()
    }

    #[must_use]
    pub const fn get_corner_type(self, corner: Corner) -> CornerType {
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

    #[must_use]
    pub fn rotate_dir(self, direction: Self) -> Self {
        match direction {
            // 180 degree rotation
            Adjacency::N => match self {
                Adjacency::N => Adjacency::S,
                Adjacency::S => Adjacency::N,
                Adjacency::E => Adjacency::W,
                Adjacency::W => Adjacency::E,
                Adjacency::NE => Adjacency::SW,
                Adjacency::SE => Adjacency::NW,
                Adjacency::SW => Adjacency::NE,
                Adjacency::NW => Adjacency::SE,
                _ => unimplemented!("Only single allowed"),
            },
            // No rotation needed!
            Adjacency::S => self,
            // Counter-clockwise 90 degrees
            Adjacency::E => match self {
                Adjacency::N => Adjacency::W,
                Adjacency::S => Adjacency::E,
                Adjacency::E => Adjacency::N,
                Adjacency::W => Adjacency::S,
                Adjacency::NE => Adjacency::NW,
                Adjacency::SE => Adjacency::NE,
                Adjacency::SW => Adjacency::SE,
                Adjacency::NW => Adjacency::SW,
                _ => unimplemented!("Only single allowed"),
            },
            // Clockwise 90 degrees
            Adjacency::W => match self {
                Adjacency::N => Adjacency::E,
                Adjacency::S => Adjacency::W,
                Adjacency::E => Adjacency::S,
                Adjacency::W => Adjacency::N,
                Adjacency::NE => Adjacency::SE,
                Adjacency::SE => Adjacency::SW,
                Adjacency::SW => Adjacency::NW,
                Adjacency::NW => Adjacency::NE,
                _ => unimplemented!("Only single allowed"),
            },
            _ => unimplemented!(
                "Rotating to diagonals doesn't make sense. This is a programming error."
            ),
        }
    }

    #[must_use]
    pub fn rotate_to(self, direction: Self) -> Self {
        self.set_flags_vec()
            .into_iter()
            .map(|x| x.rotate_dir(direction))
            .reduce(|accum, item| accum | item)
            .unwrap_or(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_flags_vec_test() {
        let adj = Adjacency::N | Adjacency::S | Adjacency::W;

        let result = adj.set_flags_vec();

        let expected = vec![Adjacency::N, Adjacency::W, Adjacency::S];

        assert!(expected.iter().all(|item| result.contains(item)));
    }
}
