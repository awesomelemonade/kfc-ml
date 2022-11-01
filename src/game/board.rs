core!();
use super::*;
use enum_map::EnumMap;

pub struct BoardState {
    pieces: Vec<Piece>,
    can_long_castle: EnumMap<Side, bool>,
    can_short_castle: EnumMap<Side, bool>,
    occupied: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

impl BoardState {
    pub fn can_move(&self, board_move: BoardMove) -> bool {
        fn get_positions_between(start: Position, end: Position) -> Vec<Position> {
            let mut vec = Vec::new();
            let mut current = start;
            loop {
                let step = (end - start).clamp(-1, 1);
                current += step;
                if current == end {
                    break;
                } else {
                    vec.push(current);
                }
            }
            vec
        }
        match board_move {
            BoardMove::Normal { piece, target } => {
                match piece.state {
                    PieceState::Moving { .. } => false,
                    PieceState::Stationary { cooldown, .. } if cooldown > 0 => false,
                    PieceState::Stationary { position, .. } => {
                        if position == target {
                            return false;
                        }
                        let target_piece = self.get_piece(target);

                        if let Some(ref target_piece) = target_piece && target_piece.side == piece.side {
                            return false
                        }
                        match piece.kind {
                            PieceKind::Pawn => {
                                let delta = target - position;
                                let is_capturing_enemy = target_piece
                                    .map_or(false, |target_piece| piece.side != target_piece.side);
                                delta.y == forward_y(piece.side)
                                    && (delta.x == 0 && !is_capturing_enemy)
                                    || (delta.x.abs() == 1 && is_capturing_enemy)
                            }
                            PieceKind::Knight => {
                                let delta = target - position;
                                let abs_x = delta.x.abs();
                                let abs_y = delta.y.abs();
                                (abs_x == 1 && abs_y == 2) || (abs_x == 2 && abs_y == 1)
                            }
                            PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen => {
                                // check that target is not occupied by friendly unit
                                let target_has_friendly = target_piece
                                    .map_or(false, |target_piece| target_piece.side == piece.side);
                                // check if every square in its path is not occupied by unit
                                let path_is_occupied = get_positions_between(position, target)
                                    .iter()
                                    .any(|&pos| self.is_occupied(pos));
                                // check if the piece can go on this delta
                                let delta = target - position;
                                let is_straight = delta.x == 0 || delta.y == 0;
                                let is_diagonal = delta.x.abs() == delta.y.abs();
                                let allowed_delta = match piece.kind {
                                    PieceKind::Bishop => is_diagonal,
                                    PieceKind::Rook => is_straight,
                                    PieceKind::Queen => is_straight || is_diagonal,
                                    _ => false,
                                };
                                !target_has_friendly && !path_is_occupied && allowed_delta
                            }
                            PieceKind::King => {
                                let delta = target - position;
                                delta.x * delta.x + delta.y * delta.y <= 2
                            }
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
            BoardMove::ShortCastle(_side) => {}
            BoardMove::Normal { piece, target } => {
                if let PieceState::Stationary { position, .. } = piece.state {
                    let Position { x, y } = position;
                    let delta = target - position;
                    let target = MoveTarget::new(target, delta.dist_l1(), MoveTarget::MIN_PRIORITY);
                    piece.state = PieceState::Moving {
                        x: x as f32,
                        y: y as f32,
                        target,
                    }
                };
            }
        }
    }
    fn get_piece(&self, position: Position) -> Option<Piece> {
        None // TODO
    }
    fn is_occupied(&self, position: Position) -> bool {
        false // TODO
    }
}

pub enum BoardMove<'a> {
    LongCastle(Side),
    ShortCastle(Side),
    Normal {
        piece: &'a mut Piece,
        target: Position,
    },
}
