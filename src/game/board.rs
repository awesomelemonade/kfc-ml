core!();

use core::slice;

use super::*;
use enum_map::EnumMap;
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};

#[derive(Debug, Clone)]
pub struct BoardState {
    pieces: Vec<Piece>,
    can_long_castle: EnumMap<Side, bool>,
    can_short_castle: EnumMap<Side, bool>,
}

pub static mut Q_COUNT: u32 = 0;
pub static mut S_COUNT: u32 = 0;

// has to be less than sqrt(2)/2 to ensure bishops do not capture squares
//                                          it is not supposed to capture
const DISTANCE_THRESHOLD_SQUARED: f32 = 0.7f32 * 0.7f32;

impl BoardState {
    pub fn pieces(&self) -> &Vec<Piece> {
        &self.pieces
    }

    // TODO-someday: For_test module?
    pub fn pieces_mut(&mut self) -> &mut Vec<Piece> {
        &mut self.pieces
    }

    fn new_with_castling(pieces: Vec<Piece>, enable_castling: bool) -> Self {
        Self {
            pieces,
            can_long_castle: enum_map! {
                Side::White => enable_castling,
                Side::Black => enable_castling,
            },
            can_short_castle: enum_map! {
                Side::White => enable_castling,
                Side::Black => enable_castling,
            },
        }
    }

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
        append_pieces(&mut pieces, Side::White, &INITIAL_WHITE_PIECES);
        append_pieces(&mut pieces, Side::Black, &INITIAL_BLACK_PIECES);
        // TODO: Support Castling
        Self::new_with_castling(pieces, false)
    }
    pub fn can_move(&self, board_move: &BoardMove) -> bool {
        fn get_positions_between(start: Position, end: Position) -> Vec<Position> {
            let mut vec = Vec::new();
            let mut current = start;
            loop {
                let step = (end - current).clamp(-1, 1);
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
            BoardMove::None(_side) => true,
            BoardMove::Normal { piece, target } => {
                let target = *target;
                match piece.state {
                    PieceState::Moving { .. } => false,
                    PieceState::Stationary { cooldown, .. } if cooldown > 0 => false,
                    PieceState::Stationary { position, .. } => {
                        if position == target {
                            return false;
                        }
                        // check that target is not occupied by friendly unit
                        // Ensure that no moving pieces has this target as the target
                        if !self.is_valid_destination(piece.side, target) {
                            return false;
                        }
                        let target_piece = self.get_stationary_piece(target); // enemy piece
                        debug_assert!(
                            target_piece.map_or(true, |enemy_piece| enemy_piece.side != piece.side)
                        );
                        match piece.kind {
                            PieceKind::Pawn => {
                                let delta = target - position;
                                let is_capturing_enemy = target_piece
                                    .map_or(false, |target_piece| piece.side != target_piece.side);
                                let can_normal_move = delta.y == forward_y(piece.side)
                                    && ((delta.x == 0 && !is_capturing_enemy)
                                        || (delta.x.abs() == 1 && is_capturing_enemy));
                                let is_in_starting_rank = position.y
                                    == match piece.side {
                                        Side::White => BOARD_SIZE as u32 - 2,
                                        Side::Black => 1,
                                    };
                                let can_double_move = delta.x == 0
                                    && delta.y == forward_y(piece.side) * 2
                                    && !is_capturing_enemy
                                    && is_in_starting_rank;
                                can_normal_move || can_double_move
                            }
                            PieceKind::Knight => {
                                let delta = target - position;
                                let abs_x = delta.x.abs();
                                let abs_y = delta.y.abs();
                                (abs_x == 1 && abs_y == 2) || (abs_x == 2 && abs_y == 1)
                            }
                            PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen => {
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
                                if allowed_delta {
                                    // check if every square in its path is not occupied by unit
                                    let path_is_occupied = get_positions_between(position, target)
                                        .iter()
                                        .any(|&pos| {
                                            self.get_piece_including_moving_on_side(pos, piece.side)
                                                .is_some()
                                        });
                                    !path_is_occupied
                                } else {
                                    false
                                }
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
    pub fn apply_move(&mut self, board_move: &BoardMove) {
        match board_move {
            BoardMove::None(_side) => {}
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
                let target = *target;
                // ensure that the piece is not already moving
                debug_assert!(piece.state.is_stationary());
                // ensure that the target is not of a piece on our own side
                debug_assert!(self.is_valid_destination(piece.side, target));
                if let PieceState::Stationary { position, .. } = piece.state {
                    let Position { x, y } = position;
                    let delta = target - position;
                    let target = MoveTarget::new(
                        position,
                        target,
                        delta.dist_linf(),
                        MoveTarget::MIN_PRIORITY,
                    );
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
        // TODO-someday: Return some stationary piece type instead of just piece?
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
    pub fn step_until_one_becomes_stationary(&mut self) -> bool {
        let min_turns_left = self
            .pieces
            .iter()
            .filter_map(|piece| {
                if let PieceState::Moving {
                    target: MoveTarget { turns_left, .. },
                    ..
                } = piece.state
                {
                    Some(turns_left)
                } else {
                    None
                }
            })
            .min();
        if let Some(min_turns_left) = min_turns_left {
            self.step_n(min_turns_left);
            true
        } else {
            false
        }
    }
    fn handle_intersections_for_singular_moving_piece(&mut self, n: f32) {
        fn is_within_distance_squared(
            p @ (x, y): (f32, f32),
            v @ (vx, vy): (f32, f32),
            n: f32,
        ) -> bool {
            let t = get_t_clamped(p, v, n);
            let intersection_x = x + vx * t;
            let intersection_y = y + vy * t;
            intersection_x * intersection_x + intersection_y * intersection_y
                <= DISTANCE_THRESHOLD_SQUARED
        }
        fn get_t_clamped((x, y): (f32, f32), (vx, vy): (f32, f32), n: f32) -> f32 {
            // at what t will p + v * t be closest to (0, 0)?
            // t = (-px * vx - py * vy) / (vx^2 + vy^2)
            ((-x * vx - y * vy) / (vx * vx + vy * vy)).clamp(0f32, n)
        }
        // we get the moving piece, analyze the target, and remove pieces
        let moving_piece = self
            .pieces
            .iter()
            .find(|piece| piece.state.is_moving())
            .cloned();
        if let Some(moving_piece) = moving_piece &&
            let PieceState::Moving {
            x: capturer_x,
            y: capturer_y,
            target:
                MoveTarget {
                    velocity: (capturer_vx, capturer_vy),
                    turns_left,
                    ..
                },
        } = moving_piece.state
        {
            self.pieces.retain(|piece| {
                if piece.side != moving_piece.side &&
                    let PieceState::Stationary { position, .. } = piece.state {
                    // will this piece get retained in the next n steps?
                    // will capturer (x, y) + t * (vx, vy) intersect with position?
                    let delta_x = capturer_x - (position.x as f32);
                    let delta_y = capturer_y - (position.y as f32);
                    !is_within_distance_squared(
                        (delta_x, delta_y),
                        (capturer_vx, capturer_vy),
                        n.min(turns_left as f32),
                    )
                } else {
                    true
                }
            });
        }
    }
    pub fn step_until_stationary_with_no_cooldown(&mut self) {
        // TODO: check intersections among moving pieces
        // currently just a conservative heuristic
        let no_intersections = self
            .pieces
            .iter()
            .filter(|piece| piece.state.is_moving())
            .count()
            <= 1;
        if no_intersections {
            // teleport all pieces to the end
            self.handle_intersections_for_singular_moving_piece(f32::MAX);
            self.advance_pieces_by_time(u32::MAX);
        } else {
            self.step_without_moves();
            self.step_until_stationary_with_no_cooldown();
        }
    }
    pub fn step_n(&mut self, n: u32) {
        let no_intersections = self
            .pieces
            .iter()
            .filter(|piece| piece.state.is_moving())
            .count()
            <= 1;
        if no_intersections {
            // teleport all pieces to the end
            self.handle_intersections_for_singular_moving_piece(n as f32);
            self.advance_pieces_by_time(n);
            debug_assert!(!self.has_overlapping_pieces());
        } else {
            self.step_without_moves();
            if n > 1 {
                self.step_n(n - 1);
            }
        }
    }
    pub fn step_without_moves(&mut self) {
        self.step(&BoardMove::None(Side::White), &BoardMove::None(Side::Black));
    }
    pub fn step(&mut self, white_move: &BoardMove, black_move: &BoardMove) {
        unsafe {
            S_COUNT += 1;
        }
        debug_assert!(white_move.side() == Side::White);
        debug_assert!(black_move.side() == Side::Black);
        self.apply_move(white_move);
        self.apply_move(black_move);
        fn position_after_step(piece_state: &PieceState) -> (f32, f32) {
            match piece_state {
                PieceState::Stationary { position, .. } => (position.x as f32, position.y as f32),
                PieceState::Moving {
                    x,
                    y,
                    target:
                        MoveTarget {
                            velocity: (vx, vy), ..
                        },
                } => (x + vx, y + vy),
            }
        }
        fn intersects((x, y): (f32, f32), (x2, y2): (f32, f32)) -> bool {
            let dx = x - x2;
            let dy = y - y2;
            dx * dx + dy * dy <= DISTANCE_THRESHOLD_SQUARED
        }
        fn piece_will_be_captured(piece: &Piece, capturer: &Piece) -> bool {
            // TODO: maybe we can short circuit if the pieces are both very far away
            fn is_within_distance_squared(
                p @ (x, y): (f32, f32),
                v @ (vx, vy): (f32, f32),
            ) -> bool {
                let t = get_t_clamped(p, v);
                let intersection_x = x + vx * t;
                let intersection_y = y + vy * t;
                intersection_x * intersection_x + intersection_y * intersection_y
                    <= DISTANCE_THRESHOLD_SQUARED
            }
            fn get_t_clamped((x, y): (f32, f32), (vx, vy): (f32, f32)) -> f32 {
                // at what t will p + v * t be closest to (0, 0)?
                // t = (-px * vx - py * vy) / (vx^2 + vy^2)
                ((-x * vx - y * vy) / (vx * vx + vy * vy)).clamp(0f32, 1f32)
            }
            match (piece.state, capturer.state) {
                (_, PieceState::Stationary { .. }) => false,
                (
                    PieceState::Stationary { position, .. },
                    PieceState::Moving {
                        x,
                        y,
                        target: MoveTarget { velocity, .. },
                    },
                ) => {
                    let px = x - position.x as f32;
                    let py = y - position.y as f32;
                    // optimization: pieces cannot move more than 1 diagonal square
                    if px * px + py * py >= 1.4143 * 1.4143 {
                        return false;
                    }
                    is_within_distance_squared((px, py), velocity)
                }
                (
                    PieceState::Moving {
                        x,
                        y,
                        target:
                            MoveTarget {
                                velocity: (vx, vy), ..
                            },
                    },
                    PieceState::Moving {
                        x: x2,
                        y: y2,
                        target:
                            MoveTarget {
                                velocity: (vx2, vy2),
                                ..
                            },
                    },
                ) => {
                    let diff_x = x2 - x;
                    let diff_y = y2 - y;
                    // optimization: pieces cannot move more than 2 diagonal squares relaltive to another piece
                    // (2 * sqrt(2)) ^ 2 = 8
                    if diff_x * diff_x + diff_y * diff_y > 8f32 {
                        return false;
                    }
                    let v_diff_x = vx2 - vx;
                    let v_diff_y = vy2 - vy;
                    is_within_distance_squared((diff_x, diff_y), (v_diff_x, v_diff_y))
                }
            }
        }
        fn get_priority(piece: &Piece) -> i32 {
            match piece.state {
                PieceState::Stationary { .. } => -1i32,
                PieceState::Moving {
                    target: MoveTarget { priority, .. },
                    ..
                } => priority as i32,
            }
        }
        fn can_be_captured(priority: i32, new_position: (f32, f32), capturer: &Piece) -> bool {
            priority <= get_priority(capturer)
                && match capturer.kind {
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
        }
        // TODO: likely not the ideal solution; probably should create Others class and implement IntoIterator
        type Others<'a, T> = core::iter::Chain<core::slice::Iter<'a, T>, core::slice::Iter<'a, T>>;

        fn retain_mut_with_others<T, F>(v: &mut Vec<T>, mut pred: F)
        where
            F: FnMut(&mut T, Others<'_, T>) -> bool,
        {
            let mut j = 0;
            for i in 0..v.len() {
                let (left, mid, right) = spliti_mut(v, i);
                let the_rest: Others<'_, T> = left.iter().chain(right.iter());
                let retain = pred(mid, the_rest);
                if retain {
                    v.swap(i, j);
                    j += 1;
                }
            }
            v.truncate(j);
        }
        pub fn spliti_mut<T>(v: &mut Vec<T>, i: usize) -> (&mut [T], &mut T, &mut [T]) {
            assert!(i < v.len());
            unsafe { spliti_mut_unchecked(v, i) }
        }
        pub unsafe fn spliti_mut_unchecked<T>(
            v: &mut Vec<T>,
            i: usize,
        ) -> (&mut [T], &mut T, &mut [T]) {
            unsafe {
                let len = v.len();
                let ptr = v.as_mut_ptr();
                let left = slice::from_raw_parts_mut(ptr, i);
                let mid = &mut *ptr.add(i);
                let right = slice::from_raw_parts_mut(ptr.add(i + 1), len - i - 1);
                (left, mid, right)
            }
        }
        retain_mut_with_others(&mut self.pieces, |piece, mut others| {
            let priority = get_priority(piece);
            let new_position = position_after_step(&piece.state);
            // check if any intersect
            let intersects = others.any(|capturer| {
                piece.side != capturer.side
                    && can_be_captured(priority, new_position, capturer)
                    && piece_will_be_captured(piece, capturer)
            });
            !intersects
        });
        self.advance_pieces_by_time(1);
        debug_assert!(!self.has_overlapping_pieces());
    }
    // Does not check intersections between pieces
    fn advance_pieces_by_time(&mut self, time: u32) {
        for piece in &mut self.pieces {
            match &mut piece.state {
                PieceState::Stationary { cooldown, .. } => {
                    // decrement cooldown
                    *cooldown = cooldown.saturating_sub(time);
                }
                PieceState::Moving {
                    x,
                    y,
                    target:
                        MoveTarget {
                            target,
                            turns_left,
                            priority,
                            velocity: (vx, vy),
                        },
                } => {
                    if *turns_left <= time {
                        // check if it's a pawn promotion
                        let is_pawn_promotion = piece.kind == PieceKind::Pawn
                            && target.y
                                == match piece.side {
                                    Side::White => 0u32,
                                    Side::Black => BOARD_SIZE as u32 - 1u32,
                                };
                        if is_pawn_promotion {
                            piece.kind = PieceKind::Queen;
                        }
                        piece.state = PieceState::Stationary {
                            position: *target,
                            cooldown: PIECE_COOLDOWN.saturating_sub(time - *turns_left),
                        }
                    } else {
                        *x += (time as f32) * *vx;
                        *y += (time as f32) * *vy;
                        *turns_left -= time;
                        *priority += time;
                    }
                }
            };
        }
    }
    pub fn get_all_possible_moves_naive(&self, side: Side) -> Vec<BoardMove> {
        let mut moves: Vec<BoardMove> = Vec::new();
        // Handle castling
        if self.can_long_castle[side] {
            // TODO: Check cooldown
            moves.push(BoardMove::LongCastle(side));
        }
        if self.can_short_castle[side] {
            // TODO: Check cooldown
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
        moves.push(BoardMove::None(side));
        moves
    }
    pub fn get_sorted_quiescent_moves<F>(&self, side: Side, f: F) -> Vec<BoardMove>
    where
        F: Fn(PieceKind) -> i32,
    {
        unsafe {
            Q_COUNT += 1;
        }
        // TODO: search promotion moves?

        // prioritize captures by computing value of victim - attacker
        // then prioritize getting out of the way - maybe by distance? shorter distance is better?
        // then search None?

        // only if capturing move AND they can't get away
        // OR if this piece is a target
        let mut capture_moves: Vec<(BoardMove, i32)> = Vec::new();
        let mut out_of_capture_moves: Vec<BoardMove> = Vec::new();
        for piece in &self.pieces {
            // TODO-someday: should be looping through stationary pieces and filter by side
            if piece.side == side {
                if let PieceState::Stationary { position, .. } = piece.state {
                    let pawn_is_about_to_be_promoted = piece.kind == PieceKind::Pawn
                        && position.y
                            == match piece.side {
                                Side::White => 1u32,
                                Side::Black => (BOARD_SIZE as u32) - 2u32,
                            };
                    if pawn_is_about_to_be_promoted || self.is_target_of_capture(&position) {
                        self.add_possible_moves_for_piece(piece, &mut out_of_capture_moves);
                    } else {
                        // attempt to capture other pieces
                        for to_be_captured in &self.pieces {
                            let potential_move = self.get_force_capture_move(to_be_captured, piece);
                            if let Some(board_move) = potential_move {
                                let priority = f(piece.kind) - f(to_be_captured.kind); // capturer - target
                                capture_moves.push((board_move, priority));
                            }
                        }
                    }
                }
            }
        }

        let mut all_moves = capture_moves
            .into_iter()
            .sorted_by_key(|(_board_move, priority)| *priority)
            .map(|(board_move, _priority)| board_move)
            .collect_vec();
        all_moves.append(&mut out_of_capture_moves);
        all_moves.push(BoardMove::None(side));
        debug_assert!(all_moves.iter().all(|board_move| {
            match board_move {
                BoardMove::Normal { target, .. } => self.is_valid_destination(side, *target),
                _ => true,
            }
        }));
        all_moves
    }
    // TODO-someday: should be 2 stationary pieces
    fn get_force_capture_move(&self, piece: &Piece, capturer: &Piece) -> Option<BoardMove> {
        if piece.side == capturer.side {
            return None;
        }
        if let PieceState::Stationary { position, cooldown } = piece.state && let PieceState::Stationary { position: capturer_position, cooldown: capturer_cooldown } = capturer.state && capturer_cooldown == 0 {

            let delta = position - capturer_position;
            let transit_time = delta.dist_linf();
            if cooldown >= transit_time {
                let board_move = BoardMove::Normal { piece: *capturer, target: position };
                if self.can_move(&board_move) {
                    Some(board_move)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    // TODO: Optimizable across multiple calls
    fn is_target_of_capture(&self, position: &Position) -> bool {
        self.pieces.iter().any(|piece| {
            if let PieceState::Moving {
                target: MoveTarget { target, .. },
                ..
            } = piece.state
            {
                &target == position
            } else {
                false
            }
        })
    }
    // TODO-someday: return nonempty list?
    pub fn get_all_possible_moves(&self, side: Side) -> Vec<BoardMove> {
        let mut moves: Vec<BoardMove> = Vec::new();
        for piece in self.pieces.iter() {
            if piece.side == side {
                // TODO-someday: should be looping through stationary pieces and filter by side
                self.add_possible_moves_for_piece(piece, &mut moves);
            }
        }
        moves.push(BoardMove::None(side));
        debug_assert!(moves.iter().all(|board_move| {
            match board_move {
                BoardMove::Normal { target, .. } => self.is_valid_destination(side, *target),
                _ => true,
            }
        }));
        moves
    }
    pub fn add_possible_moves_for_piece(&self, piece: &Piece, moves: &mut Vec<BoardMove>) {
        if let PieceState::Stationary { position, cooldown } = piece.state && cooldown == 0 {
            let side = piece.side;
            let mut generator = MoveGenerator { piece, position, side, moves, board: self };
            match piece.kind {
                PieceKind::Pawn => {
                    let forward_y = forward_y(side);
                    let double_move_dest = position + (0, forward_y * 2);
                    let can_double_move = position.y == match side {
                        Side::White => BOARD_SIZE as u32 - 2,
                        Side::Black => 1,
                    } && generator.is_valid_force_no_capture_destination(double_move_dest);
                    if can_double_move {
                        generator.add_board_move(double_move_dest);
                    }
                    let regular_move_dest = position + (0, forward_y);
                    let can_regular_move = generator.is_valid_force_no_capture_destination(regular_move_dest);
                    if can_regular_move {
                        generator.add_board_move(regular_move_dest);
                    }
                    if position.x > 0 {
                        let neg_x_capture_dest = position + (-1, forward_y);
                        let can_neg_x_capture = generator.is_valid_force_capture_destination(neg_x_capture_dest);
                        if can_neg_x_capture {
                            generator.add_board_move(neg_x_capture_dest);
                        }
                    }
                    if position.x < (BOARD_SIZE - 1) as u32 {
                        let pos_x_capture_dest = position + (1, forward_y);
                        let can_pos_x_capture = generator.is_valid_force_capture_destination(pos_x_capture_dest);
                        if can_pos_x_capture {
                            generator.add_board_move(pos_x_capture_dest);
                        }
                    }
                },
                PieceKind::Knight => {
                    let dx = [-2, -2, 2, 2, -1, -1, 1, 1];
                    let dy = [-1, 1, -1, 1, -2, 2, -2, 2];
                    let knights_delta: [Delta; 8] = dx.zip(dy).map(|x| x.into());
                    generator.add_moves_by_deltas(knights_delta);
                },
                PieceKind::Bishop => {
                    let dx = [-1, -1, 1, 1];
                    let dy = [-1, 1, -1, 1];
                    let bishops_delta: [Delta; 4] = dx.zip(dy).map(|x| x.into());
                    generator.add_moves_by_continuous_deltas(bishops_delta);
                },
                PieceKind::Rook => {
                    let dx = [-1, 1, 0, 0];
                    let dy = [0, 0, -1, 1];
                    let rooks_delta: [Delta; 4] = dx.zip(dy).map(|x| x.into());
                    generator.add_moves_by_continuous_deltas(rooks_delta);
                },
                PieceKind::Queen => {
                    let dx = [-1, -1, 1, 1, -1, 1, 0, 0];
                    let dy = [-1, 1, -1, 1, 0, 0, -1, 1];
                    let queens_delta: [Delta; 8] = dx.zip(dy).map(|x| x.into());
                    generator.add_moves_by_continuous_deltas(queens_delta);
                },
                PieceKind::King => {
                    let dx = [-1, -1, -1, 0, 0, 1, 1, 1];
                    let dy = [-1, 0, 1, -1, 1, -1, 0, 1];
                    let kings_delta: [Delta; 8] = dx.zip(dy).map(|x| x.into());
                    generator.add_moves_by_deltas(kings_delta);
                },
            }
        }
    }
    pub fn to_stationary_map<F>(&self, default_char: char, f: F) -> String
    where
        F: Fn(&Piece) -> char,
    {
        to_char_map(|position| {
            let piece = self.get_stationary_piece(position);
            piece.map_or(default_char, &f)
        })
    }
    pub fn to_stationary_map_cooldowns(&self) -> String {
        self.to_stationary_map('.', |piece| {
            if let PieceState::Stationary { cooldown, .. } = piece.state {
                if cooldown == 10 {
                    'X'
                } else {
                    cooldown.to_string().chars().next().unwrap()
                }
            } else {
                '?'
            }
        })
    }
    pub fn to_stationary_map_combo(&self) -> String {
        self.to_stationary_map('.', |piece| {
            let c: char = piece.kind.into();
            match piece.side {
                Side::White => c.to_ascii_uppercase(),
                Side::Black => c.to_ascii_lowercase(),
            }
        })
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
    pub fn parse_fen(fen: &str) -> OrError<Self> {
        let mut pieces = Vec::new();
        for (row, line) in fen.split('/').enumerate() {
            let mut cursor = 0;
            for c in line.chars() {
                if c.is_ascii_digit() {
                    cursor += c.to_digit(10).unwrap();
                } else {
                    if let Some(kind) = PieceKind::from_char(c.to_ascii_uppercase()) {
                        let side = match c.is_ascii_uppercase() {
                            true => Side::White,
                            false => Side::Black,
                        };
                        let position = (cursor, row as u32).into();
                        pieces.push(Piece {
                            side,
                            kind,
                            state: PieceState::Stationary {
                                position,
                                cooldown: 0u32,
                            },
                        })
                    } else {
                        return Err(Error!("Unable to parse piece kind {} in {}", c, line));
                    }
                    cursor += 1;
                }
            }
        }
        Ok(Self::new_with_castling(pieces, false))
    }
    pub fn is_all_pieces_stationary(&self) -> bool {
        self.pieces
            .iter()
            .all(|piece| matches!(piece.state, PieceState::Stationary { .. }))
    }
    pub fn is_all_pieces_stationary_with_no_cooldown(&self) -> bool {
        self.pieces.iter().all(
            |piece| matches!(piece.state, PieceState::Stationary { cooldown, .. } if cooldown == 0),
        )
    }
    fn has_overlapping_pieces(&self) -> bool {
        self.pieces.iter().tuple_combinations().any(|(a, b)| {
            if let PieceState::Stationary { position: a_position, .. } = a.state &&
                let PieceState::Stationary { position: b_position, .. } = b.state {
                a_position == b_position
            } else {
                false
            }
        })
    }
    // whether destination is a valid destination for a piece of a side
    fn is_valid_destination(&self, side: Side, destination: Position) -> bool {
        // ensures the destination is not some other piece's target
        // or has an existing stationary piece
        !self.pieces.iter().any(|piece| {
            if side != piece.side {
                return false;
            }
            match piece.state {
                PieceState::Stationary {
                    position: piece_position,
                    ..
                } => destination == piece_position,
                PieceState::Moving {
                    target: MoveTarget { target, .. },
                    ..
                } => destination == target,
            }
        })
    }
    pub fn generate_random_board_with(num_pieces_per_side: usize) -> Self {
        let distribution = "PPPPPPPPNNBBRRQ"
            .chars()
            .map(|c| PieceKind::from_char(c).unwrap())
            .collect_vec();
        debug_assert!(num_pieces_per_side >= 1); // requires king
        debug_assert!(num_pieces_per_side <= distribution.len());

        let white_pieces = distribution
            .choose_multiple(&mut rand::thread_rng(), num_pieces_per_side - 1)
            .chain(std::iter::once(&PieceKind::King))
            .map(|kind| (Side::White, kind.clone()));
        let black_pieces = distribution
            .choose_multiple(&mut rand::thread_rng(), num_pieces_per_side - 1)
            .chain(std::iter::once(&PieceKind::King))
            .map(|kind| (Side::Black, kind.clone()));
        let pieces = white_pieces.chain(black_pieces).collect_vec();
        Self::generate_random_board(pieces)
    }
    // TODO-someday: maybe put some rules that will generate reasonable boards
    pub fn generate_random_board(pieces: impl IntoIterator<Item = (Side, PieceKind)>) -> Self {
        fn random_position() -> Position {
            let x = rand::thread_rng().gen_range(0..BOARD_SIZE) as u32;
            let y = rand::thread_rng().gen_range(0..BOARD_SIZE) as u32;
            Position { x, y }
        }
        fn random_with_reqs<F>(f: F) -> Position
        where
            F: Fn(&Position) -> bool,
        {
            loop {
                let position = random_position();
                if f(&position) {
                    return position;
                }
            }
        }
        let mut pieces_vec = Vec::new();
        let mut occupied = [[false; BOARD_SIZE]; BOARD_SIZE];

        fn add_piece(
            pieces: &mut Vec<Piece>,
            occupied: &mut [[bool; BOARD_SIZE]; BOARD_SIZE],
            side: Side,
            kind: PieceKind,
        ) {
            let position = match kind {
                PieceKind::Pawn => random_with_reqs(|pos| {
                    pos.y != 0
                        && pos.y != BOARD_SIZE as u32 - 1
                        && !occupied[pos.x as usize][pos.y as usize]
                }),
                _ => random_with_reqs(|pos| !occupied[pos.x as usize][pos.y as usize]),
            };
            occupied[position.x as usize][position.y as usize] = true;
            pieces.push(Piece {
                side,
                kind,
                state: PieceState::Stationary {
                    position,
                    cooldown: 0,
                },
            });
        }
        for (side, kind) in pieces {
            add_piece(&mut pieces_vec, &mut occupied, side, kind);
        }
        Self::new_with_castling(pieces_vec, false)
    }
}

#[derive(Debug, Clone)]
pub enum BoardMove {
    None(Side),
    LongCastle(Side),
    ShortCastle(Side),
    Normal { piece: Piece, target: Position }, // TODO-someday: PieceState should always be stationary here - represent this differently?
}

impl BoardMove {
    pub fn side(&self) -> Side {
        match self {
            BoardMove::None(side) => *side,
            BoardMove::LongCastle(side) => *side,
            BoardMove::ShortCastle(side) => *side,
            BoardMove::Normal { piece, .. } => piece.side,
        }
    }
}

struct MoveGenerator<'a> {
    piece: &'a Piece,
    position: Position,
    side: Side,
    moves: &'a mut Vec<BoardMove>,
    board: &'a BoardState,
}

impl MoveGenerator<'_> {
    fn try_add(position: Position, Delta { x: dx, y: dy }: Delta) -> Option<Position> {
        let x = (position.x as i32) + dx;
        let y = (position.y as i32) + dy;
        if x >= 0 && y >= 0 && x < BOARD_SIZE as i32 && y < BOARD_SIZE as i32 {
            Some(Position {
                x: x as u32,
                y: y as u32,
            })
        } else {
            None
        }
    }
    fn is_valid_destination(&self, destination: Position) -> bool {
        self.board.is_valid_destination(self.side, destination)
    }
    fn is_valid_force_no_capture_destination(&self, destination: Position) -> bool {
        // used by pawns only
        self.is_valid_destination(destination)
            && self.board.get_stationary_piece(destination).is_none()
    }
    fn is_valid_force_capture_destination(&self, destination: Position) -> bool {
        // used by pawns only
        self.is_valid_destination(destination)
            && self.board.get_stationary_piece(destination).is_some()
    }
    fn add_board_move(&mut self, target: Position) {
        self.moves.push(BoardMove::Normal {
            piece: *self.piece,
            target,
        });
    }
    // knights, kings
    fn add_moves_by_deltas<const N: usize>(&mut self, deltas: [Delta; N]) {
        for delta in deltas {
            let position = Self::try_add(self.position, delta);
            if let Some(position) = position && self.is_valid_destination(position) {
                self.add_board_move(position);
            }
        }
    }
    // bishops, rooks, queens
    fn add_moves_by_continuous_deltas<const N: usize>(&mut self, deltas: [Delta; N]) {
        for delta in deltas {
            let mut current = self.position;
            while let Some(next) = Self::try_add(current, delta) {
                current = next;
                if self.is_valid_destination(current) {
                    self.add_board_move(current);
                    // break when it's an enemy
                    let has_enemy = self
                        .board
                        .get_stationary_piece(current)
                        .map_or(false, |piece| piece.side != self.side);
                    if has_enemy {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }
}
