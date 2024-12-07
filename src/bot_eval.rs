
// Weights
// All weights are percentages, so the values add to 1
const MATERIAL_WEIGHT: f32 = 0.7;
const SQUARE_TABLE_WEIGHT: f32 = 0.3;

use crate::board_representation::Board;
use crate::pesto;
use crate::generic_math;

// Square tables encourage good mobility anyway, I think actually calculating the mobility would be too computationally expensive
// const MOBILITY_WEIGHT: f32 = 0.2;

// Basic evaluation function
// Called by leaf nodes during minimax search
// Only use material change from the starting position, to the board at the leaf node
// and a piece square table value
pub fn eval(material_change: i8, board: &Board) -> f32 {
    let square_table_value = pesto::get_table_value(board);
    let material_value = generic_math::f32_scale(material_change as f32, -20.0, 20.0);

    material_value * MATERIAL_WEIGHT + square_table_value * SQUARE_TABLE_WEIGHT
}