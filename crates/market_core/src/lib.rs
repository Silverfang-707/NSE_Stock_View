use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Candle {

    #[serde(rename = "TckrSymb")]
    pub symbol: String,

    #[serde(rename = "SctySrs")]
    pub series: String,

    #[serde(rename = "TradDt")]
    pub trade_date: NaiveDate,

    #[serde(rename = "OpnPric")]
    pub open_price: f64,

    #[serde(rename = "HghPric")]
    pub high_price: f64,

    #[serde(rename = "LwPric")]
    pub low_price: f64,

    #[serde(rename = "ClsPric")]
    pub close_price: f64,

    #[serde(rename = "TtlTradgVol")]
    pub volume: i64,

}

pub mod ingest;
