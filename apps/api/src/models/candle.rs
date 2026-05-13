use chrono::NaiveDate;
use serde::Serialize;

#[derive(Serialize)]
pub struct CandleResponse {
    pub trade_date: NaiveDate,

    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub close_price: f64,

    pub volume: i64,
    pub series: String,
}