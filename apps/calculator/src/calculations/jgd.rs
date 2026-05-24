pub fn calculate_jgd(
    high_price:  f64,
    range_value: f64,
) -> f64 {
    high_price - (range_value * 0.382)
}