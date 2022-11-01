core!();
use super::*;
use enum_map::Enum;

#[derive(Debug, Enum, Copy, Clone, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Enum, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
    pub state: PieceState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StationaryState {
    pub position: Position,
    pub cooldown: u32, // cooldown counts down
}

impl StationaryState {
    pub fn x(self) -> u32 {
        self.position.x
    }
    pub fn y(self) -> u32 {
        self.position.y
    }
}

impl StationaryState {
    pub fn delta_x(self, state: StationaryState) -> i32 {
        (self.x() as i32) - (state.x() as i32)
    }
    pub fn delta_y(self, state: StationaryState) -> i32 {
        (self.y() as i32) - (state.y() as i32)
    }
    pub fn delta_xy(self, state: StationaryState) -> (i32, i32) {
        (self.delta_x(state), self.delta_y(state))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MovingState {
    x: f32,
    y: f32,
    target: MoveTarget,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceState {
    Stationary(StationaryState),
    Moving(MovingState),
}

impl From<StationaryState> for PieceState {
    fn from(position: StationaryState) -> Self {
        PieceState::Stationary(position)
    }
}

impl From<MovingState> for PieceState {
    fn from(position: MovingState) -> Self {
        PieceState::Moving(position)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MoveTarget {
    pub target: StationaryState, // Stationary Position
    pub turns_left: u32,         // number of turns left to arrive at the target
    // piece that moves first gets precedence (and eats opposing pieces in its path - the path is blocked off for its own pieces for the duration of its move)
    pub priority: u32, // priority gets incremented at every step
}
