#![feature(const_for)]
#![feature(let_chains)]
#![feature(array_zip)]
#![feature(variant_count)]

macro_rules! core {
    () => {
        #[allow(unused_imports)]
        use crate::imports::*;
    };
}

core!();

mod imports;

mod game;
use std::{fs::File, io::BufRead, io::BufReader, time::Instant};

pub use game::*;

mod minimax;
pub use minimax::*;

mod board_representation;
pub use board_representation::*;

pub mod util;

mod sequential;

use itertools::Itertools;

use numpy::{PyArray1, PyArray2};
use pyo3::{prelude::*, types::PyModule};
use rand::seq::SliceRandom;
use rayon::prelude::*;

use crate::{sequential::SequentialModel, util::UnwrapWithTraceback};

const SEARCH_DEPTH: u32 = 2;

fn get_diff(board: &BoardState) -> OrError<f32> {
    let all_moves = board.get_all_possible_moves(Side::White);
    let random_move = all_moves.choose(&mut rand::thread_rng());
    let minimax_output = search_white(board, SEARCH_DEPTH)?;
    let num_leaves = minimax_output.num_leaves;
    println!("NUM LEAVES: {}", num_leaves); // TODO: remove
    let minimax_move = minimax_output
        .moves
        .first()
        .unwrap_or(&BoardMove::None(Side::White));
    let random_score = get_average_score(50000, board, random_move.unwrap());
    let minimax_score = get_average_score(50000, board, minimax_move);
    Ok(minimax_score - random_score)
}

fn get_average_score(n: u32, board: &BoardState, white_move: &BoardMove) -> f32 {
    let mut total = 0f32;
    for _ in 0..n {
        total += get_score(board, white_move);
    }
    total / (n as f32)
}

fn get_score(board: &BoardState, white_move: &BoardMove) -> f32 {
    let mut board_mut = board.clone();
    let black_moves = board_mut.get_all_possible_moves(Side::Black);
    let random_black_move = black_moves.choose(&mut rand::thread_rng()).unwrap();
    board_mut.step(white_move, random_black_move);
    minimax::evaluate_material_heuristic(&board_mut)
}

fn run_analysis() {
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
    let floats = board_states
        .iter()
        .map(|state| {
            let representation: BoardRepresentation = state.into();
            representation.to_float_array()
        })
        .collect_vec();
    println!("{:?}", floats);

    let scores: OrError<Vec<_>> = board_states.iter().map(get_diff).collect();
    println!("scores={:?}", scores);
    let scores = scores.unwrap();
    let average = scores.iter().sum::<f32>() / scores.len() as f32;
    println!("avg={:?}", average);
}

fn main() -> OrError<()> {
    let code = include_str!("./model.py");
    let result: PyResult<_> = Python::with_gil(|py| {
        println!("Importing Python Code");
        let module = PyModule::from_code(py, code, "model", "model")?;
        println!("Creating Model");
        let model = module.getattr("Model")?;
        let model_instance = model.call0()?;

        println!("Attempting to learn");

        let file = File::open("processed_random.fen").unwrap();
        let reader = BufReader::new(file);
        let mut losses = Vec::new();
        let chunk_size = 1000;
        let debug_every_x = 1;
        for (i, lines) in reader.lines().chunks(chunk_size).into_iter().enumerate() {
            let before = Instant::now();
            let boards = lines
                .map(|line| BoardState::parse_fen(line.unwrap().as_str()).unwrap())
                .collect_vec();
            // TODO: need to bootstrap using heuristic first
            // TODO: Parallelize, use move_heuristic and leaf_heuristic
            let before_minimax_time = Instant::now();
            fn get_time_estimate(board: &BoardState) -> usize {
                board.pieces().len()
            }
            let scores: Vec<_> = boards
                .iter()
                .enumerate()
                .sorted_by_cached_key(|(_index, board)| get_time_estimate(board))
                .rev()
                .par_bridge()
                .map(|(index, board)| (index, search_white(board, SEARCH_DEPTH).unwrap().score))
                .collect();
            let scores = scores
                .into_iter()
                .sorted_by_key(|(index, _score)| *index)
                .map(|(_index, score)| score)
                .collect_vec();
            let minimax_time = before_minimax_time.elapsed();
            let representations = boards
                .iter()
                .map(|board| {
                    let representation: BoardRepresentation = board.into();
                    representation.to_float_array().to_vec()
                })
                .collect_vec();
            let representations = PyArray2::from_vec2(py, &representations).unwrap();
            let scores = PyArray1::from_vec(py, scores);

            let loss = model_instance.call_method1("learn_batch", (representations, scores));
            let loss = loss.unwrap_with_traceback(py).extract::<f32>().unwrap();
            losses.push(loss);
            let elapsed = before.elapsed();
            if i % debug_every_x == 0 {
                let avg: f32 =
                    losses.iter().rev().take(debug_every_x).sum::<f32>() / (debug_every_x as f32);
                let sequential = model_instance.call_method0("model_layer_weights")?;
                let extracted = SequentialModel::new_from_python(sequential).unwrap();
                let weights = extracted.layers().first().unwrap().to_string();
                println!(
                    "epoch={}, elapsed={:.2?}, per board={:.2?}, minimax per board={:.2?}",
                    i,
                    elapsed,
                    elapsed.div_f32(chunk_size as f32),
                    minimax_time.div_f32(chunk_size as f32),
                );
                println!("{} => {}", avg, weights);
            }
        }

        println!("Fetching Layers");
        let sequential = model_instance.call_method0("model_layer_weights")?;
        println!("Extracting Layers");
        let extracted = SequentialModel::new_from_python(sequential).unwrap();
        Ok(extracted)
    });
    let _sequential = result.map_err(|e| Error!("Unable to fetch model: {}", e))?;
    // let forwarded = sequential.forward(stacked_views);
    // println!("FORWARDED: {:?}", forwarded);
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "analyze") {
        run_analysis();
    }
    Ok(())
}
