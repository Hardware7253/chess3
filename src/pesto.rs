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

use crate::bitboard_manipulation;
use crate::generic_math;
use crate::board_representation;
use crate::board_representation::{Board, TEAM_MATERIAL_VALUE, PieceColor};


// Converts bitboatd bit to pesto table index
fn convert_bit_to_index(bit: u8) -> usize {
    let (column, row) = bitboard_manipulation::get_piece_coordinates(bit);
    return ((column - 7).abs() + (row * 8)) as usize
}

// The pesto tables only work from the perspective of the white team
// (when using the index straight out of convert_bit_to_index)
//
// This function inverts the index so the tables can be used properly from the
// black teams perspective
fn invert_index(index: usize) -> usize {
    (index as i8 - 63).abs() as usize
}

// Returns a value from 0.0 to 1.0 (generally in this range, but no clamp is applied to enforce this)
// This value describes how much the board alligns with the piece square tables
pub fn get_table_value(board: &Board) -> f32 {
    let (current_material_value, friendly_baord, invert_indices) = match board.piece_to_move {
        PieceColor::Black => (board.black_material, board.black_board, true),
        PieceColor::White => (board.white_material, board.white_board, false),
    };

    // 1.0 for midgame, 0.0 for endgame
    let mg_weight = generic_math::f32_scale(current_material_value as f32, 0.0, TEAM_MATERIAL_VALUE as f32);
    let mut total_mg: f32 = 0.0;
    let mut total_eg: f32 = 0.0;
    for bit in 0..64 {
        let piece_id = board_representation::read_piece_id(&friendly_baord, bit);

        if piece_id == 0 {
            continue;
        }

        // Get index and invert for black team if neccasary
        let index = convert_bit_to_index(bit);
        let index = if invert_indices {
            invert_index(index)
        } else {
            index
        };

        total_mg += MIDGAME_TABLES[piece_id][index] as f32;
        total_eg += ENDGAME_TABLES[piece_id][index] as f32;
    }

    
    let total = total_mg * mg_weight + total_eg * (1.0 - mg_weight);
    generic_math::f32_scale(total, -300.0, 300.0)
}

// https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
// Values clamped to fit within 8 bit signed integer
// In order of initial arrays, doesn't match this engines bitboard layout
const MIDGAME_TABLES: [[i8;64]; 7] = [
    PLACEHOLDER_TABLE,
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE,
];

const ENDGAME_TABLES: [[i8;64]; 7] = [
    PLACEHOLDER_TABLE,
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE,
];

const PLACEHOLDER_TABLE: [i8; 64] = [0;64];

const MG_PAWN_TABLE: [i8; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
   98, 127,  61,  95,  68, 126, 34, -11,
   -6,   7,  26,  31,  65,  56, 25, -20,
  -14,  13,   6,  21,  23,  12, 17, -23,
  -27,  -2,  -5,  12,  17,   6, 10, -25,
  -26,  -4,  -4, -10,   3,   3, 33, -12,
  -35,  -1, -20, -23, -15,  24, 38, -22,
    0,   0,   0,   0,   0,   0,  0,   0,
];

const EG_PAWN_TABLE: [i8; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
  127, 127, 127, 127, 127, 127, 127, 127,
   94, 100,  85,  67,  56,  53,  82,  84,
   32,  24,  13,   5,  -2,   4,  17,  17,
   13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    4,   7,  -6,   1,   0,  -5,  -1,  -8,
   13,   8,   8,  10,  13,   0,   2,  -7,
    0,   0,   0,   0,   0,   0,   0,   0,
];

const MG_KNIGHT_TABLE: [i8; 64] = [
  -128, -89, -34, -49,  61, -97, -15, -107,
   -73, -41,  72,  36,  23,  62,   7,  -17,
   -47,  60,  37,  65,  84, 127,  73,   44,
    -9,  17,  19,  53,  37,  69,  18,   22,
   -13,   4,  16,  13,  28,  19,  21,   -8,
   -23,  -9,  12,  10,  19,  17,  25,  -16,
   -29, -53, -12,  -3,  -1,  18, -14,  -19,
  -105, -21, -58, -33, -17, -28, -19,  -23,
];

const EG_KNIGHT_TABLE: [i8; 64] = [
  -58, -38, -13, -28, -31, -27, -63, -99,
  -25,  -8, -25,  -2,  -9, -25, -24, -52,
  -24, -20,  10,   9,  -1,  -9, -19, -41,
  -17,   3,  22,  22,  22,  11,   8, -18,
  -18,  -6,  16,  25,  16,  17,   4, -18,
  -23,  -3,  -1,  15,  10,  -3, -20, -22,
  -42, -20, -10,  -5,  -2, -20, -23, -44,
  -29, -51, -23, -15, -22, -18, -50, -64,
];

const MG_BISHOP_TABLE: [i8; 64] = [
  -29,   4, -82, -37, -25, -42,   7,  -8,
  -26,  16, -18, -13,  30,  59,  18, -47,
  -16,  37,  43,  40,  35,  50,  37,  -2,
   -4,   5,  19,  50,  37,  37,   7,  -2,
   -6,  13,  13,  26,  34,  12,  10,   4,
    0,  15,  15,  15,  14,  27,  18,  10,
    4,  15,  16,   0,   7,  21,  33,   1,
  -33,  -3, -14, -21, -13, -12, -39, -21,
];

const EG_BISHOP_TABLE: [i8; 64] = [
  -14, -21, -11,  -8, -7,  -9, -17, -24,
   -8,  -4,   7, -12, -3, -13,  -4, -14,
    2,  -8,   0,  -1, -2,   6,   0,   4,
   -3,   9,  12,   9, 14,  10,   3,   2,
   -6,   3,  13,  19,  7,  10,  -3,  -9,
  -12,  -3,   8,  10, 13,   3,  -7, -15,
  -14, -18,  -7,  -1,  4,  -9, -15, -27,
  -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

const MG_ROOK_TABLE: [i8; 64] = [
   32,  42,  32,  51, 63,  9,  31,  43,
   27,  32,  58,  62, 80, 67,  26,  44,
   -5,  19,  26,  36, 17, 45,  61,  16,
  -24, -11,   7,  26, 24, 35,  -8, -20,
  -36, -26, -12,  -1,  9, -7,   6, -23,
  -45, -25, -16, -17,  3,  0,  -5, -33,
  -44, -16, -20,  -9, -1, 11,  -6, -71,
  -19, -13,   1,  17, 16,  7, -37, -26,
];

const EG_ROOK_TABLE: [i8; 64] = [
  13, 10, 18, 15, 12,  12,   8,   5,
  11, 13, 13, 11, -3,   3,   8,   3,
   7,  7,  7,  5,  4,  -3,  -5,  -3,
   4,  3, 13,  1,  2,   1,  -1,   2,
   3,  5,  8,  4, -5,  -6,  -8, -11,
  -4,  0, -5, -1, -7, -12,  -8, -16,
  -6, -6,  0,  2, -9,  -9, -11,  -3,
  -9,  2,  3, -1, -5, -13,   4, -20,
];

const MG_QUEEN_TABLE: [i8; 64] = [
  -28,   0,  29,  12,  59,  44,  43,  45,
  -24, -39,  -5,   1, -16,  57,  28,  54,
  -13, -17,   7,   8,  29,  56,  47,  57,
  -27, -27, -16, -16,  -1,  17,  -2,   1,
   -9, -26,  -9, -10,  -2,  -4,   3,  -3,
  -14,   2, -11,  -2,  -5,   2,  14,   5,
  -35,  -8,  11,   2,   8,  15,  -3,   1,
   -1, -18,  -9,  10, -15, -25, -31, -50,
];

const EG_QUEEN_TABLE: [i8; 64] = [
   -9,  22,  22,  27,  27,  19,  10,  20,
  -17,  20,  32,  41,  58,  25,  30,   0,
  -20,   6,   9,  49,  47,  35,  19,   9,
    3,  22,  24,  45,  57,  40,  57,  36,
  -18,  28,  19,  47,  31,  34,  39,  23,
  -16, -27,  15,   6,   9,  17,  10,   5,
  -22, -23, -30, -16, -16, -23, -36, -32,
  -33, -28, -22, -43,  -5, -32, -20, -41,
];

const MG_KING_TABLE: [i8; 64] = [
  -65,  23,  16, -15, -56, -34,   2,  13,
   29,  -1, -20,  -7,  -8,  -4, -38, -29,
   -9,  24,   2, -16, -20,   6,  22, -22,
  -17, -20, -12, -27, -30, -25, -14, -36,
  -49,  -1, -27, -39, -46, -44, -33, -51,
  -14, -14, -22, -46, -44, -30, -15, -27,
    1,   7,  -8, -64, -43, -16,   9,   8,
  -15,  36,  12, -54,   8, -28,  24,  14,
];

const EG_KING_TABLE: [i8; 64] = [
  -74, -35, -18, -18, -11,  15,   4, -17,
  -12,  17,  14,  17,  17,  38,  23,  11,
   10,  17,  23,  15,  20,  45,  44,  13,
   -8,  22,  24,  27,  26,  33,  26,   3,
  -18,  -4,  21,  24,  27,  23,   9, -11,
  -19,  -3,  11,  21,  23,  16,   7,  -9,
  -27, -11,   4,  13,  14,   4,  -5, -17,
  -53, -34, -21, -11, -28, -14, -24, -43
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::fen::read_fen;

    #[test]
    fn test_convert_bit_to_index() {
        assert_eq!(convert_bit_to_index(8), 15);
    }

    #[test]
    fn test_invert_index() {
        assert_eq!(invert_index(56), 7);
        assert_eq!(invert_index(14), 49);
    }

    #[test]
    fn test_get_table_value() {
        let board1 = read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let board2 = read_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
        assert!(get_table_value(&board2) > get_table_value(&board1));
    }
}
