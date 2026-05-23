use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PriceLevel {
    pub value: f64,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct DailyPlan {

    pub symbol: String,

    pub pivot: f64,

    pub resistance_levels: Vec<PriceLevel>,

    pub support_levels: Vec<PriceLevel>,

    pub high: f64,

    pub low: f64,

    pub close: f64,
}