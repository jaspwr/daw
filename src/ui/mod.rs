use std::{ops::{Add, AddAssign}, cell::RefCell};

use crate::global::Globals;

pub mod element;
pub mod frame_buf;
pub mod gl;
pub mod reactive;
pub mod style;
pub mod text;

#[derive(Copy, Clone, Debug)]
pub enum Coordinate {
    Fixed(f32),
    FractionOfParent(f32),
    FractionOfParentWithOffset(f32, f32),
}

impl Coordinate {
    pub fn compute(&self, parent_size: &f32) -> f32 {
        match self {
            Coordinate::Fixed(pos) => *pos,
            Coordinate::FractionOfParent(fraction) => parent_size * fraction,
            Coordinate::FractionOfParentWithOffset(fraction, offset) => {
                parent_size * fraction + offset
            }
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: Coordinate,
    pub y: Coordinate,
}

impl Position {
    pub fn origin() -> Self {
        Self {
            x: Coordinate::Fixed(0.),
            y: Coordinate::Fixed(0.),
        }
    }

    pub fn compute(&self, parent_dims: &ComputedDimensions) -> ComputedPosition {
        ComputedPosition {
            x: self.x.compute(&parent_dims.width),
            y: self.y.compute(&parent_dims.height),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComputedPosition {
    x: f32,
    y: f32,
}

impl ComputedPosition {
    pub fn origin() -> Self {
        Self { x: 0., y: 0. }
    }
}

impl Add for ComputedPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        ComputedPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for ComputedPosition {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub fn p(x: f32, y: f32) -> Position {
    Position {
        x: Coordinate::Fixed(x),
        y: Coordinate::Fixed(y),
    }
}

pub fn p_c(x: f32, y: f32) -> ComputedPosition {
    ComputedPosition { x, y }
}

pub enum Size {
    Fixed(f32),
    FractionOfParent(f32),
    FractionOfParentWithOffset(f32, f32),
}

impl Size {
    fn to_size(&self, parent_size: f32) -> f32 {
        match self {
            Size::Fixed(size) => *size,
            Size::FractionOfParent(fraction) => parent_size * fraction,
            Size::FractionOfParentWithOffset(fraction, offset) => parent_size * fraction + offset,
        }
    }
}

pub struct Dimensions {
    pub width: Size,
    pub height: Size,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputedDimensions {
    pub width: f32,
    pub height: f32,
}

pub fn compute_dims(dims: &Dimensions, parent_dims: &ComputedDimensions) -> ComputedDimensions {
    ComputedDimensions {
        width: dims.width.to_size(parent_dims.width),
        height: dims.height.to_size(parent_dims.height),
    }
}

pub fn d(width: f32, height: f32) -> Dimensions {
    Dimensions {
        width: Size::Fixed(width),
        height: Size::Fixed(height),
    }
}