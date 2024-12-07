// This file contains code related to manipulating piece bitboards

use crate::direction_bitboards;
use crate::fixed_vecor::FixedVector;

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

// Shifts a direction bitboard so that it is alligned with a piece at $piece_coordinates
// This needs to be done to obtain a move bitboard for any piece
pub fn shift_direction_bitboard(piece_bit: u8, piece_coordinates: (i8, i8), direction_bitboard: &direction_bitboards::DirectionBitboard) -> u64 {

    let origin_coordinates = get_piece_coordinates(direction_bitboard.origin_bit);

    // If the piece is already at the bitboard origin coordinates then no work has to be done
    if origin_coordinates == piece_coordinates {
        return direction_bitboard.bitboard
    }

    let dx = origin_coordinates.0 - piece_coordinates.0;
    let dy = origin_coordinates.1 - piece_coordinates.1;

    //return shift_bytes(direction_bitboard.bitboard, dx - dy),

    match direction_bitboard.shift_type {
        direction_bitboards::ShiftType::Standard => return shift_u64(direction_bitboard.bitboard, dy * 8),
        direction_bitboards::ShiftType::Byte => return shift_bytes(direction_bitboard.bitboard, dx),
        direction_bitboards::ShiftType::Both => return shift_bytes(shift_u64(direction_bitboard.bitboard, dy * 8), dx),
        direction_bitboards::ShiftType::Diagonal => {

            let piece_row = piece_coordinates.1 as u8;
            
            let direction_bitboard_row_byte = isolate_byte(&direction_bitboard.bitboard, piece_row);
            let piece_position_bitboard_row_byte = isolate_byte(&(1 << piece_bit), piece_row);

            let dx = piece_position_bitboard_row_byte.trailing_zeros() as i8 - direction_bitboard_row_byte.trailing_zeros() as i8;
            shift_bytes(direction_bitboard.bitboard, -dx)
        },
    }
}


// Remove invalid move positions from bitboards which define movement in a single direction (no combinations)
// Returns a new fixed bitboard, and the first intersecting bits
// E.g. 0b11011101 -> 0b00011100 (byte for example, this function does u64 numbers)
//
// move_bitboard
// bitboard that describes the movement direction (for the piece)
//
// direction_bitboard
// describes the piece movement directions generically (not necasarily centered at the pieces position)
//
// masked_bitboard
// bitboard that describes the movement of the piece
// may have bits removed from original move_bitboard where there are conflicting pieces
pub fn fix_move_bitboard(piece_coordinates: (i8, i8), direction_bitboard: &u64, move_bitboard: &u64, masked_bitboard: &u64) -> (u64, (Option<i8>, Option<i8>)) {
    if move_bitboard == masked_bitboard {
        return (*masked_bitboard, (None, None))
    }

    let (piece_column, piece_row) = piece_coordinates;

    // Fix vertical move bitboards (and diagonal bitboards)
    if direction_bitboard != &direction_bitboards::HORIZONTAL_LINE.bitboard {
        let fix_bitboard_lower = remove_move_end_vertical(piece_row - 1, move_bitboard, masked_bitboard, false);
        let fix_bitboard_upper = remove_move_end_vertical(piece_row + 1, move_bitboard, masked_bitboard, true);

        //
        let intercept_bitboard = move_bitboard ^ masked_bitboard;

        // Tuple containg bits where the piece first intersected with another piece
        let intercept_bits = (
            get_piece_bit_option(get_column_from_row(fix_bitboard_lower.1, intercept_bitboard), fix_bitboard_lower.1),
            get_piece_bit_option(get_column_from_row(fix_bitboard_upper.1, intercept_bitboard), fix_bitboard_upper.1)
        );

        let fixed_bitboard = fix_bitboard_lower.0 | fix_bitboard_upper.0;
        return (fixed_bitboard, intercept_bits);
    };

    // Fix horizontal move bitboards
    let move_mask_byte = isolate_byte(masked_bitboard, piece_row as u8);

    let fix_byte_lower = remove_byte_ends(piece_column - 1, move_mask_byte, false);
    let fix_byte_upper = remove_byte_ends(piece_column + 1, move_mask_byte, true);

    // Tuple containg bits where the piece first intersected with another piece
    let intercept_bits = (
        get_piece_bit_option(fix_byte_lower.1, Some(piece_row)),
        get_piece_bit_option(fix_byte_upper.1, Some(piece_row))
    );

    let fixed_byte = fix_byte_lower.0 | fix_byte_upper.0;
    let fixed_bitboard = (fixed_byte as u64) << piece_row * 8;

    (fixed_bitboard, intercept_bits)
}

// Get the column of an intercept bit
//
// If multiple move bitboards were combined to make $intercept_bitboard this function will fail
// because it relies on only one bit being on in each byte to determine the column
fn get_column_from_row(row: Option<i8>, intercept_bitboard: u64) -> Option<i8> {
    if let Some(row) = row {
        return Some(isolate_byte(&intercept_bitboard, row as u8).trailing_zeros() as i8);
    }

    None
}

// A shortcut function to use in fix_move_bitboard
fn get_piece_bit_option(piece_column: Option<i8>, piece_row: Option<i8>) -> Option<i8> {
    if let (Some(piece_column), Some(piece_row)) = (piece_column, piece_row) {
        return Some(get_piece_bit((piece_column, piece_row)));
    }
    None
}

// Get piece bit from coordinates
fn get_piece_bit(piece_coordinates: (i8, i8)) -> i8 {
    piece_coordinates.1 * 8 + piece_coordinates.0
}

// Returns a tuple containing the pieces column and row
pub fn get_piece_coordinates(piece_bit: u8) -> (i8, i8) {
    let piece_column = piece_bit % 8;
    let piece_row = piece_bit / 8;

    (piece_column as i8, piece_row as i8)
}

// Remove floating ends of a masked vertical move bitboard
// Only does this in one direction (has to be called twice to remove both ends)
//
// The function checks outwards from $piece_row
// $check_up describes which direction to check (up/down rows)
//
// Returns a tuple containing the new bitboard and row which the function stopped iterating at
// Functionality is similiar to remove_byte_ends function, except working an entire byte at a time, rather than a bit
fn remove_move_end_vertical(piece_row: i8, move_bitboard: &u64, masked_bitboard: &u64, check_up: bool) -> (u64, Option<i8>) {
    if piece_row > 7 || piece_row < 0 {
        return (0, None)
    }

    let mut output: (u64, Option<i8>) = (0, Some(piece_row));

    let move_byte = isolate_byte(move_bitboard, piece_row as u8);
    let mask_byte = isolate_byte(masked_bitboard, piece_row as u8);

    // If the move byte equals the mask byte then this is a valid move position
    // The next position should then be checked to see if it is valid
    if move_byte == mask_byte {
        output.0 |= (mask_byte as u64) << piece_row * 8;

        let next_output = if check_up {
            remove_move_end_vertical(piece_row + 1, move_bitboard, masked_bitboard, check_up)// Go up row
        } else {
            remove_move_end_vertical(piece_row - 1, move_bitboard, masked_bitboard, check_up) // Go down row
        };

        output.0 |= next_output.0;
        output.1 = next_output.1;
    }

    output
}


// Remove floating ends of a byte (for masked horizontal move bitboard)
// Only does this in one direction (has to be called twice to remove both ends)
//
// The function checks outwards from $bit
// $check_up describes which direction to check in the byte 
// up (towards MSB) 
// down (towards LSB)
//
// Returns a tuple containing the new byte and bit which the function stopped iterating at
// E.g.
// remove_byte_ends(3, 0b11111101, false) -> (0b00001100, Some(1))
fn remove_byte_ends(bit: i8, test_byte: u8, check_up: bool) -> (u8, Option<i8>){
    if bit > 7 || bit < 0 {
        return (0, None)
    }

    let mut output: (u8, Option<i8>) = (0, Some(bit));

    // Recursively add bits to output byte until a 0 is reached, then stop
    if bit_on(test_byte, bit as u8) {
        output.0 |= 1 << bit;

        let next_output = if check_up {
            remove_byte_ends(bit + 1, test_byte, check_up)// Go up byte
        } else {
            remove_byte_ends(bit - 1, test_byte, check_up) // Go down byte
        };

        output.0 |= next_output.0;
        output.1 = next_output.1;
    }

    output
}

// Returns a vector containing the bits that are on in a u64 number
pub fn bits_on<const L: usize>(num: u64, placeholder_num: u8) -> FixedVector<u8, L> {
    let mut bits_on_vector = FixedVector::new(placeholder_num);

    let mut num = num;

    let mut bits_counted = 0;
    while bits_counted < 64 && bits_on_vector.len() < L {
        let trailing_zeros = num.trailing_zeros() as u8;

        if num == 0 {
            break;
        }

        // Count trailing zeros to avoid having to iterate over every bit
        if trailing_zeros > 0 {
            bits_counted += trailing_zeros;
            num >>= trailing_zeros;
        } else {
            
            // Add bit 0 (which will be on if there are no trailing zeros)
            // to output vector
            bits_on_vector.push(bits_counted);
            num >>= 1;

            bits_counted += 1;
        }
    }

    bits_on_vector
}


// Return true if a specified bit is on in a number
// Generic implementation shamelessly yoinked from ChatGPT
pub fn bit_on<T>(num: T, bit: u8) -> bool
where
    T: std::ops::BitAnd<Output = T>
        + std::ops::BitOr<Output = T>
        + std::ops::Shl<u8, Output = T>
        + Copy
        + PartialEq
        + From<u8>,
{
    let mask = T::from(1) << bit;
    (num & mask) != T::from(0)
}


// Instead of doing a bitwise shift on an entire 64 bit number only shift each byte
// Sign of shift variable indicates direction
// + (right)
// - (left)
pub fn shift_bytes(num: u64, shift: i8) -> u64 {
    let mut output: u64 = 0;
    for i in 0..8 {
        let current_byte = isolate_byte(&num, i);

        let new_byte = if shift < 0 {
            current_byte << shift.abs()
        } else {
            current_byte >> shift
        };

        output |= (new_byte as u64) << (i * 8)
    }
    output
}


// Shifts a u64
// Sign of shift variable indicates direction
// + (right)
// - (left)
pub fn shift_u64(num_to_shift: u64, shift: i8) -> u64 {
    if shift < 0 {
        return num_to_shift << shift.abs();
    }
    num_to_shift >> shift
}


// Isolates a byte in a 64 bit number
// The 0th byte of a 64 bit number is considered to be composed of the 8 least significant bits
pub fn isolate_byte(num: &u64, isolate: u8) -> u8 {
    (num >> isolate * 8) as u8
}


pub mod debugging {
    use super::*;

    // Prints each byte in binary of a u64
    // Prints the least significant byte first, and the most significant last
    // Basically for giving a visual representation of a bitboard and the positions each bit represents
    pub fn print_bytes(num: u64) {
        println!("{:064b}", num);
        for i in 0..8 {
            println!("{:08b}", isolate_byte(&num, i));
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction_bitboards::*;

    #[test]
    fn test_coordinate_manipulation() {
        assert_eq!(get_piece_bit(get_piece_coordinates(32)), 32);
    }

    #[test]
    fn test_shift_bytes() {
        let expected_right_shift: u64 = 0b1100000001100000001100000001100000001100000001100000001100000001 ^ DIAGONAL_RIGHT.bitboard;
        let expected_left_shift: u64 = 0b1000000001000000101000000101000000101000000101000000101000000101 ^ DIAGONAL_RIGHT.bitboard;

        assert_eq!(shift_bytes(DIAGONAL_RIGHT.bitboard, 1), expected_right_shift);
        assert_eq!(shift_bytes(DIAGONAL_RIGHT.bitboard, -2), expected_left_shift);
    }

    #[test]
    fn test_isolate_bytes() {
        assert_eq!(isolate_byte(&52, 0), 52);
        assert_eq!(isolate_byte(&342, 1), 1);
    }

    #[test]
    fn test_remove_move_end_vertical() {
        let test_mask: u64  = 0b0000000001000000001000000001000000001000000001000000000000000001;
        let expected_d: u64 = 0b0000000000000000000000000001000000001000000001000000000000000000;
        let expected_u: u64 = 0b0000000001000000001000000001000000000000000000000000000000000000;

        let result_d = remove_move_end_vertical(4, &DIAGONAL_RIGHT.bitboard, &test_mask, false);
        let result_u = remove_move_end_vertical(4, &DIAGONAL_RIGHT.bitboard, &test_mask, true);

        assert_eq!(result_d, (expected_d, Some(1)));
        assert_eq!(result_u, (expected_u, Some(7)));
    }

    #[test]
    fn test_remove_byte_ends() {
        let test_byte = 0b11111101;
        let expected_d  = 0b00001100;
        let expected_u = 0b11111000;

        assert_eq!((remove_byte_ends(3, test_byte, false)), (expected_d, Some(1)));
        assert_eq!((remove_byte_ends(3, test_byte, true)), (expected_u, None));

        let test_byte = 0b00111101;
        let expected_d  = 0b00011100;
        let expected_u = 0b00110000;

        assert_eq!((remove_byte_ends(4, test_byte, false)), (expected_d, Some(1)));
        assert_eq!((remove_byte_ends(4, test_byte, true)), (expected_u, Some(6)));
    }

    #[test]
    fn test_bit_on() {
        assert_eq!(bit_on(129, 7), true);
    }

    #[test]
    fn test_fix_move_bitboard() {

        // Test vertical case
        let test_mask: u64  = 0b0000000001000000001000000001000000001000000001000000000000000001;
        let expected: u64 = 0b0000000001000000001000000000000000001000000001000000000000000000;

        let result = fix_move_bitboard(get_piece_coordinates(36), &DIAGONAL_RIGHT.bitboard, &DIAGONAL_RIGHT.bitboard, &test_mask);

        assert_eq!(result, (expected, (Some(9), Some(63))));

        // Test horizontal case
        let row = 4;
        let column = 5;
        let move_bitboard = 0b0000000000000000000000001111111100000000000000000000000000000000;
        let test_mask = 0b0000000000000000000000001101111100000000000000000000000000000000;
        let expected = 0b0000000000000000000000001101111100000000000000000000000000000000;
        let result = fix_move_bitboard((column, row), &HORIZONTAL_LINE.bitboard, &move_bitboard, &test_mask);
        
        assert_eq!(result, (expected, (None, None)));
    }

    #[test]
    fn test_bits_on() {
        let mut expected_vec: FixedVector<u8, 16> = FixedVector::new(255);
        expected_vec.push(2);
        expected_vec.push(4);

        assert_eq!(bits_on(20, 255), expected_vec);

        let mut expected_vec: FixedVector<u8, 64> = FixedVector::new(0);
        expected_vec.push(2);
        expected_vec.push(4);
        expected_vec.push(5);
        expected_vec.push(6);
        expected_vec.push(11);

        assert_eq!(bits_on(2164, 0), expected_vec);

        let num: u64 = 0b1000000000000000000000000000000000000000000000000000000000000000;
        assert_eq!(bits_on(131071, 255).internal_array, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 255, 255]);
        assert_eq!(bits_on(num, 21).internal_array, [63, 21])

    }

    #[test]
    fn test_shift_direction_bitboard() {

        // Test knight moving
        let expected: u64 = 2833441750646784;
        let piece_bit = 34;
        let piece_coordinates = get_piece_coordinates(piece_bit);
        let result = shift_direction_bitboard(piece_bit, piece_coordinates, &direction_bitboards::KNIGHT_MOVES);
        assert_eq!(result, expected);

        // Test shifting diagonal left bitboard
        let expected: u64 = 4328785936;
        let piece_bit = 11;
        let piece_coordinates = get_piece_coordinates(piece_bit);
        let result = shift_direction_bitboard(piece_bit, piece_coordinates, &direction_bitboards::DIAGONAL_LEFT);
        assert_eq!(result, expected);

        // Test shifting diagonal right bitboard
        let expected: u64 = 141012904183812;
        let piece_bit = 47;
        let piece_coordinates = get_piece_coordinates(piece_bit);
        let result = shift_direction_bitboard(piece_bit, piece_coordinates, &direction_bitboards::DIAGONAL_RIGHT);
        assert_eq!(result, expected);

        // Test shifting vertical line bitboard
        let expected: u64 = 1157442765409226768;
        let piece_bit = 52;
        let piece_coordinates = get_piece_coordinates(piece_bit);
        let result = shift_direction_bitboard(piece_bit, piece_coordinates, &direction_bitboards::VERTICAL_LINE);
        assert_eq!(result, expected);

        // Test shifting horizontal line bitboard
        let expected: u64 = 1095216660480;
        let piece_bit = 39;
        let piece_coordinates = get_piece_coordinates(piece_bit);
        let result = shift_direction_bitboard(piece_bit, piece_coordinates, &direction_bitboards::HORIZONTAL_LINE);
        assert_eq!(result, expected);
    }

}
