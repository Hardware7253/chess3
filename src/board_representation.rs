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
use crate::pieces;


// Board is defined as the white team being at the bottom of the board, and the black team at the top (at starting position)
// The board never flips orientation

// Bitboards for chess starting position
pub const STARTING_WHITE_BOARD: [u64; 3] = [3818771009033469952, 7926335344172072960, 11024811887802974208];
pub const STARTING_BLACK_BOARD: [u64; 3] = [65332, 110, 153];

// Material value of a team at the start of the game
// Should lign up with material values provided in pieces.rs
pub const TEAM_MATERIAL_VALUE: i8 = 39;

#[derive(Debug, PartialEq, Clone)]
pub struct Board {

    // Each team has an array containing 3 bitboards, the bit represents a pieces position and is common for all 3 bitboards
    // A 3 bit number is stored by constructing considering correspondsing bits from the 3 bitboards as a 3 bit number
    // These 3 bit numbers are used to identify pieces
    // The 0th bitboard in the array is considered the 0th bit for the piece IDs
    pub white_board: [u64; 3],
    pub black_board: [u64; 3],

    pub white_king_bit: u8,
    pub black_king_bit: u8,

    pub piece_to_move: PieceColor,
    pub en_passant_target_bit: Option<u8>,
    pub castling_availability: CastlingAvailability,

    pub white_material: i8,
    pub black_material: i8,

    pub halfmove_clock: i16, // Number of half moves since capture or pawn advance
    pub fullmove_number: i16, // Incremented after blacks turn
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CastlingAvailability {
    pub w_ks: bool,
    pub w_qs: bool,
    
    pub b_ks: bool,
    pub b_qs: bool,
}

// Boards from the perspective of the team whos turn it is to move
pub struct PerspectiveBoards<'a> {
    pub friendly_board: &'a [u64; 3],
    pub enemy_board: &'a [u64; 3],
    pub friendly_starting_board: &'a [u64; 3],
    pub friendly_piece_information: [pieces::PieceInformation; 7],
    pub enemy_team_color: PieceColor,
}

impl Board {

    // Create new board with the starting position
    pub fn new() -> Self {
        Board {
            white_board: STARTING_WHITE_BOARD,
            black_board: STARTING_BLACK_BOARD,
            white_king_bit: 59,
            black_king_bit: 3,
            piece_to_move: PieceColor::White,
            en_passant_target_bit: None,
            castling_availability: CastlingAvailability::new(true),
            white_material: TEAM_MATERIAL_VALUE,
            black_material: TEAM_MATERIAL_VALUE,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    // Create new empty board
    pub fn empty() -> Self {
        Board {
            white_board: [0; 3],
            black_board: [0; 3],
            white_king_bit: 0,
            black_king_bit: 0,
            piece_to_move: PieceColor::White,
            en_passant_target_bit: None,
            castling_availability: CastlingAvailability::new(false),
            white_material: 0,
            black_material: 0,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl CastlingAvailability {
    fn new(common_state: bool) -> Self {
        CastlingAvailability {
            w_ks: common_state,
            w_qs: common_state,
            b_ks: common_state,
            b_qs: common_state
        }
    }
}

impl<'a> PerspectiveBoards<'a> {

    // Generate perspective boards for piece to move
    pub fn gen(board: &'a Board, from_persecpective: PieceColor) -> Self {

        match from_persecpective {
            PieceColor::Black => {
                PerspectiveBoards {
                    friendly_board: &board.black_board,
                    enemy_board: &board.white_board,
                    friendly_starting_board: &STARTING_BLACK_BOARD,
                    friendly_piece_information: pieces::BLACK_PIECE_INFORMATION,
                    enemy_team_color: PieceColor::White,
                }
            }

            PieceColor::White => {
                PerspectiveBoards {
                    friendly_board: &board.white_board,
                    enemy_board: &board.black_board,
                    friendly_starting_board: &STARTING_WHITE_BOARD,
                    friendly_piece_information: pieces::WHITE_PIECE_INFORMATION,
                    enemy_team_color: PieceColor::Black,
                }
            }
        } 
    }

    // Generates friendly and enemy bitboards
    // These bitboards contain no information about the type of piece, just the positions
    pub fn gen_bitboards(&self) -> (u64, u64) {
        let friendly_bitboard = self.friendly_board[0] | self.friendly_board[1] | self.friendly_board [2];
        let enemy_bitboard = self.enemy_board[0] | self.enemy_board[1] | self.enemy_board [2];

        (friendly_bitboard, enemy_bitboard)
    }
}

// Reads a piece id from a team board given a bit
// See board_representation.rs for information about how the team boards work
pub fn read_piece_id(team_board: &[u64; 3], piece_bit: u8) -> usize {
    let mut output = 0;

    for i in 0..3 {
        if bitboard_manipulation::bit_on(team_board[i], piece_bit) {
            output |= 1 << i;
        }
    }

    output
}

// Insert piece in white or black team board
pub fn insert_piece(piece_bit: u8, piece_id: usize, half_board: &mut [u64; 3]) {
    for i in 0..3 {
        if bitboard_manipulation::bit_on(piece_id, i as u8) {
            half_board[i] |= 1 << piece_bit as u64
        }
    }
}

// Removes a piece from a half board 
pub fn remove_piece(piece_bit: u8, half_board: &mut [u64; 3]) {
    for i in 0..3 {
        if bitboard_manipulation::bit_on(half_board[i], piece_bit) {
            half_board[i] ^= 1 << piece_bit as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_piece_id() {
        assert_eq!(read_piece_id(&[0, 1 << 7, 1 << 7], 7), 6)
    }

    #[test]
    fn test_remove_piece() {
        let mut half_board = [3, 0, 3];
        remove_piece(1, &mut half_board);

        assert_eq!(half_board, [1, 0, 1]);
    }

    #[test]
    fn test_insert_piece() {
        let mut half_board = [0, 3, 0];
        insert_piece(1, 1, &mut half_board);

        assert_eq!(half_board, [2, 3, 0]);
    }
}


// Module for converting to and from fen notation
pub mod fen {
    use crate::pieces::{BLACK_PIECE_TYPES, WHITE_PIECE_TYPES, KING_ID};
    use core::str::Chars;
    use super::*;

    // Create a board from a fen string
    // This implementation of fen is alsmost completely standard
    // Except the en-passant target square field is replaced by an en-passant target bit
    // Which follows the bitboard / bit coordinates (view top of file)
    pub fn read_fen(fen_string: &str) -> Board {

        let mut bit: u8 = 7;
        let mut row = 0;
        let mut board = Board::empty();

        let mut space_counter = 0;

        let mut last_character_space = false;
        for (i, c) in fen_string.chars().enumerate() {

            // The '/' seperator is useless to this implementation
            if c == '/' {
                continue;
            }

            if c == ' ' {
                space_counter += 1;
                last_character_space = true;
                continue;
            }

            // If a space hasn't appeared in the FEN string then we're looking at board layout information
            if space_counter == 0 {

                set_king_bits(bit, c, &mut board);

                // If the the character is a number, skip that many squares in the bitbaord
                let skip_numer: u8 = char_to_num(c).unwrap_or(1);
            
                // Insert black/white piece into their respective board arrays
                // and add pieces material value to appropriate variables
                if let Some(piece_id) = find_key_in_array(c, BLACK_PIECE_TYPES) {
                    insert_piece(bit, piece_id, &mut board.black_board);
                    board.black_material += pieces::BLACK_PIECE_INFORMATION[piece_id].piece_value;
                } else if let Some(piece_id) = find_key_in_array(c, WHITE_PIECE_TYPES) {
                    insert_piece(bit, piece_id, &mut board.white_board);
                    board.white_material += pieces::WHITE_PIECE_INFORMATION[piece_id].piece_value;
                }

                // For traversing bitboard
                for _ in 0..skip_numer {
                    if bit % 8 == 0 {
                        row += 1;
                        bit = row * 8 + 7;
                    } else {
                        bit -= 1
                    }
                }
            } 
            
            // If there is one space then we're looking at the team to move
            else if space_counter == 1 {

                // Set piece to move
                if c == 'w' {
                   board.piece_to_move = PieceColor::White
                } else {
                    board.piece_to_move = PieceColor::Black
                }
            }
            
            // If there are two spaces then we're looking at the castling availability
            else if space_counter == 2 {
                match c {
                    'K' => board.castling_availability.w_ks = true,
                    'Q' => board.castling_availability.w_qs = true,
                    'k' => board.castling_availability.b_ks = true,
                    'q' => board.castling_availability.b_qs = true,
                    _ => ()
                }
            }

            // If there are three spaces then we're looking at the en passant target bit
            // Not really FEN notation because something like E5 would normally be here
            // Instead we use a bit e.g. 27 = E5
            else if space_counter == 3 && last_character_space {
                if char_to_num(c) != None {
                    board.en_passant_target_bit = Some(collect_nums(fen_string.chars(), i) as u8);
                }
                last_character_space = false;
            } 
            
            // If there are four or five spaces then we're looking at the half and fullmove clocks
            else if last_character_space {

                // Half and fullmove clocks can have multiple digit numbers, so use collect_nums()
                let num = collect_nums(fen_string.chars(), i) as i16;

                match space_counter {
                    4 => board.halfmove_clock = num, 
                    5 => board.fullmove_number = num,
                    _ => ()
                }
                last_character_space = false
            }

        }

        board
    }

    // Sets king bits in the board from the current fen character
    fn set_king_bits(current_bit: u8, fen_char: char, board: &mut Board) {
        if fen_char == WHITE_PIECE_TYPES[KING_ID] {
            board.white_king_bit = current_bit;
        }

        else if fen_char == BLACK_PIECE_TYPES[KING_ID] {
            board.black_king_bit = current_bit;
        }
    }

    // Converts character to number
    fn char_to_num(c: char) -> Option<u8> {
        let c_num = c as u8;

        if c_num >= 48 && c_num <= 57 {
            return Some(c_num - 48)
        }
        None
    }

    // Collect nums in character iterator to a number
    // start_index is the index to start in the iterator
    // E.g. collect_nums(Chars(['1', '2', '3', '4']), 0) -> 1234
    // E.g. collect_nums(Chars(['1', '2', '3', '4', ' ', '1']), 0) -> 1234
    fn collect_nums(characters: Chars<'_>, start_index: usize) -> u32 {
        let mut output = 0;
        for c in characters.skip(start_index) {
            if let Some(num) = char_to_num(c) {
                output = output * 10 + num as u32
            } else {
                break;
            }
        }
        output
    }


    // Returns index of a key in an array
    fn find_key_in_array<T: Copy + PartialOrd, const COUNT: usize>(key: T, arr: [T; COUNT]) -> Option<usize> {
        arr.iter().position(|&s| s == key)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_collect_nums() {
            assert_eq!(collect_nums("1234".chars(), 0), 1234);
            assert_eq!(collect_nums("1234 1".chars(), 1), 234);
        }

        #[test]
        fn test_read_fen() {

            // Test reading fen to black board, setting move clocks, and en passant target bit
            let result = read_fen("k7/8/8/8/8/8/8/8 w HAha 31 5 20");

            let mut expected = Board::empty();
            expected.black_board = [0, 1 << 7, 1 << 7];
            expected.halfmove_clock = 5;
            expected.fullmove_number = 20;
            expected.black_king_bit = 7;
            expected.en_passant_target_bit = Some(31);

            assert_eq!(result, expected);


            // Test reading fen for multiple teams
            let result = read_fen("7p/8/8/2B5/8/5P2/8/8 b Kq - 0 1");

            let mut expected = Board::empty();
            expected.castling_availability = CastlingAvailability {
                w_ks: true,
                w_qs: false,
                b_ks: false,
                b_qs: true,
            };

            expected.white_board = [1 << 29 | 1 << 42, 1 << 29, 0];
            expected.black_board = [1, 0, 0];
            expected.piece_to_move = PieceColor::Black;
            expected.white_material = 4;
            expected.black_material = 1;

            assert_eq!(result, expected);
        }

    }
} 
