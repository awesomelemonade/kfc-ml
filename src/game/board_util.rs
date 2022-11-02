core!();

use super::*;

pub fn forward_y(side: Side) -> i32 {
    match side {
        Side::White => -1i32,
        Side::Black => 1i32,
    }
}
