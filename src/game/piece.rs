core!();
use enum_map::{Enum};

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

pub type Position = (f32, f32);

pub struct MoveTarget {
    pub target: Position,
    pub turns_left: u32,
    // piece that moves first gets precedence (and eats opposing pieces in its path - the path is blocked off for its own pieces for the duration of its move)
    pub priority: u32, // priority gets incremented at every step
}
pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
    pub position: Position,
    pub moving_target: Option<MoveTarget>, // None means the piece is not moving
}
