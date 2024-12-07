// Contains information about identifying pieces and the properties of pieces
// Basically nothing in this file should be touched unless changing implementation

// Indices of all arrays in this file correspond to a piece ID, so all arrays have the same order

use crate::direction_bitboards::*;

pub const KING_ID: usize = 6;
pub const PAWN_ID: usize = 1;

// Question mark used as a placeholder so the index of the character can be used as a piece id
// This is only used for decoding FEN strings
// Piece ID                                     1    2    3    4    5    6
pub const BLACK_PIECE_TYPES: [char; 7] = ['?', 'p', 'n', 'b', 'r', 'q', 'k'];
pub const WHITE_PIECE_TYPES: [char; 7] = ['?', 'P', 'N', 'B', 'R', 'Q', 'K'];

// More unique immplenetations than can be defined in this struct will have to be hardcoded
#[derive(Debug)]
pub struct PieceInformation {
    pub piece_value: i8, // Material value of piece
    pub is_sliding: bool, // True if the piece can slide (bishop, rook, queen)

    pub move_directions: usize, // How many direction bitboards the piece has, corresponds to elements in the $direction_bitboards array
    pub direction_bitboards: [Option<DirectionBitboard>; 4], // Direction bitboard array

    // Pawns can only capture pieces on the capture bitboard, and can only move on their direction bitboard
    // Check implementation in move_generation.rs for specifics
    pub pawn_capture_bitboard: Option<DirectionBitboard>,

    // Pawn direction bitboard which is only valid while in it's starting position
    // Check implementation in move_generation.rs for specifics
    pub pawn_double_move_bitboard: Option<DirectionBitboard>,
}

// Define piece information for pieces that can have it defined generically (where direction doesn't change based on team)
const GENERIC_KNIGHT: PieceInformation = PieceInformation {
    piece_value: 3,
    is_sliding: false,
    move_directions: 1,
    direction_bitboards: [Some(KING_MOVES), None, None, None],
    pawn_capture_bitboard: None,
    pawn_double_move_bitboard: None,
};

const GENERIC_BISHOP: PieceInformation = PieceInformation {
    piece_value: 3,
    is_sliding: true,
    move_directions: 2,
    direction_bitboards: [Some(DIAGONAL_LEFT), Some(DIAGONAL_RIGHT), None, None],
    pawn_capture_bitboard: None,
    pawn_double_move_bitboard: None,
};

const GENERIC_ROOK: PieceInformation = PieceInformation {
    piece_value: 5,
    is_sliding: true,
    move_directions: 2,
    direction_bitboards: [Some(HORIZONTAL_LINE), Some(VERTICAL_LINE), None, None],
    pawn_capture_bitboard: None,
    pawn_double_move_bitboard: None,
};

const GENERIC_QUEEN: PieceInformation = PieceInformation {
    piece_value: 9,
    is_sliding: true,
    move_directions: 4,
    direction_bitboards: [Some(DIAGONAL_LEFT), Some(DIAGONAL_RIGHT), Some(HORIZONTAL_LINE), Some(VERTICAL_LINE)],
    pawn_capture_bitboard: None,
    pawn_double_move_bitboard: None,
};

const GENERIC_KING: PieceInformation = PieceInformation {
    piece_value: 0,
    is_sliding: false,
    move_directions: 1,
    direction_bitboards: [Some(KING_MOVES), None, None, None],
    pawn_capture_bitboard: None,
    pawn_double_move_bitboard: None,
};

// ALL PIECE INFORMATION ARRAYS MUST HAVE THE SAME ORDER
// THE PIECE ID IS USED TO INDEX
// The piece square tables in pesto.rs also follow the same order
pub const BLACK_PIECE_INFORMATION: [PieceInformation; 7] = [

    // Placeholder (index 0 can't be occupied because it's used to denote an empty square)
    GENERIC_KING,

    // Pawn
    PieceInformation {
        piece_value: 1,
        is_sliding: false,
        move_directions: 1,
        direction_bitboards: [Some(BLACK_PAWN_MOVES), None, None, None],
        pawn_capture_bitboard: Some(BLACK_PAWN_CAPTURE_MOVES),
        pawn_double_move_bitboard: Some(BLACK_PAWN_DOUBLE_MOVES),
    },

    GENERIC_KNIGHT,
    GENERIC_BISHOP,
    GENERIC_ROOK,
    GENERIC_QUEEN,
    GENERIC_KING
];

pub const WHITE_PIECE_INFORMATION: [PieceInformation; 7] = [

    // Placeholder (index 0 can't be occupied because it's used to denote an empty square)
    GENERIC_KING,

    // Pawn
    PieceInformation {
        piece_value: 1,
        is_sliding: false,
        move_directions: 1,
        direction_bitboards: [Some(WHITE_PAWN_MOVES), None, None, None],
        pawn_capture_bitboard: Some(WHITE_PAWN_CAPTURE_MOVES),
        pawn_double_move_bitboard: Some(WHITE_PAWN_DOUBLE_MOVES),
    },

    GENERIC_KNIGHT,
    GENERIC_BISHOP,
    GENERIC_ROOK,
    GENERIC_QUEEN,
    GENERIC_KING
];