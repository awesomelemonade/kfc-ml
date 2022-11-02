use std::ops::{Add, AddAssign, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn round_and_from(x: f32, y: f32) -> Self {
        Self {
            x: x.round() as u32,
            y: y.round() as u32,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Delta {
    pub x: i32,
    pub y: i32,
}

impl Delta {
    pub fn clamp(self, min: i32, max: i32) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
        }
    }
    pub fn dist_linf(self) -> u32 {
        (self.x.abs().max(self.y.abs())) as u32
    }
}

impl Add<Delta> for Position {
    type Output = Position;

    fn add(self, rhs: Delta) -> Position {
        Position {
            x: (self.x as i32 + rhs.x) as u32,
            y: (self.y as i32 + rhs.y) as u32,
        }
    }
}

impl AddAssign<Delta> for Position {
    fn add_assign(&mut self, rhs: Delta) {
        self.x = (self.x as i32 + rhs.x) as u32;
        self.y = (self.y as i32 + rhs.y) as u32;
    }
}

impl Sub for Position {
    type Output = Delta;
    fn sub(self, rhs: Self) -> Delta {
        Delta {
            x: (self.x as i32) - (rhs.x as i32),
            y: (self.y as i32) - (rhs.y as i32),
        }
    }
}

impl From<(u32, u32)> for Position {
    fn from((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Self {
            x: x as u32,
            y: y as u32,
        }
    }
}

impl From<Position> for (f32, f32) {
    fn from(position: Position) -> (f32, f32) {
        (position.x as f32, position.y as f32)
    }
}
