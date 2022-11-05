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
use minimax::white_move;

use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyModule},
};

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

fn main() -> PyResult<()> {
    let board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();

    let score = white_move(&board, 0);
    println!("score={:?}", score);

    //     Python::with_gil(|py| {
    //         let activators = PyModule::from_code(
    //             py,
    //             r#"

    // import numpy as np

    // def relu(x):
    //     return max(0.0, x)
    // def leaky_relu(x, slope=0.01):
    //     return x if x >= 0 else x * slope

    //     "#,
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
    //         Ok(())
    //     })
    Ok(())
}
