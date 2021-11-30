#![allow(dead_code)]
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Portfolio {
    stocks: HashMap<String, TradeHistory>,
}

impl Portfolio {
    pub fn buy(&mut self, stock: &str, quantity: u32, value: u32) {
        let trade_history = self
            .stocks
            .entry(stock.into())
            .or_insert_with(TradeHistory::default);
        trade_history.add(Trade::new(quantity as i32, value));
    }

    pub fn sell(
        &mut self,
        stock: &str,
        quantity: u32,
        value: u32,
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
                    trade_history.add(Trade::new(-(quantity as i32), value))
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

impl Stock {
    pub fn new(symbol: &str, quantity: u32) -> Stock {
        Stock {
            symbol: symbol.into(),
            quantity,
        }
    }
}

#[derive(Debug)]
struct Trade {
    quantity: i32,
    // The trade's unit value in cents
    value: u32,
}

impl Trade {
    fn new(quantity: i32, value: u32) -> Self {
        Trade { quantity, value }
    }
}

#[derive(Debug, Default)]
struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    fn with_trades(trades: Vec<Trade>) -> Self {
        TradeHistory { trades }
    }

    fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    fn quantity(&self) -> i32 {
        self.trades.iter().map(|trade| trade.quantity as i32).sum()
    }

    fn average_price(&self) -> f64 {
        let value: u32 = self
            .trades
            .iter()
            .filter(|trade| trade.quantity > 0)
            .map(|trade| trade.value * trade.quantity as u32)
            .sum();

        let quantity: u32 = self
            .trades
            .iter()
            .filter(|trade| trade.quantity > 0)
            .map(|trade| trade.quantity as u32)
            .sum();

        (value as f64) / (quantity as f64)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotEnoughStockToSell;

#[cfg(test)]
mod tests {
    use crate::portfolio::*;

    fn setup_trade_history() -> TradeHistory {
        let trades = vec![
            Trade::new(100, 1843),
            Trade::new(75, 1033),
            Trade::new(-50, 920),
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

        assert!((trade_history.average_price() - 1495.857142857143).abs() < error_margin);
    }
}
