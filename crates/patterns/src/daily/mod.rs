use crate::models::analysis::{
    DailyPlan,
    PriceLevel,
};

pub fn generate_daily_plan(
    symbol: String,
    high: f64,
    low: f64,
    close: f64,
) -> DailyPlan {

    let pivot =
        (high + low + close) / 3.0;

    let range =
        high - low;

    let r1 =
        pivot + range * 0.382;

    let r2 =
        pivot + range * 0.618;

    let r3 =
        pivot + range;

    let s1 =
        pivot - range * 0.382;

    let s2 =
        pivot - range * 0.618;

    let s3 =
        pivot - range;

    DailyPlan {

        symbol,

        pivot,

        resistance_levels: vec![

            PriceLevel {
                label: "R1".into(),
                value: r1,
            },

            PriceLevel {
                label: "R2".into(),
                value: r2,
            },

            PriceLevel {
                label: "R3".into(),
                value: r3,
            },
        ],

        support_levels: vec![

            PriceLevel {
                label: "S1".into(),
                value: s1,
            },

            PriceLevel {
                label: "S2".into(),
                value: s2,
            },

            PriceLevel {
                label: "S3".into(),
                value: s3,
            },
        ],

        high,
        low,
        close,
    }
}