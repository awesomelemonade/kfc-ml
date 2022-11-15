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

pub struct MinimaxOutput {
    pub best_move: Option<BoardMove>,
    pub best_score: HeuristicScore,
    pub next: Option<Box<MinimaxOutput>>,
}

pub fn white_move(
    state: &BoardState,
    depth: u32,
    mut alpha: HeuristicScore,
    beta: HeuristicScore,
) -> MinimaxOutput {
    if depth >= MAX_DEPTH {
        let score = evaluate_material_heuristic(state);
        return MinimaxOutput {
            best_move: None,
            best_score: score,
            next: None,
        };
    }
    let mut best_move = Option::None;
    let mut best_opponent_move = black_move(state, depth, alpha, beta, None);
    let mut best_score = best_opponent_move.best_score;
    alpha = alpha.max(best_score);
    if best_score < beta {
        let possible_moves = state.get_all_possible_moves(Side::White);
        // TODO: reorder possible_moves
        for board_move in possible_moves {
            let opponent_move = black_move(state, depth, alpha, beta, Some(&board_move));
            let score = opponent_move.best_score;
            if score > best_score {
                best_score = score;
                best_move = Some(board_move);
                best_opponent_move = opponent_move;
            }
            alpha = alpha.max(best_score);
            if best_score >= beta {
                break;
            }
        }
    }
    MinimaxOutput {
        best_move,
        best_score,
        next: Some(Box::new(best_opponent_move)),
    }
}

pub fn black_move(
    state: &BoardState,
    depth: u32,
    alpha: HeuristicScore,
    mut beta: HeuristicScore,
    pending_white_move: Option<&BoardMove>,
) -> MinimaxOutput {
    if depth >= MAX_DEPTH {
        let score = evaluate_material_heuristic(state);
        return MinimaxOutput {
            best_move: None,
            best_score: score,
            next: None,
        };
    }
    let mut best_move = Option::None;
    let mut new_state_no_move = state.clone();
    if let Some(pending_white_move) = pending_white_move {
        new_state_no_move.apply_move(pending_white_move);
    }
    new_state_no_move.step();
    let mut best_opponent_move = white_move(&new_state_no_move, depth + 1, alpha, beta);
    let mut best_score = best_opponent_move.best_score;
    beta = beta.min(best_score);
    if best_score > alpha {
        let possible_moves = state.get_all_possible_moves(Side::Black);
        // TODO: reorder possible_moves
        for board_move in possible_moves {
            let mut new_state = state.clone();
            if let Some(pending_white_move) = pending_white_move {
                new_state.apply_move(pending_white_move);
            }
            new_state.apply_move(&board_move);
            new_state.step();
            let opponent_move = white_move(&new_state, depth + 1, alpha, beta);
            let score = opponent_move.best_score;
            if score < best_score {
                best_score = score;
                best_move = Some(board_move);
                best_opponent_move = opponent_move;
            }
            beta = beta.min(best_score);
            if best_score <= alpha {
                break;
            }
        }
    }
    MinimaxOutput {
        best_move,
        best_score,
        next: Some(Box::new(best_opponent_move)),
    }
}
