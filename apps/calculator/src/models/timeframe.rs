#[derive(Debug, Clone)]
pub enum Timeframe {

    Daily,

    Weekly,

    Monthly,

    Quarterly,

    HalfYearly,

    Yearly,
}

impl Timeframe {

    pub fn as_str(&self) -> &'static str {

        match self {

            Self::Daily =>
                "daily",

            Self::Weekly =>
                "weekly",

            Self::Monthly =>
                "monthly",

            Self::Quarterly =>
                "quarterly",

            Self::HalfYearly =>
                "half_yearly",

            Self::Yearly =>
                "yearly",
        }
    }

    pub fn trunc_str(&self) -> &'static str {

        match self {

            Self::Daily =>
                "day",

            Self::Weekly =>
                "week",

            Self::Monthly =>
                "month",

            Self::Quarterly =>
                "quarter",

            Self::HalfYearly =>
                "half_yearly",

            Self::Yearly =>
                "year",
        }
    }
}