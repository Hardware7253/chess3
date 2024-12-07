pub mod pieces;
pub mod direction_bitboards;
pub mod board_representation;

pub mod generic_math;
pub mod fixed_vecor;
pub mod bitboard_manipulation;

pub mod move_generation;
pub mod check_validation;
pub mod en_passant;
pub mod castling;

pub mod turn;
pub mod bot;
pub mod bot_eval;
pub mod pesto;

use std::time::Duration;



fn main() {

    // Generate best move for a current position describes by a fen code
    let board = board_representation::fen::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let best_move = bot::gen_best_move(&board, Duration::from_secs(1));
    println!("{:?}", best_move);

    // Code I use for generating bitboards for unit tests
    // Also for debugging
    //let board = board_representation::fen::read_fen("6bk/6pp/8/1r6/8/8/8/K6r b - - 0 1");
    //println!("{:?}", board);
    //println!("0b{:064b}", board.black_board[0]);

    // Print bytes function is also really helpful for debugging
    // use bitboard_manipulation::debugging::print_bytes;
    // print_bytes(525056 |524290 | 274877906946);
}


// let board = board_representation::fen::read_fen("8/8/5pp1/8/8/8/8/8 w HAha - 0 1");
// println!("{:?}", board);
// println!("0b{:064b}", board.black_board[0]);