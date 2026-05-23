#[derive(Clone, Copy)]

pub enum Timeframe {

    Daily,

    Weekly,

    Monthly,

    Quarterly,

    HalfYearly,

    Yearly,
}

impl Timeframe {

    pub fn interval(&self) -> &'static str {

        match self {

            Timeframe::Daily =>
                "1 day",

            Timeframe::Weekly =>
                "7 days",

            Timeframe::Monthly =>
                "1 month",

            Timeframe::Quarterly =>
                "3 months",

            Timeframe::HalfYearly =>
                "6 months",

            Timeframe::Yearly =>
                "1 year",
        }
    }
}