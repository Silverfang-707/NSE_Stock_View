// patterns.rs

pub fn detect_pattern(
    jgd:      f64,
    jwd:      f64,
    prev_jwd: f64,
) -> String {

    if jgd > prev_jwd && jwd > prev_jwd {
        "2+2".to_string()
    } else if jgd > prev_jwd && jwd < prev_jwd {
        "2+1".to_string()
    } else {
        "3+1".to_string()
    }
}

pub fn update_bdp_wdp(
    pattern:  &str,
    jgd:      f64,
    jwd:      f64,
    prev_bdp: f64,
) -> (f64, f64) {
    match pattern {
        "2+2" | "2+1" => (jgd, jwd),
        _              => (prev_bdp, jwd),
    }
}