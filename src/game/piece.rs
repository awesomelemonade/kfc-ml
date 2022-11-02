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

impl PieceKind {
    pub fn from_char(value: char) -> Option<PieceKind> {
        match value {
            'P' => Some(PieceKind::Pawn),
            'N' => Some(PieceKind::Knight),
            'B' => Some(PieceKind::Bishop),
            'R' => Some(PieceKind::Rook),
            'Q' => Some(PieceKind::Queen),
            'K' => Some(PieceKind::King),
            _ => None,
        }
    }
}

impl From<PieceKind> for char {
    fn from(kind: PieceKind) -> Self {
        match kind {
            PieceKind::Pawn => 'P',
            PieceKind::Knight => 'N',
            PieceKind::Bishop => 'B',
            PieceKind::Rook => 'R',
            PieceKind::Queen => 'Q',
            PieceKind::King => 'K',
        }
    }
}

#[derive(Debug, Enum, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

#[derive(Debug, Copy, Clone)]
pub struct Piece {
    pub side: Side,
    pub kind: PieceKind,
    pub state: PieceState,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceState {
    Stationary { position: Position, cooldown: u32 },
    Moving { x: f32, y: f32, target: MoveTarget },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MoveTarget {
    pub target: Position, // Stationary Position
    pub turns_left: u32,  // number of turns left to arrive at the target
    // piece that moves first gets precedence (and eats opposing pieces in its path - the path is blocked off for its own pieces for the duration of its move)
    pub priority: u32, // priority gets incremented at every step
}

impl MoveTarget {
    pub const MIN_PRIORITY: u32 = 0;
    pub fn new(target: Position, turns_left: u32, priority: u32) -> Self {
        Self {
            target,
            turns_left,
            priority,
        }
    }
}
