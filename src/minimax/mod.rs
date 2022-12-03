core!();

use std::cmp::Ordering;

use crate::*;
use enum_map::{enum_map, EnumMap};

#[cfg(test)]
mod minimax_tests;

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

const MAX_QUIESCENT_DEPTH: u32 = 2; // TODO (was 5)

type HeuristicScore = f32;

pub fn evaluate_material_heuristic(state: &BoardState) -> HeuristicScore {
    if let Some(end_state) = get_board_end_state(state) {
        return match end_state {
            EndState::Winner(side) => match side {
                Side::White => 100f32,
                Side::Black => -100f32,
            },
            EndState::Draw => 0f32,
        };
    }
    let mut state = state.clone();
    state.step_until_stationary_with_no_cooldown();
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

pub fn get_board_end_state(state: &BoardState) -> Option<EndState> {
    // if there are only 2 kings (and both with 0 cooldown), it's a draw
    // if one side is missing a king, the other is the winner
    // if both sides are missing a king, it's a draw (i.e. both somehow captured at the same time)

    // we could also do other heuristics, but the model would not know how to play them
    // if we put it as an end state (ex: K + R vs K is almost always a win)

    let white_king = state
        .pieces()
        .iter()
        .find(|p| p.kind == PieceKind::King && p.side == Side::White);
    let black_king = state
        .pieces()
        .iter()
        .find(|p| p.kind == PieceKind::King && p.side == Side::Black);
    match (white_king, black_king) {
        (None, None) => Some(EndState::Draw),
        (None, Some(_)) => Some(EndState::Winner(Side::Black)),
        (Some(_), None) => Some(EndState::Winner(Side::White)),
        (Some(white_king), Some(black_king)) => {
            if state.pieces().len() == 2 {
                if let PieceState::Stationary { position: white_position, cooldown: white_cooldown } = white_king.state &&
                    let PieceState::Stationary { position: black_position, cooldown: black_cooldown } = black_king.state {
                    let distance = (white_position - black_position).dist_linf();
                    if distance <= 1 {
                        match white_cooldown.cmp(&black_cooldown) {
                            Ordering::Less => Some(EndState::Winner(Side::White)),
                            Ordering::Equal => Some(EndState::Draw),
                            Ordering::Greater => Some(EndState::Winner(Side::White)),
                        }
                    } else {
                        Some(EndState::Draw)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

pub enum EndState {
    Winner(Side),
    Draw,
}

#[derive(Debug)]
pub struct MinimaxOutputInfo {
    pub board: BoardState,
    pub search_depth: u32,
    pub score: HeuristicScore,
    pub num_leaves: u32,
    pub num_regular_nodes: u32,
    pub num_quiescent_nodes: u32,
    pub moves: Vec<BoardMove>,
}

impl MinimaxOutputInfo {
    // TODO-someday: assumes white first when it probably shouldn't
    pub fn iter_states<F>(&self, mut f: F)
    where
        F: FnMut(&BoardState),
    {
        let mut board = self.board.clone();
        f(&board);
        self.moves
            .chunks(2)
            .take(self.search_depth as usize)
            .for_each(|chunk| match chunk {
                [white_move, black_move] => {
                    board.step(white_move, black_move);
                    f(&board);
                }
                _ => {
                    panic!();
                }
            });
    }
    pub fn to_representations(&self) -> Vec<BoardRepresentation> {
        let mut representations = Vec::new();
        self.iter_states(|state| {
            representations.push(state.into());
        });
        representations
    }
    fn try_from(
        output: &MinimaxOutput,
        board: BoardState,
        search_depth: u32,
    ) -> OrError<MinimaxOutputInfo> {
        let score = output.score();
        let mut moves = Vec::new();

        let mut current = output;
        while let MinimaxOutput::Node {
            best_move,
            best_score,
            next,
            ..
        } = current
        {
            if *best_score != score {
                return Err(Error!("Scores do not match"));
            }
            moves.push(best_move.clone());
            current = next;
        }
        Ok(MinimaxOutputInfo {
            score,
            num_leaves: output.num_leaves(),
            moves,
            board,
            search_depth,
            num_regular_nodes: output.num_regular_nodes(search_depth as i32),
            num_quiescent_nodes: output.num_quiescent_nodes(search_depth as i32),
        })
    }
}

#[derive(Debug, Clone)]
pub enum MinimaxOutput {
    Node {
        best_move: BoardMove,
        best_score: HeuristicScore,
        num_leaves: u32,
        next: Box<MinimaxOutput>,
        num_regular_nodes: u32,
        num_quiescent_nodes: u32,
    },
    Leaf {
        score: HeuristicScore,
    },
}

impl MinimaxOutput {
    pub fn score(&self) -> HeuristicScore {
        match self {
            MinimaxOutput::Node { best_score, .. } => *best_score,
            MinimaxOutput::Leaf { score } => *score,
        }
    }

    pub fn num_leaves(&self) -> u32 {
        match self {
            MinimaxOutput::Node { num_leaves, .. } => *num_leaves,
            MinimaxOutput::Leaf { .. } => 1,
        }
    }

    pub fn num_regular_nodes(&self, depth: i32) -> u32 {
        (match self {
            MinimaxOutput::Node {
                num_regular_nodes, ..
            } => *num_regular_nodes,
            MinimaxOutput::Leaf { .. } => 0,
        }) + (if depth < 0 { 0u32 } else { 1u32 })
    }

    pub fn num_quiescent_nodes(&self, depth: i32) -> u32 {
        (match self {
            MinimaxOutput::Node {
                num_quiescent_nodes,
                ..
            } => *num_quiescent_nodes,
            MinimaxOutput::Leaf { .. } => 0,
        }) + (if depth < 0 { 1u32 } else { 0u32 })
    }
}

pub fn search_white(board: &BoardState, depth: u32) -> OrError<MinimaxOutputInfo> {
    let output = white_move(
        board,
        depth as i32,
        f32::NEG_INFINITY,
        f32::INFINITY,
        &|_board, _board_move| 0f32, // TODO
        &evaluate_material_heuristic,
    );
    MinimaxOutputInfo::try_from(&output, board.clone(), depth)
}

fn white_move<F, G>(
    state: &BoardState,
    depth: i32,
    mut alpha: HeuristicScore,
    beta: HeuristicScore,
    move_heuristic: &F,
    leaf_heuristic: &G,
) -> MinimaxOutput
where
    F: Fn(&BoardState, &BoardMove) -> HeuristicScore,
    G: Fn(&BoardState) -> HeuristicScore,
{
    if get_board_end_state(state).is_some()
        || (state.is_all_pieces_stationary_with_no_cooldown() && depth <= 0)
        || depth <= -(MAX_QUIESCENT_DEPTH as i32)
    {
        let score = leaf_heuristic(state);
        return MinimaxOutput::Leaf { score };
    }
    let mut best_move = BoardMove::None(Side::White);
    let mut best_opponent_move = None;
    let mut best_score = f32::MIN;
    let mut num_leaves = 0;
    let mut num_regular_nodes = 0;
    let mut num_quiescent_nodes = 0;
    let possible_moves = if depth <= 0 {
        state.get_sorted_quiescent_moves(Side::White, |kind| MATERIAL_VALUE[kind] as i32)
    } else {
        let mut possible_moves = state.get_all_possible_moves(Side::White);
        // greater heuristic score = better for white, so we sort descending
        util::sort_by_cached_f32_exn(&mut possible_moves, |possible_move| {
            -move_heuristic(state, possible_move)
        });
        possible_moves
    };
    for board_move in possible_moves {
        let opponent_move = black_move(
            state,
            depth,
            alpha,
            beta,
            &board_move,
            move_heuristic,
            leaf_heuristic,
        );
        num_leaves += opponent_move.num_leaves();
        num_regular_nodes += opponent_move.num_regular_nodes(depth);
        num_quiescent_nodes += opponent_move.num_quiescent_nodes(depth);
        let score = opponent_move.score();
        if score > best_score {
            best_move = board_move;
            best_opponent_move = Some(opponent_move);
            best_score = score;
        }
        alpha = alpha.max(best_score);
        if best_score >= beta {
            break;
        }
    }
    MinimaxOutput::Node {
        best_move,
        best_score,
        num_leaves,
        next: Box::new(best_opponent_move.unwrap()),
        num_regular_nodes,
        num_quiescent_nodes,
    }
}

fn black_move<F, G>(
    state: &BoardState,
    depth: i32,
    alpha: HeuristicScore,
    mut beta: HeuristicScore,
    pending_white_move: &BoardMove,
    move_heuristic: &F,
    leaf_heuristic: &G,
) -> MinimaxOutput
where
    F: Fn(&BoardState, &BoardMove) -> HeuristicScore,
    G: Fn(&BoardState) -> HeuristicScore,
{
    if get_board_end_state(state).is_some()
        || (state.is_all_pieces_stationary_with_no_cooldown() && depth <= 0)
        || depth <= -(MAX_QUIESCENT_DEPTH as i32)
    {
        let score = evaluate_material_heuristic(state);
        return MinimaxOutput::Leaf { score };
    }
    let mut best_move = BoardMove::None(Side::Black);
    let mut best_opponent_move = None;
    let mut best_score = f32::MAX;
    let mut num_leaves = 0;
    let mut num_regular_nodes = 0;
    let mut num_quiescent_nodes = 0;
    let possible_moves = if depth <= 0 {
        state.get_sorted_quiescent_moves(Side::Black, |kind| MATERIAL_VALUE[kind] as i32)
    } else {
        let mut possible_moves = state.get_all_possible_moves(Side::Black);
        // greater heuristic score = better for white, so we sort ascending
        util::sort_by_cached_f32_exn(&mut possible_moves, |possible_move| {
            move_heuristic(state, possible_move)
        });
        possible_moves
    };
    for board_move in possible_moves {
        let mut new_state = state.clone();
        if depth <= 0 {
            new_state.apply_move(pending_white_move);
            new_state.apply_move(&board_move);
            // TODO-someday: may need to adjust
            if !new_state.step_until_one_becomes_stationary() {
                new_state.step_until_stationary_with_no_cooldown();
            }
        } else {
            new_state.step(pending_white_move, &board_move);
        }
        let opponent_move = white_move(
            &new_state,
            depth - 1,
            alpha,
            beta,
            move_heuristic,
            leaf_heuristic,
        );
        num_leaves += opponent_move.num_leaves();
        num_regular_nodes += opponent_move.num_regular_nodes(depth);
        num_quiescent_nodes += opponent_move.num_quiescent_nodes(depth);
        let score = opponent_move.score();
        if score < best_score {
            best_score = score;
            best_move = board_move;
            best_opponent_move = Some(opponent_move);
        }
        beta = beta.min(best_score);
        if best_score <= alpha {
            break;
        }
    }
    MinimaxOutput::Node {
        best_move,
        best_score,
        num_leaves,
        next: Box::new(best_opponent_move.unwrap()),
        num_regular_nodes,
        num_quiescent_nodes,
    }
}
