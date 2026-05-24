use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct OhlcRow {
    pub trade_date:  NaiveDate,
    pub open_price:  f64,
    pub high_price:  f64,
    pub low_price:   f64,
    pub close_price: f64,
}