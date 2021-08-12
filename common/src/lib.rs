use nalgebra::SVector;

pub type Number = f32;

pub type Position = SVector<Number, 9>;

pub enum Axis {
    X,
    Y,
    Z,
    A,
    B,
    C,
    U,
    V,
    W,
}

pub enum Command {
    Position { axis: Axis, value: Number },
    Motion(Motion),
}

pub enum Motion {
    Rapid,
    Feed,
}
