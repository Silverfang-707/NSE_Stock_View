pub fn calculate_buffer(
    close_price: f64,
    range_value: f64,
) -> f64 {
    (close_price * 0.00073) + (range_value * 0.073)
}