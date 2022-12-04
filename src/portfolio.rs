use anyhow::Result;
use chrono::Datelike;
use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub struct Portfolio {
    pub stocks: HashMap<String, Stock>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub quantity: u32,
    pub average_purchase_price: f64,
    pub trades: Vec<Trade>,
}

#[derive(Serialize, Deserialize)]
pub struct Trade {
    pub quantity: u32,
    pub price: f64,
    pub date: NaiveDate,
    pub kind: TradeKind,
    pub profit: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeKind {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum TradeError {
    NotEnoughShares,
    OutOfOrderTrade,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            stocks: HashMap::new(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let file = std::fs::File::create("portfolio.json")?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let file = std::fs::File::open("portfolio.json")?;
        let portfolio = serde_json::from_reader(file)?;
        Ok(portfolio)
    }

    pub fn buy(
        &mut self,
        symbol: &str,
        quantity: u32,
        price: f64,
        date: NaiveDate,
    ) -> Result<(), TradeError> {
        let stock = self
            .stocks
            .entry(symbol.to_string())
            .or_insert_with(|| Stock::new(symbol.to_string()));

        stock.buy(quantity, price, date)
    }

    pub fn sell(
        &mut self,
        symbol: &str,
        quantity: u32,
        price: f64,
        date: NaiveDate,
    ) -> Result<f64, TradeError> {
        let stock = self
            .stocks
            .get_mut(symbol)
            .ok_or(TradeError::NotEnoughShares)?;

        let profit = stock.sell(quantity, price, date)?;

        Ok(profit)
    }

    pub fn stocks(&self) -> Vec<&Stock> {
        self.stocks.values().collect()
    }

    pub fn profit_by_month(&self, year: i32) -> Vec<f64> {
        let mut profit_by_month = vec![0.0; 12];

        for stock in self.stocks() {
            let stock_profit_by_month = stock.get_profit_by_month(year);
            for (i, profit) in stock_profit_by_month.iter().enumerate() {
                profit_by_month[i] += profit;
            }
        }

        profit_by_month
    }
}

impl Stock {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            ..Default::default()
        }
    }

    pub fn buy(&mut self, quantity: u32, price: f64, date: NaiveDate) -> Result<(), TradeError> {
        let trade = Trade {
            quantity,
            price,
            date,
            kind: TradeKind::Buy,
            profit: 0.0,
        };

        self.add_trade(trade)?;

        self.average_purchase_price = ((self.average_purchase_price * self.quantity as f64)
            + (price * quantity as f64))
            / (self.quantity + quantity) as f64;
        self.quantity += quantity;

        Ok(())
    }

    pub fn sell(&mut self, quantity: u32, price: f64, date: NaiveDate) -> Result<f64, TradeError> {
        if quantity > self.quantity {
            return Err(TradeError::NotEnoughShares);
        }

        let profit = (price - self.average_purchase_price) * quantity as f64;

        let trade = Trade {
            quantity,
            price,
            date,
            kind: TradeKind::Sell,
            profit,
        };

        self.add_trade(trade)?;

        self.quantity -= quantity;

        if self.quantity == 0 {
            self.average_purchase_price = 0.0;
        }

        Ok(profit)
    }

    pub fn get_profit_by_month(&self, year: i32) -> Vec<f64> {
        let mut profit_by_month = vec![0.0; 12];

        for trade in &self.trades {
            if trade.date.year() == year {
                profit_by_month[trade.date.month() as usize - 1] += trade.profit;
            }
        }

        profit_by_month
    }

    fn add_trade(&mut self, trade: Trade) -> Result<(), TradeError> {
        // Ensure this trade has the most recent date from the whole trade history
        let last_date = self.trades.iter().map(|trade| trade.date).max();

        if let Some(last_date) = last_date {
            if trade.date < last_date {
                return Err(TradeError::OutOfOrderTrade);
            }
        }

        self.trades.push(trade);

        Ok(())
    }
}
