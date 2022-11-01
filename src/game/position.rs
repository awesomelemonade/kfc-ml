use std::ops::Sub;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Delta {
    pub x: i32,
    pub y: i32,
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
