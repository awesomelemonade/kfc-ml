use super::*;
use enum_map::{enum_map, EnumMap};

core!();

lazy_static! {
    pub static ref MATERIAL_VALUE: EnumMap<PieceKind, u32> = enum_map! {
        PieceKind::Pawn => 1,
        PieceKind::Knight => 3,
        PieceKind::Bishop => 3,
        PieceKind::Rook => 5,
        PieceKind::Queen => 9,
        PieceKind::King => 100,
    };
}

const MAX_DEPTH: u32 = 2;

type HeuristicScore = f32;

pub static mut NUM_LEAVES: usize = 0;

pub fn evaluate_material_heuristic(state: &BoardState) -> HeuristicScore {
    unsafe {
        NUM_LEAVES += 1;
    }
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

pub fn white_move(
    state: &BoardState,
    depth: u32,
    mut alpha: HeuristicScore,
    beta: HeuristicScore,
) -> (Option<BoardMove>, HeuristicScore) {
    if depth >= MAX_DEPTH {
        return (None, evaluate_material_heuristic(state));
    }
    let mut best_move = Option::None;
    let (mut _best_opponent_move, mut best_score) = black_move(state, depth, alpha, beta, None);
    alpha = alpha.max(best_score);
    if best_score < beta {
        let possible_moves = state.get_all_possible_moves(Side::White);
        // TODO: reorder possible_moves
        for board_move in possible_moves {
            let (opponent_move, score) = black_move(state, depth, alpha, beta, Some(&board_move));
            if score > best_score {
                best_score = score;
                best_move = Some(board_move);
                _best_opponent_move = opponent_move;
            }
            alpha = alpha.max(best_score);
            if best_score >= beta {
                break;
            }
        }
    }
    (best_move, best_score)
}

pub fn black_move(
    state: &BoardState,
    depth: u32,
    alpha: HeuristicScore,
    mut beta: HeuristicScore,
    pending_white_move: Option<&BoardMove>,
) -> (Option<BoardMove>, f32) {
    if depth >= MAX_DEPTH {
        return (None, evaluate_material_heuristic(state));
    }
    let possible_moves = state.get_all_possible_moves(Side::Black);
    let mut best_move = Option::None;
    let mut new_state_no_move = state.clone();
    if let Some(pending_white_move) = pending_white_move {
        new_state_no_move.apply_move(pending_white_move);
    }
    new_state_no_move.step();
    let (mut _best_opponent_move, mut best_score) =
        white_move(&new_state_no_move, depth + 1, alpha, beta);
    beta = beta.min(best_score);
    if best_score > alpha {
        for board_move in possible_moves {
            let mut new_state = state.clone();
            if let Some(pending_white_move) = pending_white_move {
                new_state.apply_move(pending_white_move);
            }
            new_state.apply_move(&board_move);
            new_state.step();
            let (opponent_move, score) = white_move(&new_state, depth + 1, alpha, beta);
            if score < best_score {
                best_score = score;
                best_move = Some(board_move);
                _best_opponent_move = opponent_move;
            }
            beta = beta.min(best_score);
            if best_score <= alpha {
                break;
            }
        }
    }
    (best_move, best_score)
}
