use super::*;
use enum_map::{enum_map, EnumMap};

core!();

lazy_static! {
    pub static ref MATERIAL_VALUE: EnumMap<PieceKind, u32> = enum_map! {
        PieceKind::Pawn => 1,
        PieceKind::Knight => 3,
        PieceKind::Bishop => 3,
        PieceKind::Rook => 5,
        PieceKind::Queen => 8,
        PieceKind::King => 100,
    };
}

pub fn evaluate_material_heuristic(state: &BoardState) -> f32 {
    // count up material
    let material_value: i32 = state
        .pieces()
        .iter()
        .map(|piece| {
            let side = match piece.side {
                Side::White => 1i32,
                Side::Black => -1i32,
            };
            let value = MATERIAL_VALUE[piece.kind] as i32;
            side * value
        })
        .sum();
    material_value as f32
}

pub fn white_move(state: &BoardState, depth: u32) -> (Option<BoardMove>, f32) {
    if depth >= 2 {
        return (None, evaluate_material_heuristic(state));
    }
    if depth == 1 {
        coz::begin!("white_move_1");
    }
    if depth == 0 {
        coz::begin!("white_move_0");
    }
    let possible_moves = state.get_all_possible_moves(Side::White);
    // if depth <= 1 {
    //     println!("white moves: {}, {}", possible_moves.len(), depth);
    // }
    let mut best_move = Option::None;
    let (mut best_opponent_move, mut best_score) = black_move(state, depth, None);
    for board_move in possible_moves {
        let (opponent_move, score) = black_move(state, depth, Some(&board_move));
        if score > best_score {
            best_score = score;
            best_move = Some(board_move);
            best_opponent_move = opponent_move;
        }
    }
    let _x = best_opponent_move;
    if depth == 0 {
        coz::end!("white_move_0");
    }
    if depth == 1 {
        coz::end!("white_move_1");
    }
    (best_move, best_score)
}

pub fn black_move(
    state: &BoardState,
    depth: u32,
    pending_white_move: Option<&BoardMove>,
) -> (Option<BoardMove>, f32) {
    if depth >= 2 {
        return (None, evaluate_material_heuristic(state));
    }
    let possible_moves = state.get_all_possible_moves(Side::Black);
    let mut best_move = Option::None;
    let mut new_state_no_move = state.clone();
    if let Some(pending_white_move) = pending_white_move {
        new_state_no_move.apply_move(pending_white_move);
    }
    new_state_no_move.step();
    let (mut best_opponent_move, mut best_score) = white_move(&new_state_no_move, depth + 1);
    for board_move in possible_moves {
        let mut new_state = state.clone();
        if let Some(pending_white_move) = pending_white_move {
            new_state.apply_move(pending_white_move);
        }
        new_state.apply_move(&board_move);
        new_state.step();
        let (opponent_move, score) = white_move(&new_state, depth + 1);
        if score < best_score {
            best_score = score;
            best_move = Some(board_move);
            best_opponent_move = opponent_move;
        }
    }
    let _x = best_opponent_move;
    (best_move, best_score)
}
