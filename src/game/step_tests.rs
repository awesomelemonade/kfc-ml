core!();

use super::*;

fn snapshots_n<F>(board: &mut BoardState, n: u32, f: F) -> Vec<String>
where
    F: Fn(&BoardState) -> String,
{
    let mut all_steps = Vec::new();
    for _ in 0..n {
        all_steps.push(f(board));
        board.step();
    }
    all_steps.push(f(board));
    all_steps
}

fn snapshots_until_stationary(board: &mut BoardState) -> Vec<String> {
    let mut all_steps = Vec::new();
    while !board.is_all_pieces_stationary() {
        all_steps.push(board.to_stationary_map_combo());
        board.step();
    }
    all_steps.push(board.to_stationary_map_combo());
    all_steps
}

#[test]
fn test_step_no_move() {
    let mut board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();
    let before_map = board.to_stationary_map_combo();
    board.step();
    let after_map = board.to_stationary_map_combo();
    assert_eq!(before_map, after_map);
}

#[test]
fn test_step_rand_move() {
    let mut board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();
    let board_move = board.get_all_possible_moves(Side::White).pop().unwrap();
    board.apply_move(&board_move);
    let snapshots = snapshots_until_stationary(&mut board);
    expect!(
        board_move,
        r#"
        Normal {
            piece: Piece {
                side: White,
                kind: Knight,
                state: Stationary {
                    position: Position {
                        x: 6,
                        y: 5,
                    },
                    cooldown: 0,
                },
            },
            target: Position {
                x: 7,
                y: 7,
            },
        }"#
    );
    expect!(
        snapshots,
        r#"
        [
            "...N....\nb...P...\n.....p.B\n..Q..bPP\nPnK.....\nr.......\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n..Q..bPP\nPnK.....\nr.......\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n..Q..bPP\nPnK.....\nr.......\n.......k\n...r...N",
        ]"#
    );
}

#[test]
fn test_step_capture_stationary() {
    let mut board = BoardState::parse_fen("3N4/b3P3/5p1B/2Q2bPP/PnK5/r5N1/7k/3r4").unwrap();
    let queen = board
        .pieces()
        .iter()
        .find(|piece| piece.side == Side::White && piece.kind == PieceKind::Queen)
        .unwrap();
    let board_move = BoardMove::Normal {
        piece: *queen,
        target: (5u32, 3u32).into(),
    };
    board.apply_move(&board_move);
    let snapshots = snapshots_until_stationary(&mut board);
    expect!(
        board_move,
        r#"
        Normal {
            piece: Piece {
                side: White,
                kind: Queen,
                state: Stationary {
                    position: Position {
                        x: 2,
                        y: 3,
                    },
                    cooldown: 0,
                },
            },
            target: Position {
                x: 5,
                y: 3,
            },
        }"#
    );
    expect!(
        snapshots,
        r#"
        [
            "...N....\nb...P...\n.....p.B\n.....bPP\nPnK.....\nr.....N.\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n.....bPP\nPnK.....\nr.....N.\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n.....bPP\nPnK.....\nr.....N.\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n.....QPP\nPnK.....\nr.....N.\n.......k\n...r....",
        ]"#
    );
}

#[test]
fn test_step_capture_moving() {
    let mut board = BoardState::parse_fen("8/8/8/8/8/8/8/R6r").unwrap();
    let white_target: Position = (7u32, 7u32).into();
    let black_target: Position = (0u32, 7u32).into();
    let white_moves = board.get_all_possible_moves(Side::White);
    let white_move = white_moves
        .iter()
        .find(|m| matches!(m, BoardMove::Normal { target, .. } if target == &white_target))
        .unwrap();
    let black_moves = board.get_all_possible_moves(Side::Black);
    let black_move = black_moves
        .iter()
        .find(|m| matches!(m, BoardMove::Normal { target, .. } if target == &black_target))
        .unwrap();
    board.apply_move(white_move);
    board.step();
    board.apply_move(black_move);
    board.step_n(15);
    expect!(
        board.to_stationary_map_combo(),
        r#""........\n........\n........\n........\n........\n........\n........\n.......R""#
    );
}

#[test]
fn test_cooldown() {
    let mut board = BoardState::parse_fen("8/8/8/8/4P3/8/8/8").unwrap();
    let board_move = board.get_all_possible_moves(Side::White).pop().unwrap();
    board.apply_move(&board_move);
    let snapshots = snapshots_n(&mut board, 15, |x| x.to_stationary_map_cooldowns());
    expect!(
        snapshots,
        r#"
        [
            "........\n........\n........\n........\n........\n........\n........\n........",
            "........\n........\n........\n....X...\n........\n........\n........\n........",
            "........\n........\n........\n....9...\n........\n........\n........\n........",
            "........\n........\n........\n....8...\n........\n........\n........\n........",
            "........\n........\n........\n....7...\n........\n........\n........\n........",
            "........\n........\n........\n....6...\n........\n........\n........\n........",
            "........\n........\n........\n....5...\n........\n........\n........\n........",
            "........\n........\n........\n....4...\n........\n........\n........\n........",
            "........\n........\n........\n....3...\n........\n........\n........\n........",
            "........\n........\n........\n....2...\n........\n........\n........\n........",
            "........\n........\n........\n....1...\n........\n........\n........\n........",
            "........\n........\n........\n....0...\n........\n........\n........\n........",
            "........\n........\n........\n....0...\n........\n........\n........\n........",
            "........\n........\n........\n....0...\n........\n........\n........\n........",
            "........\n........\n........\n....0...\n........\n........\n........\n........",
            "........\n........\n........\n....0...\n........\n........\n........\n........",
        ]"#
    );
}

#[test]
fn test_no_move_while_cooldown() {
    let mut board = BoardState::parse_fen("8/8/8/8/4P3/8/8/8").unwrap();
    let board_move = board.get_all_possible_moves(Side::White).pop().unwrap();
    board.apply_move(&board_move);
    board.step_n(5);
    let possible_moves = board.get_all_possible_moves(Side::White);
    expect!(possible_moves, "[]");
    board.step_n(10);
    let possible_moves = board.get_all_possible_moves(Side::White);
    expect!(
        possible_moves,
        r#"
        [
            Normal {
                piece: Piece {
                    side: White,
                    kind: Pawn,
                    state: Stationary {
                        position: Position {
                            x: 4,
                            y: 3,
                        },
                        cooldown: 0,
                    },
                },
                target: Position {
                    x: 4,
                    y: 2,
                },
            },
        ]"#
    );
}

#[test]
fn test_pawn_promotion() {
    let mut board = BoardState::parse_fen("8/P7/8/8/8/8/8/8").unwrap();
    let board_move = board.get_all_possible_moves(Side::White).pop().unwrap();
    expect!(
        &board_move,
        r#"
        Normal {
            piece: Piece {
                side: White,
                kind: Pawn,
                state: Stationary {
                    position: Position {
                        x: 0,
                        y: 1,
                    },
                    cooldown: 0,
                },
            },
            target: Position {
                x: 0,
                y: 0,
            },
        }"#
    );
    board.apply_move(&board_move);
    board.step();
    expect!(
        board.to_stationary_map_combo(),
        r#""Q.......\n........\n........\n........\n........\n........\n........\n........""#
    );
}
