#![feature(const_for)]
#![feature(let_chains)]

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

use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyModule},
};

// to actually move a piece, we set the moving_target

// board.calc_valid_moves_for_piece(PIECE)
// board.calc_valid_moves()
// board.calc_valid_next_states() # next states for combination moves
// board.calc_valid_next_states_one_move() # next states for one move
// board.step(moves) // Also resolves collisions

// As input to an ML model - impl from board?
struct BoardRepresentation {
    // for each side
    // pawn -> 8 slots
    // rook -> 2 slots
    // knight -> 2 slots
    // bishop -> 2 slots
    // queen -> 2 slots // extra slot for queens for promotion
    // king -> 1 slot
}

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let activators = PyModule::from_code(
            py,
            r#"

import numpy as np

def relu(x):
    return max(0.0, x)
def leaky_relu(x, slope=0.01):
    return x if x >= 0 else x * slope


    "#,
            "activators.py",
            "activators",
        )?;

        let relu_result: f64 = activators.getattr("relu")?.call1((-1.0,))?.extract()?;
        assert_eq!(relu_result, 0.0);

        let kwargs = [("slope", 0.2)].into_py_dict(py);
        let lrelu_result: f64 = activators
            .getattr("leaky_relu")?
            .call((-1.0,), Some(kwargs))?
            .extract()?;
        assert_eq!(lrelu_result, -0.2);
        Ok(())
    })
}
