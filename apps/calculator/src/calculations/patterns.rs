// patterns.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    TwoTwo,
    TwoOne,
    ThreeOne,
    None, // Used for the very first period where there is no previous data
}

impl Pattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            Pattern::TwoTwo => "2+2",
            Pattern::TwoOne => "2+1",
            Pattern::ThreeOne => "3+1",
            Pattern::None => "",
        }
    }
}

pub fn detect_pattern(
    jgd:      f64,
    jwd:      f64,
    prev_jwd: f64,
) -> Pattern {
    if jgd > prev_jwd && jwd > prev_jwd {
        Pattern::TwoTwo
    } else if jgd > prev_jwd && jwd < prev_jwd {
        Pattern::TwoOne
    } else {
        Pattern::ThreeOne
    }
}

pub fn update_bdp_wdp(
    pattern:  Pattern,
    jgd:      f64,
    jwd:      f64,
    prev_bdp: f64,
) -> (f64, f64) {
    match pattern {
        Pattern::TwoTwo | Pattern::TwoOne => (jgd, jwd),
        _ => (prev_bdp, jwd),
    }
}