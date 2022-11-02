core!();

use itertools::Itertools;

use super::*;

#[test]
fn test_forward_y() {
    expect!(forward_y(Side::White), "-1");
    expect!(forward_y(Side::Black), "1");
}

#[test]
fn test_initial_white_pieces() {
    expect!(
        &*INITIAL_WHITE_PIECES,
        r#"
        {
            Pawn: [
                Position {
                    x: 0,
                    y: 6,
                },
                Position {
                    x: 1,
                    y: 6,
                },
                Position {
                    x: 2,
                    y: 6,
                },
                Position {
                    x: 3,
                    y: 6,
                },
                Position {
                    x: 4,
                    y: 6,
                },
                Position {
                    x: 5,
                    y: 6,
                },
                Position {
                    x: 6,
                    y: 6,
                },
                Position {
                    x: 7,
                    y: 6,
                },
            ],
            Knight: [
                Position {
                    x: 1,
                    y: 7,
                },
                Position {
                    x: 6,
                    y: 7,
                },
            ],
            Bishop: [
                Position {
                    x: 2,
                    y: 7,
                },
                Position {
                    x: 5,
                    y: 7,
                },
            ],
            Rook: [
                Position {
                    x: 0,
                    y: 7,
                },
                Position {
                    x: 7,
                    y: 7,
                },
            ],
            Queen: [
                Position {
                    x: 3,
                    y: 7,
                },
            ],
            King: [
                Position {
                    x: 4,
                    y: 7,
                },
            ],
        }"#
    );
}

#[test]
fn test_initial_black_pieces() {
    expect!(
        &*INITIAL_BLACK_PIECES,
        r#"
        {
            Pawn: [
                Position {
                    x: 0,
                    y: 1,
                },
                Position {
                    x: 1,
                    y: 1,
                },
                Position {
                    x: 2,
                    y: 1,
                },
                Position {
                    x: 3,
                    y: 1,
                },
                Position {
                    x: 4,
                    y: 1,
                },
                Position {
                    x: 5,
                    y: 1,
                },
                Position {
                    x: 6,
                    y: 1,
                },
                Position {
                    x: 7,
                    y: 1,
                },
            ],
            Knight: [
                Position {
                    x: 1,
                    y: 0,
                },
                Position {
                    x: 6,
                    y: 0,
                },
            ],
            Bishop: [
                Position {
                    x: 2,
                    y: 0,
                },
                Position {
                    x: 5,
                    y: 0,
                },
            ],
            Rook: [
                Position {
                    x: 0,
                    y: 0,
                },
                Position {
                    x: 7,
                    y: 0,
                },
            ],
            Queen: [
                Position {
                    x: 3,
                    y: 0,
                },
            ],
            King: [
                Position {
                    x: 4,
                    y: 0,
                },
            ],
        }"#
    );
}

#[test]
fn test_initial_board_state() {
    let colors = BoardState::new_initial_state().to_stationary_map_color();
    let piece_types = BoardState::new_initial_state().to_stationary_map_type();
    expect!(
        colors,
        r#""BBBBBBBB\nBBBBBBBB\n........\n........\n........\n........\nWWWWWWWW\nWWWWWWWW""#
    );
    expect!(
        piece_types,
        r#""RNBQKBNR\nPPPPPPPP\n........\n........\n........\n........\nPPPPPPPP\nRNBQKBNR""#
    );
}

fn num_moves_per_type(moves: &Vec<BoardMove>) -> Vec<(PieceKind, usize)> {
    let all_kinds = vec![
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];
    all_kinds
        .iter()
        .map(|&kind| {
            let count = moves
                .iter()
                .filter(|board_move| {
                    if let BoardMove::Normal { piece, .. } = board_move {
                        piece.kind == kind
                    } else {
                        false
                    }
                })
                .count();
            (kind, count)
        })
        .collect_vec()
}

#[test]
fn test_initial_possible_moves() {
    let board = BoardState::new_initial_state();
    let moves = board.get_all_possible_moves(Side::White);
    let num_moves_per_type = num_moves_per_type(&moves);
    expect!(moves.len(), "20");
    expect!(
        num_moves_per_type,
        r#"
        [
            (
                Pawn,
                16,
            ),
            (
                Knight,
                4,
            ),
            (
                Bishop,
                0,
            ),
            (
                Rook,
                0,
            ),
            (
                Queen,
                0,
            ),
            (
                King,
                0,
            ),
        ]"#
    );
}

#[test]
fn test_random_state_possible_moves() {
    let board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();
    let moves = board.get_all_possible_moves(Side::White);
    let num_moves_per_type = num_moves_per_type(&moves);
    expect!(moves.len(), "38");
    expect!(
        num_moves_per_type,
        r#"
        [
            (
                Pawn,
                4,
            ),
            (
                Knight,
                9,
            ),
            (
                Bishop,
                2,
            ),
            (
                Rook,
                0,
            ),
            (
                Queen,
                16,
            ),
            (
                King,
                7,
            ),
        ]"#
    );
}
