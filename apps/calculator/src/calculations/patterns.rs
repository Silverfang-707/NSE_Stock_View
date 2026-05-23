pub fn detect_pattern(

    open_price: f64,

    close_price: f64,

)
-> String
{

    if close_price > open_price {

        "BULLISH".to_string()
    }

    else if close_price < open_price {

        "BEARISH".to_string()
    }

    else {

        "NEUTRAL".to_string()
    }
}