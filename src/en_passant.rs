use crate::board_representation;
use crate::board_representation::{Board, PieceColor};
use crate::pieces;

// Returns en passant capture bit and move bit if an en passant move is available
// Does not assumes the piece given is a pawn
pub fn get_en_passant_capture(board: &Board, friendly_board: &[u64; 3], enemy_board: &[u64; 3], piece_bit: u8) -> Option<(u8, u8)> {

    // No need to check for en-passant caputes if the piece isn't a pawn
    if board_representation::read_piece_id(friendly_board, piece_bit) != pieces::PAWN_ID {
        return None;
    }

    if let Some(en_passant_target_bit) = board.en_passant_target_bit {

        // The en-passant can't be made if the friendly pawn isn't next to the enemy pawn
        if (piece_bit as i8 - en_passant_target_bit as i8).abs() > 1 {
            return None;
        }
        
        // Check there is an enemy pawn in the en_passant target bit
        if board_representation::read_piece_id(enemy_board, en_passant_target_bit) == pieces::PAWN_ID {
            
            let ep_move_bit = calc_ep_move_bit(en_passant_target_bit, board.piece_to_move);
            let friendly_move_bit_id = board_representation::read_piece_id(friendly_board, ep_move_bit);
            let enemy_move_bit_id = board_representation::read_piece_id(enemy_board, ep_move_bit);

            // If there is no friendly or enemy piece in the en passant move bit then the pawn can perform the en passant
            if friendly_move_bit_id + enemy_move_bit_id == 0 {
                return Some((en_passant_target_bit, ep_move_bit))
            }
        }
    }

    None
}

// Calculate en-passant move bit from en-passant target bit
fn calc_ep_move_bit(en_passant_target_bit: u8, piece_color: PieceColor) -> u8 {
    match piece_color {
        PieceColor::White => en_passant_target_bit - 8,
        PieceColor::Black => en_passant_target_bit + 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::fen::read_fen;

    #[test]
    fn test_calc_ep_move_bit() {

        // Test en-passant (white capturing)
        let board = read_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq 28 0 1");
        let result = get_en_passant_capture(&board, &board.white_board, &board.black_board, 27);

        assert_eq!(result, Some((28, 20)));

        // Test no en-passant (piece blocking)
        let board = read_fen("r1bqkbnr/ppp1pppp/3n4/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq 28 0 1");
        let result = get_en_passant_capture(&board, &board.white_board, &board.black_board, 27);

        assert_eq!(result, None);

        // Test en-passant (black capturing)
        let board = read_fen("rnbqkbnr/ppppp1pp/8/8/5pP1/8/PPPPPP1P/RNBQKBNR b KQkq 33 0 1");
        let result = get_en_passant_capture(&board, &board.black_board, &board.white_board, 34);

        assert_eq!(result, Some((33, 41)));

        // Test white trying to do an en-passant when the capturing pawn isn't in the correct position
        let board = read_fen("rnbqkbnr/pppp1ppp/8/4p3/8/5P2/PPPPP1PP/RNBQKBNR w KQkq 27 0 1");
        let result = get_en_passant_capture(&board, &board.white_board, &board.black_board, 42);

        assert_eq!(result, None);
    }

}