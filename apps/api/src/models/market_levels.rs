use serde::Serialize;

#[derive(Debug, Serialize)]

pub struct MarketLevelResponse {

    pub timeframe: String,

    pub trade_date: String,

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

    // =========================
    // LEGACY PATTERN SUPPORT
    // =========================

    pub prev_jgd: Option<f64>,

    pub prev_jwd: Option<f64>,

    pub prev_bdp: Option<f64>,

    pub prev_wdp: Option<f64>,
}