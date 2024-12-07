// Scales input to a floating point number between 0.0 and 1.0
// No clamp enforces the limits, so if a input is provided outside the range larger values can be expected
pub fn f32_scale(input: f32, input_min: f32, input_max: f32) -> f32 {
    (input - input_min) / (input_max - input_min)
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_f32_scale() {
        assert_eq!(f32_scale(10.0, -10.0, 30.0), 0.5);
        assert_eq!(f32_scale(30.0, -10.0, 30.0), 1.0);
        assert_eq!(f32_scale(39.0, 0.0, 39.0), 1.0);
    }
}