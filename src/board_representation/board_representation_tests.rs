core!();

use super::*;

#[test]
fn test_initial_board() {
    let board = BoardState::new_initial_state();
    let representation: BoardRepresentation = board.into();
    expect!(
        representation.to_float_array(),
        r#"
        [
            1.0,
            0.0,
            6.0,
            1.0,
            1.0,
            6.0,
            1.0,
            2.0,
            6.0,
            1.0,
            3.0,
            6.0,
            1.0,
            4.0,
            6.0,
            1.0,
            5.0,
            6.0,
            1.0,
            6.0,
            6.0,
            1.0,
            7.0,
            6.0,
            1.0,
            1.0,
            7.0,
            1.0,
            6.0,
            7.0,
            1.0,
            2.0,
            7.0,
            1.0,
            5.0,
            7.0,
            1.0,
            0.0,
            7.0,
            1.0,
            7.0,
            7.0,
            1.0,
            3.0,
            7.0,
            0.0,
            0.0,
            0.0,
            1.0,
            4.0,
            7.0,
            1.0,
            0.0,
            1.0,
            1.0,
            1.0,
            1.0,
            1.0,
            2.0,
            1.0,
            1.0,
            3.0,
            1.0,
            1.0,
            4.0,
            1.0,
            1.0,
            5.0,
            1.0,
            1.0,
            6.0,
            1.0,
            1.0,
            7.0,
            1.0,
            1.0,
            1.0,
            0.0,
            1.0,
            6.0,
            0.0,
            1.0,
            2.0,
            0.0,
            1.0,
            5.0,
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            7.0,
            0.0,
            1.0,
            3.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            4.0,
            0.0,
        ]"#
    );
    expect!(
        representation,
        r#"
        BoardRepresentation {
            white: BoardRepresentationSide {
                pawns: [
                    Stationary {
                        x: 0.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 1.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 2.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 3.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 4.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 5.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 6.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 7.0,
                        y: 6.0,
                        cooldown: 0.0,
                    },
                ],
                knights: [
                    Stationary {
                        x: 1.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 6.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                ],
                bishops: [
                    Stationary {
                        x: 2.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 5.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                ],
                rooks: [
                    Stationary {
                        x: 0.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 7.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                ],
                queens: [
                    Stationary {
                        x: 3.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                    Missing,
                ],
                king: [
                    Stationary {
                        x: 4.0,
                        y: 7.0,
                        cooldown: 0.0,
                    },
                ],
            },
            black: BoardRepresentationSide {
                pawns: [
                    Stationary {
                        x: 0.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 1.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 2.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 3.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 4.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 5.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 6.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 7.0,
                        y: 1.0,
                        cooldown: 0.0,
                    },
                ],
                knights: [
                    Stationary {
                        x: 1.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 6.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                ],
                bishops: [
                    Stationary {
                        x: 2.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 5.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                ],
                rooks: [
                    Stationary {
                        x: 0.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 7.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                ],
                queens: [
                    Stationary {
                        x: 3.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                    Missing,
                ],
                king: [
                    Stationary {
                        x: 4.0,
                        y: 0.0,
                        cooldown: 0.0,
                    },
                ],
            },
        }"#
    );
}

#[test]
fn test_multi_queen() {
    let board = BoardState::parse_fen("8/8/8/8/2QQQ3/8/8/8").unwrap();
    let representation: BoardRepresentation = board.into();
    expect!(
        representation,
        r#"
        BoardRepresentation {
            white: BoardRepresentationSide {
                pawns: [
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                ],
                knights: [
                    Missing,
                    Missing,
                ],
                bishops: [
                    Missing,
                    Missing,
                ],
                rooks: [
                    Missing,
                    Missing,
                ],
                queens: [
                    Stationary {
                        x: 2.0,
                        y: 4.0,
                        cooldown: 0.0,
                    },
                    Stationary {
                        x: 3.0,
                        y: 4.0,
                        cooldown: 0.0,
                    },
                ],
                king: [
                    Missing,
                ],
            },
            black: BoardRepresentationSide {
                pawns: [
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                    Missing,
                ],
                knights: [
                    Missing,
                    Missing,
                ],
                bishops: [
                    Missing,
                    Missing,
                ],
                rooks: [
                    Missing,
                    Missing,
                ],
                queens: [
                    Missing,
                    Missing,
                ],
                king: [
                    Missing,
                ],
            },
        }"#
    )
}
