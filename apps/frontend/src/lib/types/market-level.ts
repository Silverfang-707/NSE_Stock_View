export interface MarketLevel {

    timeframe: string;

    trade_date: string;

    open_price: number;

    high_price: number;

    low_price: number;

    close_price: number;

    range_value: number;

    buffer_value: number;

    jgd: number;

    jwd: number;

    bdp: number;

    wdp: number;

    pattern: string;

    // =====================
    // LEGACY PATTERN SUPPORT
    // =====================

    prev_jgd?: number;

    prev_jwd?: number;

    prev_bdp?: number;

    prev_wdp?: number;
}