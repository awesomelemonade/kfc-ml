import numpy as np

KING = 1
QUEEN = 2
ROOK = 3
BISHOP = 4
KNIGHT = 5
PAWN = 6

MATERIALS = {
    KING: 0,
    QUEEN: 9,
    ROOK: 5,
    BISHOP: 3,
    KNIGHT: 3,
    PAWN: 1
}

PIECE_TYPES = [KING, QUEEN, ROOK, BISHOP, KNIGHT, PAWN]

"""
Piece info:
- Type
- Color
- Position
- Existence
- Cooldown
"""

class Piece (object) :

    def __init__(self, type, color, position) :
        self.type = type
        self.color = color
        self.position = position
        self.is_alive = True
        self.cooldown = 0

    def mat (self) :
        return MATERIALS[self.type]

class Board (object) :

    def __init__(self) :
        self.pieces = {
            KING :
                [
                    Piece(KING, 0, (0, 4)),
                    Piece(KING, 1, (7, 4))
                ],
            QUEEN :
                [
                    Piece(QUEEN, 0, (0, 3)),
                    Piece(QUEEN, 1, (7, 3))
                ],
            ROOK :
                [
                    Piece(ROOK, 0, (0, 0)),
                    Piece(ROOK, 0, (0, 7)),
                    Piece(ROOK, 1, (7, 0)),
                    Piece(ROOK, 1, (7, 7))
                ],
            BISHOP :
                [
                    Piece(BISHOP, 0, (0, 2)),
                    Piece(BISHOP, 0, (0, 5)),
                    Piece(BISHOP, 1, (7, 2)),
                    Piece(BISHOP, 1, (7, 5))
                ],
            KNIGHT :
                [
                    Piece(KNIGHT, 0, (0, 1)),
                    Piece(KNIGHT, 0, (0, 6)),
                    Piece(KNIGHT, 1, (7, 1)),
                    Piece(KNIGHT, 1, (7, 6))
                ],
            PAWN :
                [
                    Piece(PAWN, 0, (1, 0)),
                    Piece(PAWN, 0, (1, 1)),
                    Piece(PAWN, 0, (1, 2)),
                    Piece(PAWN, 0, (1, 3)),
                    Piece(PAWN, 0, (1, 4)),
                    Piece(PAWN, 0, (1, 5)),
                    Piece(PAWN, 0, (1, 6)),
                    Piece(PAWN, 0, (1, 7)),
                    Piece(PAWN, 1, (6, 0)),
                    Piece(PAWN, 1, (6, 1)),
                    Piece(PAWN, 1, (6, 2)),
                    Piece(PAWN, 1, (6, 3)),
                    Piece(PAWN, 1, (6, 4)),
                    Piece(PAWN, 1, (6, 5)),
                    Piece(PAWN, 1, (6, 6)),
                    Piece(PAWN, 1, (6, 7))
                ]
        }

    def to_array (self) :
        # piece existence, piece pos1, piece pos2, piece cd
        pieces = np.zeros((16, 4))

        # piece count (in order type, no king)
        piece_counts = np.zeros((10))

        for i, piece in enumerate(self.pieces[KING]) :
            pieces[i] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])

        for i, piece in enumerate(self.pieces[QUEEN]) :
            pieces[i + 2] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])

        for i, piece in enumerate(self.pieces[ROOK]) :
            pieces[i + 4] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])

        for i, piece in enumerate(self.pieces[BISHOP]) :
            pieces[i + 8] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])

        for i, piece in enumerate(self.pieces[KNIGHT]) :
            pieces[i + 12] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])

        for i, piece in enumerate(self.pieces[PAWN]) :
            pieces[i + 16] = np.array([piece.is_alive, piece.position[0], piece.position[1], piece.cooldown])


        piece_types = PIECE_TYPES[1:]
        for piece_type in piece_types :
            piece_counts[0] = len([p for p in self.pieces[piece_type] if p.is_alive])