core!();

use super::*;

pub fn forward_y(side: Side) -> i32 {
    match side {
        Side::White => -1i32,
        Side::Black => 1i32,
    }
}

pub fn to_char_map<F>(func: F) -> String
where
    F: Fn(Position) -> char,
{
    let mut all: Vec<String> = Vec::new();
    for row in 0..BOARD_SIZE {
        let mut buffer = Vec::new();
        for col in 0..BOARD_SIZE {
            let c = func((col, row).into());
            buffer.push(c);
        }
        all.push(buffer.iter().collect());
    }
    all.join("\n")
}
