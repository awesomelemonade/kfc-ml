#![feature(const_for)]
#![feature(let_chains)]
#![feature(array_zip)]
#![feature(variant_count)]

use numpy::PyArray;
use pyo3::{types::IntoPyDict, PyResult, Python, prelude::*};

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
pub use minimax::*;

mod board_representation;
pub use board_representation::*;

use itertools::Itertools;

// use pyo3::{
//     prelude::*,
//     types::{IntoPyDict, PyModule},
// };
use rand::seq::SliceRandom;

const SEARCH_DEPTH: u32 = 2;

fn get_diff(board: &BoardState) -> f32 {
    let all_moves = board.get_all_possible_moves(Side::White);
    let random_move = all_moves.choose(&mut rand::thread_rng());
    let minimax_output = white_move(board, SEARCH_DEPTH, f32::NEG_INFINITY, f32::INFINITY);
    let minimax_move = match minimax_output {
        MinimaxOutput::Node { best_move, .. } => best_move,
        MinimaxOutput::Leaf { .. } => None,
    };
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


fn single_sample(board: &BoardState) -> PyResult<f32> {

    let model_file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/model/model.py"));

    let representation: BoardRepresentation = board.into();

    let from_python: PyResult<f32> = Python::with_gil(|py| {
        let np = py.import("numpy")?;
        let locals = [("np", np)].into_py_dict(py);

        let module = PyModule::from_code(py, model_file, "model.model", "model.model")?;


        let gil = pyo3::Python::acquire_gil();
        let pyarray = PyArray::from_vec(gil.python(), representation.to_float_array().to_vec());

        
        let model = module.getattr("Model")?;
        let model_instance = model.call0()?;

        let result = model_instance.call_method1("eval_single", (pyarray,)).unwrap();
        //let result = model.call_method1("forward", (pyarray,))?;

        // let instance = module.getattr("test")?;
        // let result = instance.call_method1("testfunction", (pyarray,))?;
        // Ok(result.extract::<f32>()?)
        Ok(result.extract::<f32>()?)
    });

    Ok(from_python?)
}


fn train_variation(initial_board: &BoardState) -> () {

    println!("train_variation");
    let mut board_mut = initial_board.clone();
    let minimax_output = white_move(initial_board, SEARCH_DEPTH, f32::NEG_INFINITY, f32::INFINITY);
    let minimax_move = match minimax_output {
        MinimaxOutput::Node { best_move, .. } => best_move,
        MinimaxOutput::Leaf { .. } => None,
    };
    let black_moves = board_mut.get_all_possible_moves(Side::Black);
    let random_black_move = black_moves.choose(&mut rand::thread_rng()).unwrap();
    board_mut.apply_move(&minimax_move.unwrap());
    board_mut.apply_move(random_black_move);
    board_mut.step();

    let final_board = &board_mut;

    let variation: [&BoardState; 2] = [initial_board, &final_board];

    let variation_arrays = variation
    .iter()
    .map(|state| {
        let representation: BoardRepresentation = (*state).into();
        representation.to_float_array()
    })
    .collect_vec();

    // println!("variation: {:?}", variation);
    
    let model_file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/model/model.py"));

    let from_python: PyResult<f32> = Python::with_gil(|py| {
        let np = py.import("numpy")?;
        let locals = [("np", np)].into_py_dict(py);

        let module = PyModule::from_code(py, model_file, "model.model", "model.model")?;


        let gil = pyo3::Python::acquire_gil();
        let pyarrays = variation_arrays.iter().map(|array| PyArray::from_vec(gil.python(), array.to_vec())).collect_vec();
        
        let model = module.getattr("Model")?;
        let model_instance = model.call0()?;

        let result = model_instance.call_method1("train_variation", (pyarrays,)).unwrap();
        //let result = model.call_method1("forward", (pyarray,))?;

        // let instance = module.getattr("test")?;
        // let result = instance.call_method1("testfunction", (pyarray,))?;
        // Ok(result.extract::<f32>()?)
        Ok(result.extract::<f32>()?)
    });

    println!("from_python: {:?}", from_python);
}


fn main() {
    /*
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
        .collect_vec(); // TODO: map to numpy array
    println!("{:?}", floats);

    let scores = board_states.iter().map(get_diff).collect_vec();
    println!("scores={:?}", scores);
    let average = scores.iter().sum::<f32>() / scores.len() as f32;
    println!("avg={:?}", average);
    */

    let model_file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/model/model.py"));
    
    let board = BoardState::parse_fen("2qn3B/P2k3p/5b1Q/8/8/Pb1r3P/1p5P/4K2R").unwrap();

    let sample = train_variation(&board);
    // println!("sample={:?}", sample);
    
}
