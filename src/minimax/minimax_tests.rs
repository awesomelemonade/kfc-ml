core!();

use anyhow::Ok;

use super::*;

lazy_static! {
    pub static ref BOARD_STATES: Vec<BoardState> = {
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
        fen_strings
            .split('\n')
            .map(|fen| BoardState::parse_fen(fen).unwrap())
            .collect_vec()
    };
}

const SEARCH_DEPTH: u32 = 2;

fn to_compressed_debug(board_move: &BoardMove) -> String {
    match board_move {
        BoardMove::None(side) => format!("None: {:?}", side),
        BoardMove::LongCastle(side) => format!("LongCastle: {:?}", side),
        BoardMove::ShortCastle(side) => format!("ShortCastle: {:?}", side),
        BoardMove::Normal {
            piece: Piece { side, kind, state },
            target,
        } => {
            if let PieceState::Stationary { position, .. } = state {
                format!(
                    "side={:?}, kind={:?}, move=[{}, {}] -> [{}, {}]",
                    side, kind, position.x, position.y, target.x, target.y
                )
            } else {
                String::from("Unknown: Moving?")
            }
        }
    }
}

#[test]
fn test_moves() {
    let board_moves: OrError<Vec<_>> = BOARD_STATES
        .iter()
        .map(|state| {
            let MinimaxOutputInfo {
                score,
                num_leaves,
                moves,
                ..
            } = search_white_with_heuristic(state, SEARCH_DEPTH).unwrap();
            let moves = moves.iter().map(to_compressed_debug).collect_vec();
            Ok((score, num_leaves, moves))
        })
        .collect();
    let board_moves = board_moves.unwrap();
    let average_leaves = (board_moves
        .iter()
        .map(|(_, num_leaves, _)| num_leaves)
        .sum::<u32>() as f32)
        / (BOARD_STATES.len() as f32);
    expect!(average_leaves, "88668.164");
    expect!(
        board_moves,
        r#"
        [
            (
                -13.0,
                356860,
                [
                    "side=White, kind=King, move=[4, 7] -> [4, 6]",
                    "side=Black, kind=Pawn, move=[7, 1] -> [7, 3]",
                    "side=White, kind=Rook, move=[7, 7] -> [1, 7]",
                    "side=Black, kind=Queen, move=[2, 0] -> [0, 0]",
                    "None: White",
                    "None: Black",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                2.0,
                24267,
                [
                    "side=White, kind=Queen, move=[1, 3] -> [1, 6]",
                    "side=Black, kind=King, move=[3, 2] -> [4, 1]",
                    "side=White, kind=Rook, move=[4, 6] -> [4, 7]",
                    "side=Black, kind=Pawn, move=[2, 2] -> [2, 3]",
                    "None: White",
                    "None: Black",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                -15.0,
                64440,
                [
                    "side=White, kind=Pawn, move=[2, 5] -> [1, 4]",
                    "side=Black, kind=Rook, move=[0, 0] -> [1, 0]",
                    "side=White, kind=Bishop, move=[4, 1] -> [3, 0]",
                    "side=Black, kind=Queen, move=[4, 3] -> [4, 2]",
                    "None: White",
                    "side=Black, kind=Knight, move=[3, 5] -> [1, 4]",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                30.0,
                35284,
                [
                    "side=White, kind=Queen, move=[6, 3] -> [5, 4]",
                    "side=Black, kind=Bishop, move=[5, 2] -> [3, 4]",
                    "side=White, kind=Knight, move=[4, 4] -> [6, 5]",
                    "side=Black, kind=King, move=[1, 5] -> [0, 4]",
                    "side=White, kind=Pawn, move=[1, 1] -> [1, 0]",
                    "None: Black",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                -14.0,
                277030,
                [
                    "side=White, kind=King, move=[5, 2] -> [4, 1]",
                    "side=Black, kind=Knight, move=[4, 1] -> [2, 2]",
                    "side=White, kind=Rook, move=[7, 0] -> [0, 0]",
                    "side=Black, kind=Queen, move=[0, 0] -> [1, 1]",
                    "None: White",
                    "side=Black, kind=Pawn, move=[4, 6] -> [4, 7]",
                    "None: White",
                    "side=Black, kind=Pawn, move=[6, 6] -> [6, 7]",
                ],
            ),
            (
                6.0,
                39992,
                [
                    "side=White, kind=Knight, move=[5, 0] -> [3, 1]",
                    "side=Black, kind=Bishop, move=[6, 1] -> [5, 2]",
                    "side=White, kind=Pawn, move=[1, 1] -> [1, 0]",
                    "side=Black, kind=Rook, move=[0, 7] -> [2, 7]",
                    "None: White",
                    "None: Black",
                    "None: White",
                    "side=Black, kind=Pawn, move=[4, 6] -> [4, 7]",
                ],
            ),
            (
                -8.0,
                24237,
                [
                    "side=White, kind=Pawn, move=[1, 6] -> [0, 5]",
                    "side=Black, kind=Pawn, move=[0, 5] -> [0, 6]",
                    "side=White, kind=Pawn, move=[5, 2] -> [5, 1]",
                    "side=Black, kind=Rook, move=[1, 0] -> [0, 0]",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                3.0,
                65362,
                [
                    "side=White, kind=Rook, move=[0, 5] -> [0, 6]",
                    "side=Black, kind=Pawn, move=[0, 2] -> [1, 3]",
                    "side=White, kind=Knight, move=[2, 3] -> [0, 2]",
                    "side=Black, kind=King, move=[2, 1] -> [2, 2]",
                    "None: White",
                    "None: Black",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                -20.0,
                14890,
                [
                    "side=White, kind=Bishop, move=[7, 0] -> [5, 2]",
                    "side=Black, kind=Rook, move=[3, 1] -> [2, 1]",
                    "side=White, kind=Bishop, move=[6, 2] -> [5, 1]",
                    "side=Black, kind=Bishop, move=[1, 3] -> [0, 2]",
                    "None: White",
                    "side=Black, kind=Pawn, move=[2, 6] -> [2, 7]",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                -7.0,
                93397,
                [
                    "side=White, kind=Pawn, move=[3, 2] -> [4, 1]",
                    "side=Black, kind=Bishop, move=[0, 3] -> [1, 2]",
                    "side=White, kind=Rook, move=[6, 7] -> [6, 4]",
                    "side=Black, kind=King, move=[7, 1] -> [6, 0]",
                    "None: White",
                    "side=Black, kind=Queen, move=[0, 5] -> [4, 1]",
                    "None: White",
                    "None: Black",
                ],
            ),
            (
                3.0,
                14068,
                [
                    "side=White, kind=Queen, move=[0, 0] -> [1, 1]",
                    "side=Black, kind=Queen, move=[1, 1] -> [2, 2]",
                    "side=White, kind=Pawn, move=[4, 3] -> [4, 2]",
                    "side=Black, kind=Bishop, move=[3, 0] -> [4, 1]",
                    "side=White, kind=Knight, move=[6, 0] -> [4, 1]",
                    "None: Black",
                    "None: White",
                    "side=Black, kind=Knight, move=[6, 2] -> [4, 1]",
                ],
            ),
            (
                8.0,
                54191,
                [
                    "side=White, kind=Rook, move=[1, 6] -> [3, 6]",
                    "side=Black, kind=Pawn, move=[6, 1] -> [5, 2]",
                    "side=White, kind=Knight, move=[0, 3] -> [2, 2]",
                    "side=Black, kind=Pawn, move=[3, 6] -> [4, 7]",
                    "None: White",
                    "None: Black",
                    "side=White, kind=King, move=[4, 6] -> [4, 7]",
                    "None: Black",
                ],
            ),
        ]"#
    );
}
