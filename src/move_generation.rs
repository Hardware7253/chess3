// This file is for generating poitential move bitboards for a piece taking into account enemy and friendly piece positions

use crate::board_representation;
use crate::board_representation::{Board, PieceColor, PerspectiveBoards};
use crate::bitboard_manipulation::*;
use crate::direction_bitboards::DirectionBitboard;
use crate::en_passant::get_en_passant_capture;

// // Get friendly, enemy, and piece information corresponding to the given PieceColor
// let (friendly_board, friendly_starting_board, enemy_board, piece_information) = match piece_color {
//     PieceColor::Black => (&board.black_board, &STARTING_BLACK_BOARD, &board.white_board, &BLACK_PIECE_INFORMATION),
//     PieceColor::White => (&board.white_board, &STARTING_WHITE_BOARD, &board.black_board, &WHITE_PIECE_INFORMATION)
// };

// Generates possible moves for a piece in the form of a bitboard
// Only considers collisions with enemy and friendly pieces, doesn't care about king being in check
// Assumes there is a piece at the position provided
//
// This function does not care about whose turn to move it is
// Because sometimes it is useful to generate enemy moves without updating piece_to_move within the Board struct
// This might happen when finding pieces that are, or could be putting the king in check
//
// Returns move bitboard,
// en passant target bit (if a pawn double move is part of the move bitboard),
// and en passant capture and move bits (if a pawn can perform an en-passant on an enemy pawn)
pub fn generate_moves(
    board: &Board,
    piece_bit: u8,
    piece_id: usize,
    piece_color: PieceColor,
    perspective_boards: &PerspectiveBoards,
) -> (u64, Option<u8>, Option<(u8, u8)>) {

    let mut output_move_bitboard: u64 = 0;

    // Set if a double move is part of a pawns moveset generated in this function
    let mut en_passant_target_bit: Option<u8> = None;

    let piece_coordinates = get_piece_coordinates(piece_bit);
    let piece_information = &perspective_boards.friendly_piece_information[piece_id];

    // Get friendly and enemy position bitboards
    let (friendly_bitboard, enemy_bitboard) = perspective_boards.gen_bitboards();

    // Use pawn_double_move_bitboard if the piece has one
    if piece_information.pawn_double_move_bitboard != None {

        // This bitboard only works if the piece is in it's starting position
        if piece_id == board_representation::read_piece_id(perspective_boards.friendly_starting_board, piece_bit) {
            let direction_bitboard = piece_information.pawn_double_move_bitboard.as_ref().unwrap();

            // Calculate move bitboards
            let (move_bitboard, _f, _e, intercepted_mbb) = calc_move_bitboards(piece_bit, piece_coordinates, direction_bitboard, &friendly_bitboard, &enemy_bitboard);
            
            // Use the double move bitbord if there are no collisision with any other piece
            if intercepted_mbb == move_bitboard {
                output_move_bitboard |= move_bitboard;
                en_passant_target_bit = Some(calc_ep_target_bit(&move_bitboard, piece_color))
            }
        }
    }

    // Use all piece move directions for the output move bitboard
    for i in 0..piece_information.move_directions {
        let direction_bitboard = piece_information.direction_bitboards[i].as_ref().unwrap();

        // Calculate move bitboards
        let (move_bitboard, friendly_mbb_intercepts, _e, intercepted_mbb) = calc_move_bitboards(piece_bit, piece_coordinates, direction_bitboard, &friendly_bitboard, &enemy_bitboard);

        // If thie piece doesn't slide then the move bitboard doesn't have to be corrected for pieces intercepting the moving pieces path
        if !piece_information.is_sliding {

            
            if piece_information.pawn_capture_bitboard == None {
                // Allow the piece to move ontop of enemy pieces if it doesn't have a capture bitboard
                output_move_bitboard |= move_bitboard ^ friendly_mbb_intercepts;
            } else {

                // Add valid move bitboard to output (where the piece can move without intercepting any friendly or enmy pieces)
                output_move_bitboard |= intercepted_mbb;

                // Add pawn capture bitboard to output
                let capture_bitboard = shift_direction_bitboard(piece_bit, piece_coordinates, &piece_information.pawn_capture_bitboard.as_ref().unwrap());
                output_move_bitboard |= enemy_bitboard & capture_bitboard;
            }
            
        } else {
            
            // Fix the move bitboard so sliding pieces can't move on the other side of pieces blocking thier path
            let (fixed_bitboard, first_intersecting_bits) = fix_move_bitboard(piece_coordinates, &direction_bitboard.bitboard, &move_bitboard, &intercepted_mbb);
            output_move_bitboard |= fixed_bitboard; // At this stage the movement is blocked by any piece

            // Add enemey pieces which blocked the movement back into the output (so they can be moved ontop of to capture)
            let mut cutoff_bitboard = 0;
            if let Some(intersecting_bit) = first_intersecting_bits.0 {
                cutoff_bitboard |= 1 << intersecting_bit;
            }

            if let Some(intersecting_bit) = first_intersecting_bits.1 {
                cutoff_bitboard |= 1 << intersecting_bit;
            }

            output_move_bitboard |= enemy_bitboard & cutoff_bitboard;
        }
    }

    // Add en passant move bit to output move bitboard
    let en_passant_cap_bits = get_en_passant_capture(board, perspective_boards.friendly_board, perspective_boards.enemy_board, piece_bit);
    if en_passant_cap_bits != None {
        output_move_bitboard |= 1 << en_passant_cap_bits.unwrap().1;
    }

    (output_move_bitboard, en_passant_target_bit, en_passant_cap_bits)
}

// Calculate en-passant target bit given a pawns shifted double move bitboard and color
fn calc_ep_target_bit(move_bitboard: &u64, piece_color: PieceColor) -> u8 {
    match piece_color {
        PieceColor::White => move_bitboard.trailing_zeros() as u8,
        PieceColor::Black => 63 - move_bitboard.leading_zeros() as u8,
    }
}

// Calculates move bitboard and usefull intercept bitboards from a direction bitboard
fn calc_move_bitboards(piece_bit: u8, piece_coordinates: (i8, i8), direction_bitboard: &DirectionBitboard, friendly_bitboard: &u64, enemy_bitboard: &u64) -> (u64, u64, u64, u64){

    // Shift direction bitboard so it alligns with piece position
    let move_bitboard = shift_direction_bitboard(piece_bit, piece_coordinates, direction_bitboard);

    let friendly_mbb_intercepts = move_bitboard & friendly_bitboard; // Where the move bitboard is intercepted by friendly pieces
    let enemy_mbb_intercepts = move_bitboard & enemy_bitboard; // Where the move bitboard is intercepted by enemy pieces

    // Remove the intercept points from the original move bitboard
    let intercepted_mbb = move_bitboard ^ (friendly_mbb_intercepts | enemy_mbb_intercepts);

    (move_bitboard, friendly_mbb_intercepts, enemy_mbb_intercepts, intercepted_mbb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::PieceColor;

    // Gets information needed to run the function
    fn generate_moves_result(board: &Board, piece_bit: u8, for_team: PieceColor) -> (u64, Option<u8>, Option<(u8, u8)>) {
        let perspective_boards = &PerspectiveBoards::gen(&board, for_team);
        let piece_id = board_representation::read_piece_id(perspective_boards.friendly_board, piece_bit);
        generate_moves(&board, piece_bit, piece_id, for_team, perspective_boards)
    }

    #[test]
    fn test_generate_moves() {
        use crate::board_representation::fen::read_fen;

        // Test white queen movement
        let board = read_fen("rnbqkbnr/pppppppp/8/8/2Q5/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let expected_bitboard: u64 = 0b0000000000000000011100001101111101110000101010000010010000000000;
        assert_eq!(generate_moves_result(&board, 37, PieceColor::White), (expected_bitboard, None, None));

        // Test black pawn capturing and moving
        let board = read_fen("rnbqkbnr/ppp2pp1/7p/3pp3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
        let expected_bitboard: u64 = 0b0000000000000000000000000001100000000000000000000000000000000000;
        assert_eq!(generate_moves_result(&board, 27, PieceColor::Black), (expected_bitboard, None, None));

        // Test black pawn double move
        let board = read_fen("rnbqkbnr/ppp2pp1/7p/3pp3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
        let expected_bitboard: u64 = 0b0000000000000000000000000000000000000010000000100000000000000000;
        assert_eq!(generate_moves_result(&board, 9, PieceColor::Black), (expected_bitboard, Some(25), None));

        // Test white pawn double move
        let board = Board::new();
        let expected_bitboard: u64 = 0b0000000000000000000100000001000000000000000000000000000000000000;
        assert_eq!(generate_moves_result(&board, 52, PieceColor::White), (expected_bitboard, Some(36), None));

        // Test white pawn double move (blocked by piece)
        let board = read_fen("rnbqkbnr/ppp2pp1/7p/3pp3/3P4/2N5/PPP1PPPP/R1BQKBNR w KQkq - 0 1");
        assert_eq!(generate_moves_result(&board, 53, PieceColor::White), (0, None, None));

        // Test white pawn en-passant
        let board = read_fen("rnbqkbnr/ppppp1pp/8/5pP1/8/8/PPPPPP1P/RNBQKBNR w KQkq 26 0 1");
        let expected_bitboard: u64 = 0b0000000000000000000000000000000000000000000001100000000000000000;
        assert_eq!(generate_moves_result(&board, 25, PieceColor::White), (expected_bitboard, None, Some((26, 18))))
    }
}