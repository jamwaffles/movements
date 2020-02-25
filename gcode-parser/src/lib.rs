#![deny(intra_doc_link_resolution_failure)]

pub mod word;

struct Coord {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    a: Option<f32>,
    b: Option<f32>,
    c: Option<f32>,
    u: Option<f32>,
    v: Option<f32>,
    w: Option<f32>,
}

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
