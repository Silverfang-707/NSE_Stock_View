use chrono::NaiveDate;

#[derive(Debug)]

pub struct Level {

    pub symbol: String,

    pub series: String,

    pub trade_date: NaiveDate,

    pub open_price: f64,

    pub high_price: f64,

    pub low_price: f64,

    pub close_price: f64,

    pub range_value: f64,

    pub buffer_value: f64,

    pub jgd: f64,

    pub jwd: f64,

    pub bdp: f64,

    pub wdp: f64,

    pub pattern: String,
}