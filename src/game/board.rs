core!();
use super::*;
use enum_map::EnumMap;

struct BoardState {
    pieces: Vec<Piece>,
    can_long_castle: EnumMap<Side, bool>,
    can_short_castle: EnumMap<Side, bool>,
    occupied: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

impl BoardState {
    fn can_move(&self, board_move: BoardMove) -> bool {
        match board_move {
            BoardMove::Normal { piece, target } => {
                match piece.state {
                    PieceState::Moving { .. } => false,
                    PieceState::Stationary { cooldown, .. } if cooldown > 0 => false,
                    PieceState::Stationary { position, .. } => {
                        match piece.kind {
                            PieceKind::Pawn => {
                                let delta = target - position;
                                let is_capturing = self
                                    .get_piece(target)
                                    .map_or(false, |target_piece| piece.side != target_piece.side);
                                delta.y == forward_y(piece.side) && (delta.x == 0 && !is_capturing)
                                    || (delta.x.abs() == 1 && is_capturing)
                            }
                            PieceKind::Knight => {
                                let delta = target - position;
                                let abs_x = delta.x.abs();
                                let abs_y = delta.y.abs();
                                (abs_x == 1 && abs_y == 2) || (abs_x == 2 && abs_y == 1)
                            }
                            _ => false, // TODO: do other pieces
                        }
                    }
                }
            }
            BoardMove::LongCastle(_side) => false, // TODO: do castling
            BoardMove::ShortCastle(_side) => false, // TODO: do castling
        }
    }
    fn apply_move(&mut self, board_move: BoardMove) {
        match board_move {
            BoardMove::LongCastle(_side) => {
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
    fn get_piece(&self, position: Position) -> Option<Piece> {
        None // TODO
    }
    fn is_occupied(&self, position: Position) -> bool {
        false // TODO
    }
}

enum BoardMove {
    LongCastle(Side),
    ShortCastle(Side),
    Normal { piece: Piece, target: Position },
}
