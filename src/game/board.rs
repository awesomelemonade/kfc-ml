core!();

use super::*;
use enum_map::EnumMap;

#[derive(Debug)]
pub struct BoardState {
    pieces: Vec<Piece>,
    can_long_castle: EnumMap<Side, bool>,
    can_short_castle: EnumMap<Side, bool>,
}

impl BoardState {
    pub fn new_initial_state() -> Self {
        let mut pieces = Vec::new();
        fn append_pieces(
            pieces: &mut Vec<Piece>,
            side: Side,
            map: &EnumMap<PieceKind, Vec<Position>>,
        ) {
            for (kind, positions) in map {
                pieces.extend(positions.iter().map(|&position| Piece {
                    side,
                    kind,
                    state: PieceState::Stationary {
                        position,
                        cooldown: 0u32,
                    },
                }));
            }
        }
        append_pieces(&mut pieces, Side::White, &*INITIAL_WHITE_PIECES);
        append_pieces(&mut pieces, Side::Black, &*INITIAL_BLACK_PIECES);
        const ENABLE_CASTLING: bool = false;
        Self {
            pieces,
            // TODO: Support Castling
            can_long_castle: enum_map! {
                Side::White => ENABLE_CASTLING,
                Side::Black => ENABLE_CASTLING,
            },
            can_short_castle: enum_map! {
                Side::White => ENABLE_CASTLING,
                Side::Black => ENABLE_CASTLING,
            },
        }
    }
    pub fn can_move(&self, board_move: &BoardMove) -> bool {
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
                let target = *target;
                match piece.state {
                    PieceState::Moving { .. } => false,
                    PieceState::Stationary { cooldown, .. } if cooldown > 0 => false,
                    PieceState::Stationary { position, .. } => {
                        if position == target {
                            return false;
                        }
                        let target_piece =
                            self.get_piece_including_moving_on_side(target, piece.side);

                        // check that target is not occupied by friendly unit
                        // Ensure that no moving pieces has this target as the target
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
                                // check if every square in its path is not occupied by unit
                                let path_is_occupied =
                                    get_positions_between(position, target).iter().any(|&pos| {
                                        self.get_piece_including_moving_on_side(pos, piece.side)
                                            .is_some()
                                    });
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
                                !path_is_occupied && allowed_delta
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
    pub fn apply_move(&mut self, board_move: BoardMove) {
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
                    let target =
                        MoveTarget::new(target, delta.dist_linf(), MoveTarget::MIN_PRIORITY);
                    // TODO: somehow optimize so we don't have to loop through?
                    if let Some(piece) = self.get_stationary_piece_mut(position) {
                        piece.state = PieceState::Moving {
                            x: x as f32,
                            y: y as f32,
                            target,
                        }
                    }
                    // TODO: invalidate any castling if needed
                };
            }
        }
    }
    fn get_stationary_piece(&self, position: Position) -> Option<&Piece> {
        self.pieces.iter().find(|piece| match piece.state {
            PieceState::Stationary {
                position: piece_position,
                ..
            } => position == piece_position,
            PieceState::Moving { .. } => false,
        })
    }
    fn get_stationary_piece_mut(&mut self, position: Position) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|piece| match piece.state {
            PieceState::Stationary {
                position: piece_position,
                ..
            } => position == piece_position,
            PieceState::Moving { .. } => false,
        })
    }
    fn get_piece_including_moving_on_side(&self, position: Position, side: Side) -> Option<&Piece> {
        self.pieces.iter().find(|piece| match piece.state {
            PieceState::Stationary {
                position: piece_position,
                ..
            } => position == piece_position,
            PieceState::Moving {
                target: MoveTarget { target, .. },
                ..
            } => side == piece.side && position == target,
        })
    }
    pub fn step(&mut self) {
        fn position_after_step(piece_state: &PieceState, step_size: f32) -> (f32, f32) {
            match piece_state {
                PieceState::Stationary { position, .. } => (position.x as f32, position.y as f32),
                PieceState::Moving {
                    x,
                    y,
                    target:
                        MoveTarget {
                            target, turns_left, ..
                        },
                } => {
                    let progress = step_size / (*turns_left as f32);
                    let new_x = (target.x as f32 - x) * progress + x;
                    let new_y = (target.y as f32 - y) * progress + y;
                    (new_x, new_y)
                }
            }
        }
        fn intersects((x, y): (f32, f32), (x2, y2): (f32, f32)) -> bool {
            let dx = x - x2;
            let dy = y - y2;
            dx * dx + dy * dy <= 1f32
        }
        fn pieces_intersect(a: &Piece, b: &Piece) -> bool {
            // TODO: can be made continuous
            let a2 = position_after_step(&a.state, 0.5f32);
            let b2 = position_after_step(&b.state, 0.5f32);
            let a3 = position_after_step(&a.state, 1f32);
            let b3 = position_after_step(&b.state, 1f32);
            intersects(a2, b2) || intersects(a3, b3)
        }
        fn get_priority(piece: &Piece) -> u32 {
            match piece.state {
                PieceState::Stationary { .. } => 0u32,
                PieceState::Moving {
                    target: MoveTarget { priority, .. },
                    ..
                } => priority + 1u32,
            }
        }
        fn can_be_captured(priority: u32, new_position: (f32, f32), capturer: &Piece) -> bool {
            if priority <= get_priority(capturer) {
                match capturer.kind {
                    // if the piece is a knight, it never intersects unless this is the moving knight's target
                    PieceKind::Knight => match capturer.state {
                        PieceState::Stationary { .. } => false,
                        PieceState::Moving {
                            target: MoveTarget { target, .. },
                            ..
                        } => intersects(new_position, target.into()),
                    },
                    _ => true,
                }
            } else {
                false
            }
        }
        // Two moving pieces with the same priority needs to both get captured
        self.pieces = self
            .pieces
            .iter()
            .filter_map(|piece| {
                let priority = get_priority(piece);
                let new_position = position_after_step(&piece.state, 1f32);
                // check if any intersect
                let intersects = self.pieces.iter().any(|capturer| {
                    piece.side != capturer.side
                        && can_be_captured(priority, new_position, capturer)
                        && pieces_intersect(piece, capturer)
                });
                if intersects {
                    None
                } else {
                    let new_state = match piece.state {
                        PieceState::Stationary { position, cooldown } => PieceState::Stationary {
                            position,
                            cooldown: cooldown.saturating_sub(1),
                        },
                        PieceState::Moving {
                            target:
                                MoveTarget {
                                    target,
                                    turns_left,
                                    priority,
                                },
                            ..
                        } => {
                            if turns_left == 1 {
                                PieceState::Stationary {
                                    position: target,
                                    cooldown: PIECE_COOLDOWN,
                                }
                            } else {
                                let (new_x, new_y) = new_position;
                                PieceState::Moving {
                                    x: new_x,
                                    y: new_y,
                                    target: MoveTarget {
                                        target,
                                        turns_left: turns_left - 1,
                                        priority: priority + 1,
                                    },
                                }
                            }
                        }
                    };
                    Some(Piece {
                        side: piece.side,
                        kind: piece.kind,
                        state: new_state,
                    })
                }
            })
            .collect();
    }
    pub fn get_all_possible_moves(&self, side: Side) -> Vec<BoardMove> {
        let mut moves: Vec<BoardMove> = Vec::new();
        // handle castling
        if self.can_long_castle[side] {
            moves.push(BoardMove::LongCastle(side));
        }
        if self.can_short_castle[side] {
            moves.push(BoardMove::ShortCastle(side));
        }
        for piece in self.pieces.iter().cloned() {
            if piece.side == side {
                // loop through all squares
                for i in 0..BOARD_SIZE {
                    for j in 0..BOARD_SIZE {
                        let target = (i, j).into();
                        let board_move = BoardMove::Normal { piece, target };
                        if self.can_move(&board_move) {
                            moves.push(board_move);
                        }
                    }
                }
            }
        }
        moves
    }
    pub fn to_stationary_map<F>(&self, default_char: char, f: F) -> String
    where
        F: Fn(&Piece) -> char,
    {
        let mut all: Vec<String> = Vec::new();
        for row in 0..BOARD_SIZE {
            let mut buffer = Vec::new();
            for col in 0..BOARD_SIZE {
                let piece = self.get_stationary_piece((col, row).into());
                let c = piece.map_or(default_char, &f);
                buffer.push(c);
            }
            all.push(buffer.iter().collect());
        }
        all.join("\n")
    }
    pub fn to_stationary_map_type(&self) -> String {
        self.to_stationary_map('.', |piece| piece.kind.into())
    }
    pub fn to_stationary_map_color(&self) -> String {
        self.to_stationary_map('.', |piece| match piece.side {
            Side::White => 'W',
            Side::Black => 'B',
        })
    }
}

#[derive(Debug)]
pub enum BoardMove {
    LongCastle(Side),
    ShortCastle(Side),
    Normal { piece: Piece, target: Position },
}
