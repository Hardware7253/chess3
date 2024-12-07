use crate::board_representation;
use crate::board_representation::{Board, PieceColor};
use crate::pieces;
use crate::check_validation;
use crate::check_validation::MAX_CHECKING_PIECES;
use crate::fixed_vecor::*;

// For the thing don't iterate over every thing, use the bits on thing
// Maybe benchmark both
// Instead of returning a blank ass error you could make an enum for it to describe the type of error
// Right now there is only one type of error though, so kind of pointless

#[derive(Debug, PartialEq)]
pub enum TurnError {
    Check,
    NotCapture,
}

// Takes a turn by moving piece at initial_bit to the final_bit
// Returns the a new, updated board and the value of any pieces captured
// The initial and final bits are assumed to be valid
pub fn take_turn(
    initial_board: &Board,
    piece_id: usize,
    initial_bit: u8,
    final_bit: u8,
    only_use_captures: bool,
    ep_bits_for_turn: (Option<u8>, Option<u8>),
    potential_checking_pieces: FixedVector<u8, MAX_CHECKING_PIECES>,
) -> Result<(Board, i8), TurnError> {
    let mut new_board = initial_board.clone();

    let (en_passant_target_bit, en_passant_capture_bit) = ep_bits_for_turn;
    
    let (friendly_board, enemy_board, next_piece_to_move) = match new_board.piece_to_move {
        PieceColor::Black => (
            &mut new_board.black_board,
            &mut new_board.white_board,
            PieceColor::White
        ),

        PieceColor::White => (
            &mut new_board.white_board,
            &mut new_board.black_board,
            PieceColor::Black
        ),
    };

    // Get the captured piece id
    // The location of the captured piece is the bit which the piece moves to
    // Unless the move is an en-passant
    let capture_piece_id = if let Some(en_passant_capture_bit) = en_passant_capture_bit {
        let id = board_representation::read_piece_id(&enemy_board, en_passant_capture_bit);
        board_representation::remove_piece(en_passant_capture_bit, enemy_board);
        new_board.en_passant_target_bit = None;

        id
    } else {
        board_representation::read_piece_id(&enemy_board, final_bit)
    };

    // Get capture piece value
    let capture_piece_value = if capture_piece_id == 0 {
        if only_use_captures {
            return Err(TurnError::NotCapture);
        }
        
        0
    } else {
        pieces::BLACK_PIECE_INFORMATION[capture_piece_id].piece_value
    };

    // Subtract material value of capture from enemy teams total material
    match new_board.piece_to_move {
        PieceColor::Black => new_board.white_material -= capture_piece_value,
        PieceColor::White => new_board.black_material -= capture_piece_value,
    }

    // Move friendly piece to it's new position
    // Remove enemy piece from the position the piece moves to
    board_representation::remove_piece(initial_bit, friendly_board);
    board_representation::insert_piece(final_bit, piece_id, friendly_board);
    board_representation::remove_piece(final_bit, enemy_board);

    //crate::bitboard_manipulation::debugging::print_bytes(friendly_board[1]);
    //crate::bitboard_manipulation::debugging::print_bytes(new_board.white_board[1]);

    // If the king is moved the potential checking pieces needs to be updated
    // Otherwise recalculation can be avoided
    let potential_checking_pieces = if piece_id == pieces::KING_ID {
        match initial_board.piece_to_move {
            PieceColor::Black => new_board.black_king_bit = final_bit,
            PieceColor::White => new_board.white_king_bit = final_bit,
        }

        check_validation::get_potential_checking_pieces(&new_board, initial_board.piece_to_move)
    } else {
        potential_checking_pieces
    };

    // If the king is in check return an error
    if check_validation::is_king_in_check(&new_board, initial_board.piece_to_move, &potential_checking_pieces) {
        return Err(TurnError::Check);
    }

    // Rest of the function for updating board states / clocks
    if initial_board.piece_to_move == PieceColor::Black {
        new_board.fullmove_number += 1;
    }

    // set / reset en-passant target bit
    new_board.en_passant_target_bit = en_passant_target_bit;

    if capture_piece_value == 0 {
        if piece_id == pieces::PAWN_ID {
            new_board.halfmove_clock = 0; // Reset halfmove clock when a pawn advances
        } else {
            new_board.halfmove_clock += 1; // Increment halfmove clock when no capture is made
        }
    } else {
        new_board.halfmove_clock = 0; // Reset halfmove clock when a capture is made
    }

    new_board.piece_to_move = next_piece_to_move;

    Ok((new_board, capture_piece_value))
}

// For converting en_passant outputs from move generator to those needed by the turn function
pub fn get_ep_bits_for_turn(
    en_passant_target_bit: Option<u8>,
    en_passant_cap_bits: Option<(u8, u8)>,
    move_bit: u8
) -> (Option<u8>, Option<u8>) {
    // Get en-passant target bit if the piece is performing a double move as its turn
    let en_passant_target_bit = if let Some(en_passant_target_bit) = en_passant_target_bit {

        // If the move bit is equal to the en passant target bit
        // this piece is performing a double move
        if move_bit == en_passant_target_bit {
            Some(en_passant_target_bit)
        } else {
            None
        }
    } else {
        None
    };

    let en_passant_capture_bit = if let Some(en_passant_cap_bits) = en_passant_cap_bits {

        // If the move bit is equal to the en passant move bit
        // this piece is performing an en-passant
        if move_bit == en_passant_cap_bits.1 {
            Some(en_passant_cap_bits.0)
        } else {
            None
        }
    } else {
        None
    };

    // en_passant_target_bit: Only set when a pawn does a double move as part of the turn
    // en_passant_capture_bit: Only set when an en-passant capture is being made as part of the turn
    (en_passant_target_bit, en_passant_capture_bit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::fen::read_fen;

    #[test]
    fn test_take_turn() {

        // Test white capturing a piece
        let board = read_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 1");
        let potential_checking_pieces = check_validation::get_potential_checking_pieces(&board, board.piece_to_move);
        let expected_board = read_fen("r1bqkb1r/pppp1ppp/2n2n2/4N3/4P3/2N5/PPPP1PPP/R1BQKB1R b KQkq - 0 1");

        assert_eq!(take_turn(&board, 2, 42, 27, false, (None, None), potential_checking_pieces), Ok((expected_board, 1)));

        // Test white attempting to put it's own king in check (error)
        let board =  read_fen("r1bqkb1r/p1pp1pp1/1p3n1p/4n3/6b1/2N5/PPPP1PPP/R1BQK2R w KQkq - 0 1");
        let potential_checking_pieces = check_validation::get_potential_checking_pieces(&board, board.piece_to_move);

        assert_eq!(take_turn(&board, 6, 59, 51, false, (None, None), potential_checking_pieces), Err(TurnError::Check)); 

        // Test black doing an en-passant
        let board =  read_fen("rn1qkbnr/p1ppp1pp/bp6/8/5pP1/2N5/PPPPPP1P/R1BQKBNR b KQkq 33 0 1");
        let potential_checking_pieces = check_validation::get_potential_checking_pieces(&board, board.piece_to_move);
        let expected_board = read_fen("rn1qkbnr/p1ppp1pp/bp6/8/8/2N3p1/PPPPPP1P/R1BQKBNR w KQkq - 0 2");

        assert_eq!(take_turn(&board, 1, 34, 41, false, (None, Some(33)), potential_checking_pieces), Ok((expected_board, 1)));
    }
}