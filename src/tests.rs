#![cfg(test)]
use crate::*;

fn setup_trade_history() -> TradeHistory {
    let trades = vec![
        Trade::new(TradeType::BUY, 100, 18.43),
        Trade::new(TradeType::BUY, 75, 10.33),
        Trade::new(TradeType::SELL, 50, 9.20),
    ];

    TradeHistory::new(trades)
}

#[test]
fn test_trade_history_value() {
    let trade_history = setup_trade_history();
    let error_margin = f64::EPSILON;

    assert!((trade_history.value() - 2157.75).abs() < error_margin);
}

#[test]
fn test_trade_history_quantity() {
    let trade_history = setup_trade_history();
    assert_eq!(trade_history.quantity(), 125);
}

#[test]
fn test_trade_history_average_price() {
    let trade_history = setup_trade_history();
    let error_margin = f64::EPSILON;

    assert!((trade_history.average_price() - 14.958571428571428).abs() < error_margin);
}
