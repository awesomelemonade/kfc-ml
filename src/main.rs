#![feature(const_for)]
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

fn main() {
    println!("Hello, world!");
}
