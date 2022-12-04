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
use std::{cmp::Ordering, fs::File, io::BufRead, io::BufReader, io::Write, time::Instant};

use counter::Counter;
pub use game::*;

mod minimax;
pub use minimax::*;

mod board_representation;
pub use board_representation::*;

pub mod util;

mod sequential;

use itertools::Itertools;

use numpy::{ndarray::Array1, PyArray1, PyArray2};
use pyo3::{prelude::*, types::PyModule};
use rand::seq::SliceRandom;

use crate::{
    sequential::SequentialModel,
    util::{parallel_map_prioritized_by, UnwrapWithTraceback},
};

const SEARCH_DEPTH: u32 = 2;

fn random_move(board: &BoardState, side: Side) -> BoardMove {
    let all_moves = board.get_all_possible_moves(side);
    all_moves.choose(&mut rand::thread_rng()).cloned().unwrap()
}

fn move_from_minimax_with_sequential(board: &BoardState, model: &SequentialModel) -> BoardMove {
    // TODO: only supports Side::White!!
    search_white(board, SEARCH_DEPTH, |board| {
        let representation: BoardRepresentation = board.into();
        let array = Array1::from_vec(representation.to_float_array().to_vec());
        model.forward_one(array)
    })
    .unwrap()
    .moves
    .first()
    .cloned()
    .unwrap()
}

fn move_from_minimax_with_heuristic(board: &BoardState) -> BoardMove {
    // TODO: only supports Side::White!!
    search_white_with_heuristic(board, SEARCH_DEPTH)
        .unwrap()
        .moves
        .first()
        .cloned()
        .unwrap()
}

// take some random boards, play one as white, one as black, all the way to the end, count wins/draws/losses of each player as white/black
fn get_versus_stats<F, G>(
    boards: &Vec<BoardState>,
    max_steps: usize,
    mut player_a: F,
    mut player_b: G,
) -> (Counter<Outcome>, Counter<Outcome>)
where
    F: FnMut(&BoardState) -> BoardMove,
    G: FnMut(&BoardState) -> BoardMove,
{
    let (outcomes_as_white, outcomes_as_black): (Vec<_>, Vec<_>) = boards
        .iter()
        .map(|board| {
            let a = play_to_end_state(
                board.clone(),
                max_steps,
                |board| player_a(board),
                |board| player_b(board),
            );
            let b = play_to_end_state(
                board.clone(),
                max_steps,
                |board| player_b(board),
                |board| player_a(board),
            );
            let outcome_a = match a {
                EndState::Winner(side) => match side {
                    Side::White => Outcome::Win,
                    Side::Black => Outcome::Lose,
                },
                EndState::Draw => Outcome::Draw,
            };
            let outcome_b = match b {
                EndState::Winner(side) => match side {
                    Side::White => Outcome::Lose,
                    Side::Black => Outcome::Win,
                },
                EndState::Draw => Outcome::Draw,
            };
            (outcome_a, outcome_b)
        })
        .unzip();
    // change to counters
    (
        outcomes_as_white.into_iter().collect::<Counter<_>>(),
        outcomes_as_black.into_iter().collect::<Counter<_>>(),
    )
}

#[derive(Hash, Eq, PartialEq)]
enum Outcome {
    Win,
    Draw,
    Lose,
}

impl Outcome {
    fn score(self) -> f32 {
        match self {
            Outcome::Win => 1f32,
            Outcome::Draw => 0.5f32,
            Outcome::Lose => 0f32,
        }
    }
}

fn play_to_end_state<F, G>(
    mut board: BoardState,
    max_steps: usize,
    mut white_player: F,
    mut black_player: G,
) -> EndState
where
    F: FnMut(&BoardState) -> BoardMove,
    G: FnMut(&BoardState) -> BoardMove,
{
    let mut current_step = 0;
    while current_step < max_steps {
        if let Some(end_state) = minimax::get_board_end_state(&board) {
            return end_state;
        }
        let white_move = white_player(&board);
        let black_move = black_player(&board);
        debug_assert!(white_move.side() == Side::White);
        debug_assert!(black_move.side() == Side::Black);
        board.step(&white_move, &black_move);
        current_step += 1;
    }
    let material = minimax::evaluate_material_heuristic(&board);
    match material.total_cmp(&0f32) {
        Ordering::Less => EndState::Winner(Side::Black),
        Ordering::Equal => EndState::Draw,
        Ordering::Greater => EndState::Winner(Side::Black),
    }
}

fn main() -> OrError<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "test") {
        println!("test");
    }
    let code = include_str!("./model.py");
    let result: PyResult<_> = Python::with_gil(|py| {
        println!("Importing Python Code");
        let module = PyModule::from_code(py, code, "model", "model")?;
        println!("Creating Model");
        let model = module.getattr("Model")?;
        let model_instance = model.call0()?;

        println!("Attempting to learn");

        let training_file = File::open("processed_random.fen").expect("No training set found");
        let reader = BufReader::new(training_file);
        let mut losses_file = File::create("losses.txt").expect("Unable to open file for writing");
        let mut weights_file =
            File::create("weights.txt").expect("Unable to open file for writing");
        let mut losses = Vec::new();
        let chunk_size = 1000;
        let learn_batch_size = 10;
        let debug_every_x = 1;
        let debug_stats = false;
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
            let scores = parallel_map_prioritized_by(
                &boards,
                |board| {
                    let before = Instant::now();
                    let out = search_white_with_heuristic(board, SEARCH_DEPTH).unwrap();
                    let score = out.score;
                    let elapsed = before.elapsed();
                    // number of pieces and elapsed
                    if debug_stats {
                        let best_piece = if let Some(best_move) = out.moves.first() {
                            match best_move {
                                BoardMove::None(_) => "None".to_owned(),
                                BoardMove::LongCastle(_) => "LongCastle".to_owned(),
                                BoardMove::ShortCastle(_) => "ShortCastle".to_owned(),
                                BoardMove::Normal { piece, .. } => format!("{:?}", piece.kind),
                            }
                        } else {
                            "N/A".to_owned()
                        };
                        println!(
                            "{}, {}, {}, {}, {}, {}, {}",
                            board.pieces().len(),
                            out.num_leaves,
                            out.num_regular_nodes,
                            out.num_quiescent_nodes,
                            elapsed.as_millis(),
                            board.to_stationary_fen().unwrap(),
                            best_piece,
                        );
                    }
                    score
                },
                |board| -(get_time_estimate(board) as i32), // order by descending time estimate
            );
            let minimax_time = before_minimax_time.elapsed();

            let mut total_loss = 0f32;
            let mut num_losses = 0;
            for chunks in &boards
                .into_iter()
                .zip_eq(scores.into_iter())
                .chunks(learn_batch_size)
            {
                let (boards, scores): (Vec<_>, Vec<_>) = chunks.unzip();

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
                total_loss += loss;
                num_losses += 1;
            }

            losses.push(total_loss / (num_losses as f32));
            let elapsed = before.elapsed();
            if i % debug_every_x == 0 {
                let avg: f32 =
                    losses.iter().rev().take(debug_every_x).sum::<f32>() / (debug_every_x as f32);
                let sequential = model_instance.call_method0("model_layer_weights")?;
                let extracted = SequentialModel::new_from_python(sequential).unwrap();
                let weights = extracted
                    .layers()
                    .iter()
                    .map(|layer| layer.to_raw_string())
                    .join("\n");

                println!(
                    "epoch={}, loss={}, elapsed={:.2?}, per board={:.2?}, minimax per board={:.2?}",
                    i,
                    avg,
                    elapsed,
                    elapsed.div_f32(chunk_size as f32),
                    minimax_time.div_f32(chunk_size as f32),
                );
                writeln!(losses_file, "{}", avg).expect("Unable to write to losses_file");
                writeln!(weights_file, "epoch={}\n{}", i, weights)
                    .expect("Unable to write to weights_file");
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
    Ok(())
}
