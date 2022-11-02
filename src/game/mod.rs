core!();
use enum_map::{enum_map, EnumMap};

mod piece;
pub use piece::*;

mod board;
pub use board::*;

mod position;
pub use position::*;

mod board_util;
pub use board_util::*;

#[cfg(test)]
mod tests;

pub const BOARD_SIZE: usize = 8;
pub const PIECE_COOLDOWN: u32 = 10;

lazy_static! {
    pub static ref INITIAL_WHITE_PIECES: EnumMap<PieceKind, Vec<Position>> = {
        let initial_board: String = r#"
            ........
            ........
            ........
            ........
            ........
            ........
            PPPPPPPP
            RNBQKBNR
        "#
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
        let find_locations = |target_c: char| {
            initial_board
                .char_indices()
                .filter_map(move |(i, c)| {
                    if c == target_c {
                        Some(((i % BOARD_SIZE), (i / BOARD_SIZE)).into())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Position>>()
        };
        let pieces = enum_map! {
            PieceKind::Rook => 'R',
            PieceKind::Knight => 'N',
            PieceKind::Bishop => 'B',
            PieceKind::Queen => 'Q',
            PieceKind::King => 'K',
            PieceKind::Pawn => 'P',
        };
        pieces.map(|_, c| find_locations(c))
    };
    pub static ref INITIAL_BLACK_PIECES: EnumMap<PieceKind, Vec<Position>> =
        INITIAL_WHITE_PIECES.clone().map(|_, v| v
            .iter()
            .map(|&Position { x, y }| (x, BOARD_SIZE as u32 - y - 1u32).into())
            .collect());
}
