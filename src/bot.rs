// For main chess bot algorithm
// Quiescence Search

use std::time::{Duration, Instant};

use crate::board_representation;
use crate::board_representation::{Board, PerspectiveBoards};
use crate::move_generation;
use crate::bitboard_manipulation;
use crate::fixed_vecor::FixedVector;
use crate::turn;
use crate::check_validation;
use crate::bot_eval::eval;

// Non capture weight for move ordering
// Use value of -10 so non captures are searched last
const NON_CAPTURE_WEIGHT: i8 = -10;

// Checkmate weight for minimax
// Use 5.0 because typical max value from eval fn is 1.0
const CHECKMATE_WEIGHT: f32 = 5.0;

const QUIESCENCE_SEARCH_MAX_DEPTH: u8 = 3;
const FIXED_VECTOR_PLACEHOLDER_VALUE: u8 = 255;

// Max values for fixed vectors
const MAX_MOVE_BITBOARD_BITS_ON: usize = 28;
const MAX_TEAM_MOVES: usize = 96; // Maximum valid moves for one team in a turn

// Move information for move ordering vector
#[derive(Clone, Copy, PartialEq, Debug)]
struct MoveInformation {
    initial_bit: u8,
    final_bit: u8,

    move_score: i8,
    ep_bits: (Option<u8>, Option<u8>)
}

impl MoveInformation {
    fn new() -> Self {
        MoveInformation {
            initial_bit: 0,
            final_bit: 0,
            move_score: i8::MIN,
            ep_bits: (None, None)
        }
    }
}

// Generate best move using iterative deepening to get pv-moves
// Returns a tuple with the initial pieces bit and the final bit it moves to
pub fn gen_best_move(board: &Board, max_duration: Duration) -> Result<(u8, u8), ()> {
    let start = Instant::now();

    let mut pv_move: Option<MoveInformation> = None;
    for depth_limit in 3..100 {
        let (_, move_information, timeout) = minimax(&board, 0, None, pv_move, true, 0, depth_limit, false, &start, &max_duration);

        // Everything from the search that was currently running when the timeout occured is thrown out
        // Instead use the old pv move as the best result
        if timeout {
            break;
        } else {
            pv_move = Some(move_information);
        }
    }

    // Return best move
    if let Some(pv_move) = pv_move {
        Ok((pv_move.initial_bit, pv_move.final_bit))
    } else {
        Err(())
    }
}

// Generates best move using minimax algorithm
//
// Returns a tuple of the min/max value, move_information, and a bool which is true if the function timed out
fn minimax(
    board: &Board,

    parent_value: i8,
    parent_min_max: Option<f32>, // For pruning

    // Move that is searched first
    // (leftmost branch)
    pv_move: Option<MoveInformation>,

    is_returning_max: bool,
    current_depth: u8, // Depth of 0 for root

    // Depth at which the tree stops being searched down in favor of a final quiescence search
    depth_limit: u8,
    quiescence_search: bool,

    // For making search exit once it has been running for too long
    start_instant: &Instant,
    timeout_duration: &Duration,
) -> (f32, MoveInformation, bool) {

    // Timeout
    if start_instant.elapsed() > *timeout_duration {
        return (0.0, MoveInformation::new(), true)
    }

    // What to do when the depth limit is reached
    if current_depth == depth_limit {
        if quiescence_search { // Stop quiescence search
            return (eval(parent_value, board), MoveInformation::new(), false);
        } else { // Start quiescence search
            return minimax(
                board,                          // board
                parent_value,                   // parent_value
                None,                           // parent_min_max
                None,                           // pv_move
                is_returning_max,               // is_returning_max
                0,                              // current_depth
                QUIESCENCE_SEARCH_MAX_DEPTH,    // depth_limit
                true,                           // quiescence_search
                start_instant,                  // start_instant
                timeout_duration,               // timeout_duration
            );
        }
    }

    let (mut min_or_max, parent_min_max_def, min_max_multiplier) =
    if is_returning_max {
        (f32::MIN, f32::MAX, 1)
    } else {
        (f32::MAX, f32::MIN, -1)
    };

    // If no parent min or max is provided use one that will result in no pruning
    let parent_min_max = parent_min_max.unwrap_or(parent_min_max_def);

    // Get initial information
    let perspective_boards = PerspectiveBoards::gen(board, board.piece_to_move);
    let moves = order_moves(&board, pv_move, &perspective_boards);
    let potential_checking_pieces = check_validation::get_potential_checking_pieces(&board, board.piece_to_move);

    let mut king_was_in_check = false;
    let mut children_searched = 0;
    let mut best_move: MoveInformation = MoveInformation::new();

    for i in 0..moves.len() {
        let move_information = moves.internal_array[i];
        let piece_id = board_representation::read_piece_id(perspective_boards.friendly_board, move_information.initial_bit);

        // Make turn by moving the piece from the initial bit to the final bit
        // Only make a turn if it involves a capture when quiescence_search == true
        let turn_data = turn::take_turn(
            board,
            piece_id,
            move_information.initial_bit,
            move_information.final_bit,
            quiescence_search,
            move_information.ep_bits,
            potential_checking_pieces.clone()
        );

        if let Ok((new_board, capture_value)) = turn_data {
            children_searched += 1;

            // Sign of capture value changes if the enemy is making a capture
            // (negatively influences team which the search is running in favor of)
            let capture_value = capture_value * min_max_multiplier;
            
            // Sign of capture value changes if the enemy is making a capture
            // (negatively influences team which the search is running in favor of)
            let capture_value = capture_value * min_max_multiplier;
            let (branch_value, _, timeout) = minimax(
                &new_board,                     // board
                parent_value + capture_value,   // parent_value
                Some(min_or_max),               // parent_min_max
                None,                           // pv_move
                !is_returning_max,              // is_returning_max
                current_depth + 1,              // current_depth
                depth_limit,                    // depth_limit
                quiescence_search,              // quiescence_search
                start_instant,                  // start_instant
                timeout_duration,               // timeout_duration
            );

            // Propogate timeout upwards
            if timeout {
               return (0.0, MoveInformation::new(), timeout); 
            }

            // Update min or max value and best move
            if update_min_or_max(min_or_max, branch_value, is_returning_max) {
                min_or_max = branch_value;
                best_move = move_information;
            }

            // Prune branches which do not need to be searched down
            if prune(parent_min_max, min_or_max, is_returning_max) {
                break;
            }
        } else if turn_data == Err(turn::TurnError::Check) {
            king_was_in_check = true;
        }
    }

    // If 0 children were searched there are no valid moves for the piece
    // If the king is in check this makes a checkmate
    if children_searched == 0 {
        if quiescence_search {
            return (eval(parent_value, board), MoveInformation::new(), false);
        } else if king_was_in_check {

            // Ignore checkmates for quiescence_search since it only evaluates capture moves
            return (CHECKMATE_WEIGHT * -min_max_multiplier as f32, MoveInformation::new(), false);
        }
    }

    return (min_or_max, best_move, false);
}

// Return true if the min_or_max value should be updated to the branch_value
fn update_min_or_max(min_or_max: f32, branch_value: f32, is_returning_max: bool) -> bool {
    if is_returning_max {
        if branch_value > min_or_max {
            return true;
        }
    } else {
        if branch_value < min_or_max {
            return true;
        }
    }

    false
}

// Return true if the current branch should be pruned
fn prune(parent_min_max: f32, min_or_max: f32, is_returning_max: bool) -> bool {
    if is_returning_max {
        min_or_max >= parent_min_max
    } else {
        min_or_max <= parent_min_max
    }
}

// Returns a FixedVector of mostly valid moves, with the format (initial_bit, final_bit, move_score)
// This does not consider king safety
fn order_moves(
    board: &Board,
    pv_move: Option<MoveInformation>,
    perspective_boards: &PerspectiveBoards<'_>,
) -> FixedVector<MoveInformation, MAX_TEAM_MOVES>{
    let mut moves_fixed_vector: FixedVector<MoveInformation, MAX_TEAM_MOVES> = FixedVector::new(MoveInformation::new());

    for initial_bit in 0..64 {
        let piece_id = board_representation::read_piece_id(perspective_boards.friendly_board, initial_bit);
        let piece_value = perspective_boards.friendly_piece_information[piece_id].piece_value;

        if piece_id == 0 {
            continue;
        }

        // Generate moves for this piece
        let (
            move_bitboard,
            en_passant_target_bit,
            en_passant_cap_bits
        ) = move_generation::generate_moves(board, initial_bit, piece_id, board.piece_to_move, &perspective_boards);

        let final_bits_vec: FixedVector<u8, MAX_MOVE_BITBOARD_BITS_ON> = bitboard_manipulation::bits_on(move_bitboard, FIXED_VECTOR_PLACEHOLDER_VALUE);

        // Iterate over each move
        for i in 0..final_bits_vec.len() {
            let final_bit = final_bits_vec.internal_array[i];

            // Skip over pv_move bits so they dont get added to the output vec twice
            if let Some(pv_move) = pv_move {
                if initial_bit == pv_move.initial_bit && final_bit == pv_move.final_bit {
                    continue;
                }
            }
            
            // Get enemy piece value
            let enemy_piece_id = board_representation::read_piece_id(perspective_boards.enemy_board, final_bit);
            let enemy_piece_value = if enemy_piece_id == 0 {
                0
            } else {
                perspective_boards.friendly_piece_information[enemy_piece_id].piece_value
            };

            // Calculate move score
            let move_score = if enemy_piece_value == 0 {
                NON_CAPTURE_WEIGHT
            } else {
                enemy_piece_value - piece_value
            };

            let ep_bits = turn::get_ep_bits_for_turn(en_passant_target_bit, en_passant_cap_bits, final_bit);
            let move_information = MoveInformation {
                initial_bit,
                final_bit,
                move_score,
                ep_bits,
            };

            moves_fixed_vector.push(move_information);
        }
    }

    // Add pv move with max move score so it is sorted ontop of the array
    if let Some(mut pv_move) = pv_move {
        pv_move.move_score = i8::MAX;
        moves_fixed_vector.push(pv_move);
    }
    
    // Sort moves
    moves_fixed_vector.internal_array.sort_by(|a, b| b.move_score.cmp(&a.move_score));
    moves_fixed_vector
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_representation::fen::read_fen;

    #[test]
    fn test_bot() {
        let board = board_representation::fen::read_fen("7k/6pp/8/1r6/6b1/8/8/K7 b - - 0 1");
        let best_move = gen_best_move(&board, Duration::from_secs(1));

        assert_eq!(best_move, Ok((33, 19)));
    }

    #[test]
    fn test_order_moves() {
        let board = read_fen("6pk/3p2pp/r7/8/6p1/3Q3q/8/K7 w - - 0 1");
        let perspective_boards = PerspectiveBoards::gen(&board, board.piece_to_move);
        let result = order_moves(&board, None, &perspective_boards);

        assert_eq!(result.len(), 27);
    }
}