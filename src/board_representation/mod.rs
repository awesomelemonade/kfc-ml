core!();

use enum_map::Enum;

use crate::*;

#[cfg(test)]
mod board_representation_tests;

#[derive(Debug)]
pub struct BoardRepresentation {
    white: BoardRepresentationSide,
    black: BoardRepresentationSide,
}

impl BoardRepresentation {
    fn new() -> Self {
        Self {
            white: BoardRepresentationSide::new(),
            black: BoardRepresentationSide::new(),
        }
    }

    fn get_side_mut(&mut self, side: Side) -> &mut BoardRepresentationSide {
        match side {
            Side::White => &mut self.white,
            Side::Black => &mut self.black,
        }
    }

    const fn num_floats() -> usize {
        BoardRepresentationSide::num_floats() * 2
    }

    pub fn to_float_array(&self) -> [f32; BoardRepresentation::num_floats()] {
        const N: usize = BoardRepresentation::num_floats();
        let mut array = [0f32; N];
        let mut i = 0;
        for pieces in self.white.all_arrays() {
            for piece in pieces {
                piece.write_floats(&mut array[i..]);
                i += BoardRepresentationPiece::num_floats();
            }
        }
        for pieces in self.black.all_arrays() {
            for piece in pieces {
                piece.write_floats(&mut array[i..]);
                i += BoardRepresentationPiece::num_floats();
            }
        }
        array
    }
}

impl Default for BoardRepresentation {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&BoardState> for BoardRepresentation {
    fn from(state: &BoardState) -> Self {
        let mut board = BoardRepresentation::new();
        let mut counter = PieceCounter::new();
        for piece in state.pieces() {
            let Piece { side, kind, .. } = piece;
            let count = counter.get_mut(*side, *kind);
            board
                .get_side_mut(piece.side)
                .insert_piece(piece.kind, *count, piece.state.into());
            *count += 1;
        }
        board
    }
}

const NUM_SIDES: usize = std::mem::variant_count::<Side>();
const NUM_KINDS: usize = std::mem::variant_count::<PieceKind>();
struct PieceCounter {
    counts: [usize; NUM_SIDES * NUM_KINDS],
}

impl PieceCounter {
    pub fn new() -> Self {
        Self {
            counts: [0; NUM_SIDES * NUM_KINDS],
        }
    }
    pub fn get_mut(&mut self, side: Side, kind: PieceKind) -> &mut usize {
        let i = Side::into_usize(side) * NUM_KINDS + PieceKind::into_usize(kind);
        &mut self.counts[i]
    }
}

impl Default for PieceCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct BoardRepresentationSide {
    // pawn -> 8 slots
    // knight -> 2 slots
    // bishop -> 2 slots
    // rook -> 2 slots
    // queen -> 2 slots // extra slot for queens for promotion
    // king -> 1 slot
    pawns: [BoardRepresentationPiece; 8],
    knights: [BoardRepresentationPiece; 2],
    bishops: [BoardRepresentationPiece; 2],
    rooks: [BoardRepresentationPiece; 2],
    queens: [BoardRepresentationPiece; 2],
    king: [BoardRepresentationPiece; 1],
}

impl BoardRepresentationSide {
    fn insert_piece(&mut self, kind: PieceKind, position: usize, piece: BoardRepresentationPiece) {
        let array = self.get_array_mut(kind);
        array[position] = piece;
    }
    fn get_array_mut(&mut self, kind: PieceKind) -> &mut [BoardRepresentationPiece] {
        match kind {
            PieceKind::Pawn => &mut self.pawns,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Queen => &mut self.queens,
            PieceKind::King => &mut self.king,
        }
    }
    fn new() -> BoardRepresentationSide {
        Self {
            pawns: [BoardRepresentationPiece::Missing; 8],
            knights: [BoardRepresentationPiece::Missing; 2],
            bishops: [BoardRepresentationPiece::Missing; 2],
            rooks: [BoardRepresentationPiece::Missing; 2],
            queens: [BoardRepresentationPiece::Missing; 2],
            king: [BoardRepresentationPiece::Missing; 1],
        }
    }
    const fn all_arrays(&self) -> [&[BoardRepresentationPiece]; 6] {
        [
            &self.pawns,
            &self.knights,
            &self.bishops,
            &self.rooks,
            &self.queens,
            &self.king,
        ]
    }

    // TODO-someday: we want to create a #[derive(FloatSerializable)] ??
    const fn num_floats() -> usize {
        BoardRepresentationSide::num_pieces() * BoardRepresentationPiece::num_floats()
    }

    const fn num_pieces() -> usize {
        const NUM_PIECES_PER_SIDE: usize = 8 + 2 + 2 + 2 + 2 + 1;
        NUM_PIECES_PER_SIDE
    }
}

impl Default for BoardRepresentationSide {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum BoardRepresentationPiece {
    Stationary { x: f32, y: f32, cooldown: f32 },
    Moving { x: f32, y: f32 },
    Missing,
}

impl From<PieceState> for BoardRepresentationPiece {
    fn from(state: PieceState) -> Self {
        match state {
            PieceState::Stationary { position, cooldown } => BoardRepresentationPiece::Stationary {
                x: position.x as f32,
                y: position.y as f32,
                cooldown: cooldown as f32,
            },
            PieceState::Moving { x, y, .. } => BoardRepresentationPiece::Moving { x, y },
        }
    }
}

impl BoardRepresentationPiece {
    fn write_floats(&self, array: &mut [f32]) {
        match self {
            BoardRepresentationPiece::Missing => {
                array[0] = 0f32;
                array[1] = 0f32;
                array[2] = 0f32;
                array[3] = 0f32;
            }
            BoardRepresentationPiece::Stationary { x, y, cooldown } => {
                array[0] = 1f32;
                array[1] = *x;
                array[2] = *y;
                array[3] = *cooldown;
            }
            BoardRepresentationPiece::Moving { x, y } => {
                array[0] = 2f32;
                array[1] = *x;
                array[2] = *y;
                array[3] = 10f32; // moving pieces will have big cooldown
            }
        }
    }
    const fn num_floats() -> usize {
        4
    }
}
