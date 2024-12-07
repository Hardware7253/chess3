// This file is for finding if the king is in check

use crate::fixed_vecor::*;
use crate::board_representation;
use crate::board_representation::{Board, PieceColor, PerspectiveBoards};
use crate::direction_bitboards::ALL_CAPTURE_BITBOARDS;
use crate::bitboard_manipulation;
use crate::move_generation::generate_moves;

const FIXED_VECTOR_PLACEHOLDER_VALUE: u8 = 255;
pub const MAX_CHECKING_PIECES: usize = 16; // Maximum number of pieces that can potentially be putting the king in check

// Returns a vector of pieces which could potentially be putting the king in check
pub fn get_potential_checking_pieces(board: &Board, king_color: PieceColor) -> FixedVector<u8, MAX_CHECKING_PIECES> {
    let mut potential_checking_pieces_bitboard: u64 = 0;

    let (enemy_board, king_bit) = match king_color {
        PieceColor::Black => (&board.white_board, board.black_king_bit),
        PieceColor::White => (&board.black_board, board.white_king_bit),
    };

    let enemy_bitboard = enemy_board[0] | enemy_board[1] | enemy_board[2];

    let king_coordinates = bitboard_manipulation::get_piece_coordinates(king_bit);

    for direction_bitboard in ALL_CAPTURE_BITBOARDS {

        // Update direction bitboard so it is centered on the king
        let direction_bitboard = bitboard_manipulation::shift_direction_bitboard(king_bit, king_coordinates, direction_bitboard);

        // Any collisions are pieces which could be putting the king in check
        potential_checking_pieces_bitboard |= direction_bitboard & enemy_bitboard
    }

    
    //crate::bitboard_manipulation::debugging::print_bytes(potential_checking_pieces_bitboard);
    
    bitboard_manipulation::bits_on(potential_checking_pieces_bitboard, FIXED_VECTOR_PLACEHOLDER_VALUE)
}

// Returns true if the king is in check
pub fn is_king_in_check(
    board: &Board,
    king_color: PieceColor,
    potential_checking_pieces: &FixedVector<u8, MAX_CHECKING_PIECES>
) -> bool {

    let (enemy_color, king_bit) = match king_color {
        PieceColor::Black => (PieceColor::White, board.black_king_bit),
        PieceColor::White => (PieceColor::Black, board.white_king_bit),
    };
    
    // Go through all pieces which could be putting the king in check and generate their moves
    // Use the moves to see if the pieces can capture the king
    // If any of the potential pieces can capture the king then the king is in check
    for i in 0..potential_checking_pieces.len() {
        let potential_checking_piece_bit = potential_checking_pieces.internal_array[i];

        if potential_checking_piece_bit != FIXED_VECTOR_PLACEHOLDER_VALUE {
            let enemy_persepective_boards = PerspectiveBoards::gen(board, enemy_color);
            let enemy_piece_id = board_representation::read_piece_id(enemy_persepective_boards.friendly_board, potential_checking_piece_bit);
            let enemy_piece_moves = generate_moves(board, potential_checking_piece_bit, enemy_piece_id, enemy_color, &enemy_persepective_boards).0;

            if bitboard_manipulation::bit_on(enemy_piece_moves, king_bit) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::fen::read_fen;

    #[test]
    fn test_get_potential_checking_pieces() {
        let board = read_fen("rnbqkbnr/pppppppp/8/8/3K4/5p2/PPPPPPPP/RNBQ1BNR w kq - 0 1");
        let result = get_potential_checking_pieces(&board, PieceColor::White);
        let mut result_array = result.internal_array;
        result_array.sort();

        // Bits of pieces which might be puting the king in check
        let expected_array = [0, 4, 9, 12, 15, 42, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];

        assert_eq!(result_array, expected_array);
    }

    #[test]
    fn test_is_king_in_check() {

        // Test king not being in check
        let board = read_fen("rnbqkbnr/pppppppp/8/8/3K4/5p2/PPPPPPPP/RNBQ1BNR w kq - 0 1");
        let potential_checking_pieces = get_potential_checking_pieces(&board, PieceColor::White);

        let result = is_king_in_check(&board, PieceColor::White, &potential_checking_pieces);
        assert_eq!(result, false);



        // Test king being in check
        let board = read_fen("rnbqkbnr/pppppppp/8/8/1b6/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
        let potential_checking_pieces = get_potential_checking_pieces(&board, PieceColor::White);
        
        let result = is_king_in_check(&board, PieceColor::White, &potential_checking_pieces);
        assert_eq!(result, true);
        
    }
}