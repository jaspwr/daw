use std::ops::{Add, AddAssign};

pub mod element;
pub mod style;
pub mod gl;

#[derive(Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32,
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub fn p(x: f32, y: f32) -> Position {
    Position { x, y }
}

pub enum Size {
    Fixed(f32),
    FractionOfParent(f32),
}

impl Size {
    fn to_size(&self, parent_size: f32) -> f32 {
        match self {
            Size::Fixed(size) => *size,
            Size::FractionOfParent(fraction) => parent_size * fraction,
        }
    }
}

pub struct Dimensions {
    width: Size,
    height: Size,
}

pub struct ComputedDimensions {
    pub width: f32,
    pub height: f32,
}

pub fn d(width: Size, height: Size) -> Dimensions {
    Dimensions { width, height }
}
