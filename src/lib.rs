#![allow(dead_code)]
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Portfolio {
    stocks: HashMap<String, TradeHistory>,
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            stocks: HashMap::new(),
        }
    }

    pub fn buy(&mut self, stock: &str, quantity: u32, value: f64) {
        let trade_history = self
            .stocks
            .entry(stock.into())
            .or_insert_with(TradeHistory::new);
        trade_history.add(Trade::new(TradeType::Buy, quantity, value));
    }

    pub fn sell(
        &mut self,
        stock: &str,
        quantity: u32,
        value: f64,
    ) -> Result<(), NotEnoughStockToSell> {
        if let Some(trade_history) = self.stocks.get_mut(stock) {
            match trade_history.quantity().cmp(&(quantity as i32)) {
                std::cmp::Ordering::Less => return Err(NotEnoughStockToSell),
                std::cmp::Ordering::Equal => {
                    // For now we remove the TradeHistory from the Portfolio when it reaches zero
                    // quantity
                    self.stocks.remove(stock).unwrap();
                }
                std::cmp::Ordering::Greater => {
                    trade_history.add(Trade::new(TradeType::Sell, quantity, value))
                }
            };

            return Ok(());
        }

        Err(NotEnoughStockToSell)
    }

    pub fn stock(&self, symbol: &str) -> Option<Stock> {
        if let Some(trade_history) = self.stocks.get(symbol) {
            let quantity = trade_history.quantity();

            if quantity < 0 {
                panic!(
                    "This is a bug in the Portfolio implementation, the trade history could never
                 contain a negative quantity"
                );
            }

            Some(Stock {
                symbol: symbol.into(),
                quantity: quantity as u32,
            })
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Stock {
    pub symbol: String,
    pub quantity: u32,
}

#[derive(Debug)]
struct Trade {
    trade_type: TradeType,
    quantity: u32,
    // The trade's unit value
    value: f64,
}

impl Trade {
    fn new(trade_type: TradeType, quantity: u32, value: f64) -> Self {
        Trade {
            trade_type,
            quantity,
            value,
        }
    }
}

#[derive(Debug, PartialEq)]
enum TradeType {
    Sell,
    Buy,
}

#[derive(Debug)]
struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    fn new() -> Self {
        TradeHistory { trades: Vec::new() }
    }

    fn with_trades(trades: Vec<Trade>) -> Self {
        TradeHistory { trades }
    }

    fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    fn quantity(&self) -> i32 {
        self.trades
            .iter()
            .map(|trade| match trade.trade_type {
                TradeType::Sell => -(trade.quantity as i32),
                TradeType::Buy => (trade.quantity as i32),
            })
            .sum()
    }

    fn average_price(&self) -> f64 {
        let value: f64 = self
            .trades
            .iter()
            .filter(|trade| trade.trade_type == TradeType::Buy)
            .map(|trade| trade.value * trade.quantity as f64)
            .sum();

        let quantity: u32 = self
            .trades
            .iter()
            .filter(|trade| trade.trade_type == TradeType::Buy)
            .map(|trade| trade.quantity)
            .sum();

        value / (quantity as f64)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotEnoughStockToSell;

#[cfg(test)]
mod tests {
    use crate::*;

    fn setup_trade_history() -> TradeHistory {
        let trades = vec![
            Trade::new(TradeType::Buy, 100, 18.43),
            Trade::new(TradeType::Buy, 75, 10.33),
            Trade::new(TradeType::Sell, 50, 9.20),
        ];

        TradeHistory::with_trades(trades)
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
}
