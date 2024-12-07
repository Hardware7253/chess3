// Contains direction bitboard constants and information about how to manipulate them

// Chesboard indices (corresponds to bits in the bitboards)
//
//      C7 C6 C5 C4 C3 C2 C1 C0
//-----------------------------
// R0 | 07 06 05 04 03 02 01 00
// R1 | 15 14 13 12 11 10 09 08
// R2 | 23 22 21 20 19 18 17 16
// R3 | 31 30 29 28 27 26 25 24
// R4 | 39 38 37 36 35 34 33 32
// R5 | 47 46 45 44 43 42 41 40
// R6 | 55 54 53 52 51 50 49 48
// R7 | 63 62 61 60 59 58 57 56

// Contains a direction bitboard and information about how to shift the bitboard to the location of a piece
#[derive(Debug, PartialEq)]
pub struct DirectionBitboard {
    pub bitboard: u64,
    pub origin_bit: u8, // No shifts need to be done if a piece is at this bit
    pub shift_type: ShiftType,
}

// To use a direction bitboard as potential moves for a piece
// the direction bitboard needs to be moved to the correct place
//
// This is done with bitwise shifts
// This enum describes what type of shift is necessary to move a direction bitboard to the correct location
#[derive(Debug, PartialEq)]
pub enum ShiftType {
    Standard,   // Shift up/down row
    Byte,       // Shift up/down column
    Diagonal,   // Special shift type for diagonal bitboards
    Both,       // Shift using byte and standard methods
}

pub const ALL_CAPTURE_BITBOARDS: [&DirectionBitboard; 6] = [&DIAGONAL_RIGHT, &DIAGONAL_LEFT, &VERTICAL_LINE, &HORIZONTAL_LINE, &KNIGHT_MOVES, &KING_MOVES];

// Basic move directions -----------------------------------------------------------------------------------

// A diagonal line which goes from the bottom left of the board to the top right
pub const DIAGONAL_RIGHT: DirectionBitboard = DirectionBitboard {
    bitboard: 0b1000000001000000001000000001000000001000000001000000001000000001,
    origin_bit: 36,
    shift_type: ShiftType::Diagonal,
};

// A diagonal line which goes from the top left of the board to the bottom right
pub const DIAGONAL_LEFT: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000100000010000001000000100000010000001000000100000010000000,
    origin_bit: 35,
    shift_type: ShiftType::Diagonal,
};

// Vertical line in colum 7
pub const VERTICAL_LINE: DirectionBitboard = DirectionBitboard {
    bitboard: 0b1000000010000000100000001000000010000000100000001000000010000000,
    origin_bit: 31,
    shift_type: ShiftType::Byte,
};

// Horizontal line in row 7
pub const HORIZONTAL_LINE: DirectionBitboard = DirectionBitboard {
    bitboard: 0b1111111100000000000000000000000000000000000000000000000000000000,
    origin_bit: 60,
    shift_type: ShiftType::Standard,
};

// Basic move directions -----------------------------------------------------------------------------------

// Piece move directions -----------------------------------------------------------------------------------

pub const KNIGHT_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000101000010001000000000001000100001010000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};

pub const KING_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000001110000010100000111000000000000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};


// Pawn moves
pub const WHITE_PAWN_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000000000000000000000010000000000000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};

pub const WHITE_PAWN_CAPTURE_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000000000000000000000101000000000000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};

pub const BLACK_PAWN_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000000100000000000000000000000000000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};

pub const BLACK_PAWN_CAPTURE_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000001010000000000000000000000000000000000000000000,
    origin_bit: 36,
    shift_type: ShiftType::Both,
};

// Implementation for finding en_passant_target_bit based on these directions bitboards
// is in a fn in move_generation.rs
pub const WHITE_PAWN_DOUBLE_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000000000010000000100000000000000000000000000000000,
    origin_bit: 48,
    shift_type: ShiftType::Both,
};

pub const BLACK_PAWN_DOUBLE_MOVES: DirectionBitboard = DirectionBitboard {
    bitboard: 0b0000000000000000000000000000000000000001000000010000000000000000,
    origin_bit: 8,
    shift_type: ShiftType::Both,
};
// Piece move directions -----------------------------------------------------------------------------------