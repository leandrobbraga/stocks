#![allow(dead_code)]
mod tests;

#[derive(Debug)]
pub struct Stock {
    symbol: String,
    trade_history: TradeHistory,
}

impl Stock {
    pub fn new(symbol: String) -> Self {
        let trades: Vec<Trade> = Vec::new();
        let trade_history = TradeHistory::new(trades);

        Stock {
            symbol,
            trade_history,
        }
    }

    pub fn with_trade_history(symbol: String, trade_history: TradeHistory) -> Self {
        Stock {
            symbol,
            trade_history,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TradeType {
    SELL,
    BUY,
}

#[derive(Debug)]
pub struct Trade {
    trade_type: TradeType,
    quantity: u32,
    value: f64,
}

impl Trade {
    pub fn new(trade_type: TradeType, quantity: u32, value: f64) -> Self {
        Trade {
            trade_type,
            quantity,
            value,
        }
    }
}

#[derive(Debug)]
pub struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    pub fn new(trades: Vec<Trade>) -> Self {
        TradeHistory { trades }
    }

    pub fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn value(&self) -> f64 {
        self.trades
            .iter()
            .map(|trade| match trade.trade_type {
                TradeType::SELL => -(trade.quantity as f64 * trade.value),
                TradeType::BUY => (trade.quantity as f64 * trade.value),
            })
            .sum()
    }

    pub fn quantity(&self) -> i32 {
        self.trades
            .iter()
            .map(|trade| match trade.trade_type {
                TradeType::SELL => -(trade.quantity as i32),
                TradeType::BUY => (trade.quantity as i32),
            })
            .sum()
    }

    pub fn average_price(&self) -> f64 {
        let value: f64 = self
            .trades
            .iter()
            .filter(|trade| trade.trade_type == TradeType::BUY)
            .map(|trade| trade.value * trade.quantity as f64)
            .sum();

        let quantity: u32 = self
            .trades
            .iter()
            .filter(|trade| trade.trade_type == TradeType::BUY)
            .map(|trade| trade.quantity)
            .sum();

        value / (quantity as f64)
    }
}

struct Portfolio {}
