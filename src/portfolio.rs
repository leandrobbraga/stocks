#![allow(dead_code)]
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Default, PartialEq, Eq)]
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

impl ToString for Portfolio {
    fn to_string(&self) -> String {
        let mut buffer = String::new();

        for (stock, trade_history) in &self.stocks {
            buffer.push_str(stock);
            buffer.push('!');
            buffer.push_str(&trade_history.to_string());
            buffer.push('#');
        }

        buffer.pop();

        buffer
    }
}

impl FromStr for Portfolio {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.split('#').collect();

        let mut stocks: HashMap<String, TradeHistory> = HashMap::new();

        for serialized_stock in s {
            let serialized_stock: Vec<&str> = serialized_stock.split('!').collect();

            if serialized_stock.len() != 2 {
                return Err(ParseError);
            }

            let stock = String::from(serialized_stock[0]);
            let trade_history = match TradeHistory::from_str(serialized_stock[1]) {
                Ok(trade_history) => trade_history,
                Err(_) => return Err(ParseError),
            };

            stocks.insert(stock, trade_history);
        }

        Ok(Portfolio { stocks })
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

#[derive(Debug, PartialEq, Eq)]
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

impl ToString for Trade {
    fn to_string(&self) -> String {
        format!("{}|{}", self.quantity, self.value)
    }
}

impl FromStr for Trade {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.split('|').collect();

        if s.len() != 2 {
            return Err(ParseError);
        }

        let quantity: i32 = match s[0].parse() {
            Ok(value) => value,
            Err(_) => return Err(ParseError),
        };

        let value: u32 = match s[1].parse() {
            Ok(value) => value,
            Err(_) => return Err(ParseError),
        };

        Ok(Trade { quantity, value })
    }
}

#[derive(Debug)]
pub struct ParseError;

#[derive(Debug, Default, PartialEq, Eq)]
struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
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

impl ToString for TradeHistory {
    fn to_string(&self) -> String {
        let mut buffer = String::new();

        for trade in &self.trades {
            buffer.push_str(&trade.to_string());
            buffer.push(';');
        }

        buffer.pop();

        buffer
    }
}

impl FromStr for TradeHistory {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.split(';').collect();

        let mut trades: Vec<Trade> = Vec::new();

        for serialized_trade in s {
            let trade = match Trade::from_str(serialized_trade) {
                Ok(trade) => trade,
                Err(_) => return Err(ParseError),
            };

            trades.push(trade)
        }

        Ok(TradeHistory { trades })
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

        TradeHistory { trades }
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
    #[test]
    fn test_trade_serialize_deserialize() {
        let trade = Trade::new(100, 50);

        let serialized = trade.to_string();
        let deserialized_trade = Trade::from_str(&serialized).unwrap();

        assert_eq!(trade, deserialized_trade)
    }

    #[test]
    fn test_trade_history_serialize_deserialize() {
        let trade_history = setup_trade_history();

        let serialized = trade_history.to_string();

        println!("{}", serialized);
        let deserialized_trade = TradeHistory::from_str(&serialized).unwrap();

        assert_eq!(trade_history, deserialized_trade)
    }
}
