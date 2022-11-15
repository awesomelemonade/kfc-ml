#![feature(const_for)]
#![feature(let_chains)]
#![feature(array_zip)]

macro_rules! core {
    () => {
        #[allow(unused_imports)]
        use crate::imports::*;
    };
}

core!();

mod imports;

mod game;
pub use game::*;

mod minimax;
use itertools::Itertools;
use minimax::white_move;
use minimax::MinimaxOutput;

// use pyo3::{
//     prelude::*,
//     types::{IntoPyDict, PyModule},
// };
use rand::seq::SliceRandom;

// board.calc_valid_moves_for_piece(PIECE)
// board.calc_valid_moves()
// board.calc_valid_next_states() # next states for combination moves
// board.calc_valid_next_states_one_move() # next states for one move
// board.step(moves) // Also resolves collisions

// As input to an ML model - impl from board?
// struct BoardRepresentation {
//     // for each side
//     // pawn -> 8 slots
//     // rook -> 2 slots
//     // knight -> 2 slots
//     // bishop -> 2 slots
//     // queen -> 2 slots // extra slot for queens for promotion
//     // king -> 1 slot
// }

const SEARCH_DEPTH: u32 = 2;

fn get_diff(board: &BoardState) -> f32 {
    let all_moves = board.get_all_possible_moves(Side::White);
    let random_move = all_moves.choose(&mut rand::thread_rng());
    unsafe {
        minimax::NUM_LEAVES = 0;
    }
    let minimax_output = white_move(board, SEARCH_DEPTH, f32::NEG_INFINITY, f32::INFINITY);
    let minimax_move = match minimax_output {
        MinimaxOutput::Node { best_move, .. } => best_move,
        MinimaxOutput::Leaf { .. } => None,
    };
    unsafe {
        println!("NUM_LEAVES={}", minimax::NUM_LEAVES);
    }
    let random_score = get_score(board, random_move);
    let minimax_score = get_score(board, minimax_move.as_ref());
    minimax_score - random_score
}

fn get_score(board: &BoardState, white_move: Option<&BoardMove>) -> f32 {
    let mut board_mut = board.clone();
    let black_moves = board_mut.get_all_possible_moves(Side::Black);
    let random_black_move = black_moves.choose(&mut rand::thread_rng());
    if let Some(white_move) = white_move {
        board_mut.apply_move(white_move);
    }
    if let Some(black_move) = random_black_move {
        board_mut.apply_move(black_move)
    }
    // step until stationary
    while !board_mut.is_all_pieces_stationary() {
        board_mut.step();
    }
    minimax::evaluate_material_heuristic(&board_mut)
}

fn main() {
    let fen_strings = r#"2qn3B/P2k3p/5b1Q/8/8/Pb1r3P/1p5P/4K2R
6n1/4P1B1/1Npkp1Pp/1Q1q4/8/2r2P2/4RK2/4n3
r7/4B3/4P1p1/2P1q3/1p3n2/1bPn2PP/6N1/1k3K2
8/1P6/5bK1/PP4Q1/3PN2B/1k4pN/4p1B1/1n2R3
q6R/4n3/5K2/4PRB1/1p3Q2/1pP4p/2k1p1p1/7r
5N2/1Pp3b1/8/3B2R1/1B1k2KQ/6p1/4p1P1/r4n1r
1r6/bk6/5P2/2R1pPPp/1K5P/pp2n2b/1P6/8
8/2k2K2/p1P4P/1PN2p1P/1p1B4/R3p3/4P3/2r3b1
7B/3r4/1p4BP/1b5P/pk1r1p2/pn5P/2p5/6K1
1K6/3Bn2k/pR1P4/b2pP3/7p/qP6/5r1B/6R1
Q2b2N1/1qp5/2R3n1/4P2k/K3P3/2P5/1P1P3p/7N
8/4R1p1/3P1Pb1/Npp5/3P4/5Bk1/1R1pK3/r3N3"#;
    let board_states = fen_strings
        .split('\n')
        .map(|fen| BoardState::parse_fen(fen).unwrap())
        .collect_vec();

    let scores = board_states.iter().map(get_diff).collect_vec();
    println!("scores={:?}", scores);
    let average = scores.iter().sum::<f32>() / scores.len() as f32;
    println!("avg={:?}", average);

    // let board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();

    // let score = white_move(&board, 0);
    // println!("score={:?}", score);

    //     let result = Python::with_gil(|py| {
    //         let activators = PyModule::from_code(
    //             py,
    //             r#"
    // import numpy as np

    // def relu(x):
    //     return max(0.0, x)
    // def leaky_relu(x, slope=0.01):
    //     return x if x >= 0 else x * slope
    //         "#,
    //             "activators.py",
    //             "activators",
    //         )?;

    //         let relu_result: f64 = activators.getattr("relu")?.call1((-1.0,))?.extract()?;
    //         assert_eq!(relu_result, 0.0);

    //         let kwargs = [("slope", 0.2)].into_py_dict(py);
    //         let lrelu_result: f64 = activators
    //             .getattr("leaky_relu")?
    //             .call((-1.0,), Some(kwargs))?
    //             .extract()?;
    //         assert_eq!(lrelu_result, -0.2);
    //         PyResult::Ok(())
    //     });
    //     println!("Result: {:?}", result);
}
