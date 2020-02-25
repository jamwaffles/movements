#![deny(intra_doc_link_resolution_failure)]

pub mod coord;
pub mod word;

use crate::coord::Coord;

enum Motion {
    /// G0
    Rapid,

    /// G1
    Feed,
}

enum Token {
    /// Group 1
    Motion(Motion),

    /// Coordinate
    Coord(Coord),
}

#[cfg(test)]
mod tests {
    #[test]
    fn rapid_move() {
        //
    }
}
