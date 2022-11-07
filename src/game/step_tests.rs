core!();

use super::*;

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
fn test_step_capture() {
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
            "...N....\nb...P...\n.....p.B\n......PP\nPnK.....\nr.....N.\n.......k\n...r....",
            "...N....\nb...P...\n.....p.B\n.....QPP\nPnK.....\nr.....N.\n.......k\n...r....",
        ]"#
    );
}
