core!();

use super::*;

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
    expect!(&*INITIAL_BLACK_PIECES, r#"
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
        }"#);
}
