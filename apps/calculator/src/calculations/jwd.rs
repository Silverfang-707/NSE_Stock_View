pub fn calculate_jwd(
    low_price:   f64,
    range_value: f64,
) -> f64 {
    low_price + (range_value * 0.382)
}