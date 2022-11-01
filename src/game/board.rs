core!();
use enum_map::{EnumMap};
use super::{BOARD_SIZE, Piece, Side, PieceKind, Position};

struct BoardState {
    pieces: Vec<Piece>,
    can_long_castle: EnumMap<Side, bool>,
    can_short_castle: EnumMap<Side, bool>,
    occupied: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

impl BoardState {
    fn can_move(&self, board_move: BoardMove) -> bool {
        match board_move {
            BoardMove::Normal { piece, target } => match piece.kind {
                PieceKind::Pawn => {true}
                _ => false,
            },
            _ => false,
        }
    }
    fn apply_move(&mut self, board_move: BoardMove) {
        match board_move {
            BoardMove::LongCastle(side) => {
                // TODO
                // let rook_position = match side {
                //     Side::White => (0f32, 0f32),
                //     Side::Black => (0f32, 0f32),
                // };
                // get starting positions of rook and king
            }
            BoardMove::ShortCastle(side) => {}
            BoardMove::Normal { piece, target } => {}
        }
    }
    fn get_piece(&self, position: Position) {}
}

enum BoardMove {
    LongCastle(Side),
    ShortCastle(Side),
    Normal { piece: Piece, target: Position },
}
