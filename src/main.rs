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
use std::{
    cmp::Ordering,
    fmt::Display,
    fs::File,
    io::BufRead,
    io::BufReader,
    io::Write,
    time::{Duration, Instant, SystemTime},
};

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

fn evaluate_board_with_sequential(board: &BoardState, model: &SequentialModel) -> f32 {
    if let Some(end_state) = minimax::get_board_end_state(board) {
        return end_state.to_heuristic_score(100f32);
    }
    let mut board = board.clone();
    board.step_until_stationary_with_no_cooldown();
    let representation: BoardRepresentation = board.into();
    let array = Array1::from_vec(representation.to_float_array().to_vec());
    model.forward_one(array)
}
fn move_from_minimax_with_sequential(
    board: &BoardState,
    side: Side,
    model: &SequentialModel,
) -> BoardMove {
    search_white(board, SEARCH_DEPTH, |board| {
        evaluate_board_with_sequential(board, model)
    })
    .unwrap()
    .get_first_move_of_side(side)
}

fn move_from_minimax_with_heuristic(board: &BoardState, side: Side) -> BoardMove {
    search_white_with_heuristic(board, SEARCH_DEPTH)
        .unwrap()
        .get_first_move_of_side(side)
}

struct VersusStats {
    a_as_white: Counter<Outcome>,
    a_as_black: Counter<Outcome>,
    b_as_white: Counter<Outcome>,
    b_as_black: Counter<Outcome>,
}

impl Display for VersusStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let a = Outcome::values().map(|x| (x, self.a_as_white[&x]));
        let b = Outcome::values().map(|x| (x, self.a_as_black[&x]));
        let c = Outcome::values().map(|x| (x, self.b_as_white[&x]));
        let d = Outcome::values().map(|x| (x, self.b_as_black[&x]));
        let a_score: f32 = (self.a_as_white.clone() | self.a_as_black.clone())
            .into_iter()
            .map(|(outcome, count)| outcome.score() * count as f32)
            .sum();
        let b_score: f32 = (self.b_as_white.clone() | self.b_as_black.clone())
            .into_iter()
            .map(|(outcome, count)| outcome.score() * count as f32)
            .sum();
        writeln!(f, "Player A: score={a_score}")?;
        writeln!(f, "\tAs White: {a:?}")?;
        writeln!(f, "\tAs Black: {b:?}")?;
        writeln!(f, "Player B: score={b_score}")?;
        writeln!(f, "\tAs White: {c:?}")?;
        writeln!(f, "\tAs Black: {d:?}")
    }
}

// take some random boards, play one as white, one as black, all the way to the end, count wins/draws/losses of each player as white/black
fn get_versus_stats<F, G>(
    boards: &[BoardState],
    max_steps: usize,
    player_a: F,
    player_b: G,
) -> VersusStats
where
    F: Fn(&BoardState, Side) -> BoardMove + Sync,
    G: Fn(&BoardState, Side) -> BoardMove + Sync,
{
    let (a_as_white, a_as_black, b_as_white, b_as_black): (
        Counter<_>,
        Counter<_>,
        Counter<_>,
        Counter<_>,
    ) = parallel_map_prioritized_by_pieces(boards, |board| {
        println!("Playing as white... {}", board.to_stationary_fen().unwrap());
        let a = play_to_end_state(
            board.clone(),
            max_steps,
            |board| player_a(board, Side::White),
            |board| player_b(board, Side::Black),
        );
        println!("Playing as black... {}", board.to_stationary_fen().unwrap());
        let b = play_to_end_state(
            board.clone(),
            max_steps,
            |board| player_b(board, Side::White),
            |board| player_a(board, Side::Black),
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
        (
            outcome_a,
            outcome_b,
            outcome_b.opposite(),
            outcome_a.opposite(),
        )
    })
    .into_iter()
    .multiunzip();

    VersusStats {
        a_as_white,
        a_as_black,
        b_as_white,
        b_as_black,
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
enum Outcome {
    Win,
    Draw,
    Lose,
}

impl Outcome {
    // TODO-someday: to be derived
    fn values() -> [Outcome; 3] {
        [Outcome::Win, Outcome::Draw, Outcome::Lose]
    }
    fn score(self) -> f32 {
        match self {
            Outcome::Win => 1f32,
            Outcome::Draw => 0.5f32,
            Outcome::Lose => 0f32,
        }
    }
    fn opposite(self) -> Outcome {
        match self {
            Outcome::Win => Outcome::Lose,
            Outcome::Draw => Outcome::Draw,
            Outcome::Lose => Outcome::Win,
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
    let mut white_time = Duration::from_secs(0);
    let mut black_time = Duration::from_secs(0);
    let mut step_time = Duration::from_secs(0);
    while current_step < max_steps {
        if let Some(end_state) = minimax::get_board_end_state(&board) {
            println!(
                "TOTAL TIME: white={white_time:.2?}, black={black_time:.2?}, step={step_time:.2?}, num steps={current_step}"
            );
            return end_state;
        }
        let a = Instant::now();
        let white_move = white_player(&board);
        white_time += a.elapsed();
        let b = Instant::now();
        let black_move = black_player(&board);
        black_time += b.elapsed();
        debug_assert!(white_move.side() == Side::White);
        debug_assert!(black_move.side() == Side::Black);
        let c = Instant::now();
        board.step(&white_move, &black_move);
        step_time += c.elapsed();
        current_step += 1;
    }
    println!(
        "TOTAL TIME: white={white_time:.2?}, black={black_time:.2?}, step={step_time:.2?}, num steps={current_step}"
    );
    let material = minimax::evaluate_material_heuristic(&board);
    match material.total_cmp(&0f32) {
        Ordering::Less => EndState::Winner(Side::Black),
        Ordering::Equal => EndState::Draw,
        Ordering::Greater => EndState::Winner(Side::Black),
    }
}

fn parallel_map_prioritized_by_pieces<T, F>(boards: &[BoardState], f: F) -> Vec<T>
where
    F: Fn(&BoardState) -> T + Sync,
    T: Send,
{
    fn get_time_estimate(board: &BoardState) -> usize {
        board.pieces().len()
    }
    parallel_map_prioritized_by(
        boards,
        f,
        |board| -(get_time_estimate(board) as i32), // order by descending time estimate
    )
}

fn main() -> OrError<()> {
    let args: Vec<String> = std::env::args().collect();
    let run_all_epochs = args.iter().any(|arg| arg == "--all");
    let train = args.iter().any(|arg| arg == "--train");
    println!("Run all epochs? {run_all_epochs}");
    println!("train? {train}");
    let code = include_str!("./model.py");
    let result: PyResult<_> = Python::with_gil(|py| {
        println!("Importing Python Code");
        let module = PyModule::from_code(py, code, "model", "model")?;
        println!("Creating Model");
        let model = module.getattr("Model")?;
        let model_instance = model.call0()?;
        let mut current_sequential = {
            let sequential = model_instance.call_method0("model_layer_weights")?;
            let extracted = SequentialModel::new_from_python(sequential).unwrap();
            extracted
        };
        println!("Attempting to learn");

        let training_file = File::open("processed_random.fen").expect("No training set found");
        let reader = BufReader::new(training_file);
        let losses_filename = {
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
            format!("losses-{time}.txt")
        };
        let mut losses_file =
            File::create(losses_filename).expect("Unable to open file for writing");
        let mut losses = Vec::new();
        let chunk_size = 1000;
        let learn_batch_size = 10;
        let debug_every_x = 1;
        let debug_stats = false;
        let num_versus_games = if run_all_epochs { 20 } else { 1 };
        let versus_stats = true;
        let versus_stats_max_steps = if run_all_epochs { 1000 } else { 5 };
        for (i, lines) in reader.lines().chunks(chunk_size).into_iter().enumerate() {
            let before = Instant::now();
            let boards = lines
                .map(|line| BoardState::parse_fen(line.unwrap().as_str()).unwrap())
                .collect_vec();
            if versus_stats {
                println!("Computing versus stats");
                let versus_stats = get_versus_stats(
                    &boards[..num_versus_games],
                    versus_stats_max_steps,
                    |board, side| {
                        move_from_minimax_with_sequential(board, side, &current_sequential)
                    },
                    move_from_minimax_with_heuristic,
                );
                println!("{versus_stats}");
            }
            // TODO: need to bootstrap using heuristic first
            if train {
                let before_minimax_time = Instant::now();
                let scores = parallel_map_prioritized_by_pieces(&boards, |board| {
                    let before = Instant::now();
                    let out = search_white(board, SEARCH_DEPTH, |board| {
                        evaluate_board_with_sequential(board, &current_sequential)
                    })
                    .unwrap();
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
                });
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

                    let loss =
                        model_instance.call_method1("learn_batch", (representations, scores));
                    let loss = loss.unwrap_with_traceback(py).extract::<f32>().unwrap();
                    total_loss += loss;
                    num_losses += 1;
                }

                losses.push(total_loss / (num_losses as f32));
                current_sequential = {
                    let sequential = model_instance.call_method0("model_layer_weights")?;
                    let extracted = SequentialModel::new_from_python(sequential).unwrap();
                    extracted
                };
                let elapsed = before.elapsed();

                if i % debug_every_x == 0 {
                    let avg: f32 = losses.iter().rev().take(debug_every_x).sum::<f32>()
                        / (debug_every_x as f32);
                    println!(
                        "epoch={}, loss={}, elapsed={:.2?}, per board={:.2?}, minimax per board={:.2?}",
                        i,
                        avg,
                        elapsed,
                        elapsed.div_f32(chunk_size as f32),
                        minimax_time.div_f32(chunk_size as f32),
                    );
                    writeln!(losses_file, "{avg}").expect("Unable to write to losses_file");
                    // save weights
                    let weights_filename = {
                        let time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_millis();
                        format!("weights_epoch-{i}_{time}.tar")
                    };
                    model_instance.call_method1("save_state", (weights_filename,))?;
                }
            }
            if !run_all_epochs {
                break;
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
