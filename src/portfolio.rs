use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Default, Serialize, Deserialize)]
pub struct Portfolio {
    pub stocks: HashMap<String, Stock>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub trades: Vec<Trade>,
}

#[derive(Serialize, Deserialize)]
pub struct Trade {
    pub quantity: u32,
    pub price: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub datetime: OffsetDateTime,
    pub kind: TradeKind,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeKind {
    Buy,
    Sell,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MonthSummary {
    pub profit: f64,
    pub sold_amount: f64,
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

    pub fn buy(&mut self, symbol: &str, quantity: u32, price: f64, datetime: OffsetDateTime) {
        let stock = self
            .stocks
            .entry(symbol.to_string())
            .or_insert_with(|| Stock::new(symbol.to_string()));

        stock.buy(quantity, price, datetime)
    }

    pub fn sell(
        &mut self,
        symbol: &str,
        quantity: u32,
        price: f64,
        datetime: OffsetDateTime,
    ) -> Result<f64> {
        let stock = self
            .stocks
            .get_mut(symbol)
            .context("Not enough shares to sell")?;

        stock.sell(quantity, price, datetime)
    }

    pub fn profit_by_month(&self, year: i32) -> [MonthSummary; 12] {
        let mut profit_by_month = [MonthSummary::default(); 12];

        for stock in self.stocks.values() {
            let stock_profit_by_month = stock.get_profit_by_month(year);

            for (month, summary) in stock_profit_by_month.into_iter().enumerate() {
                profit_by_month[month].profit += summary.profit;
                profit_by_month[month].sold_amount += summary.sold_amount;
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
    /// Dynamically calculate the total quantity of the stock at a given date.
    pub fn quantity(&self, date: OffsetDateTime) -> u32 {
        let mut quantity = 0;

        for trade in &self.trades {
            if trade.datetime >= date {
                break;
            }

            if trade.kind == TradeKind::Buy {
                quantity += trade.quantity;
            } else {
                quantity -= trade.quantity;
            }
        }

        quantity
    }

    /// Dynamically calculate the average purchase price of the stock at a given date.
    pub fn average_purchase_price(&self, date: OffsetDateTime) -> f64 {
        let mut quantity = 0;
        let mut average_purchase_price = 0.0;

        // We assume that the trades are sorted by date.
        for trade in &self.trades {
            if trade.datetime >= date {
                break;
            }

            if trade.kind == TradeKind::Buy {
                average_purchase_price = ((average_purchase_price * quantity as f64)
                    + (trade.price * trade.quantity as f64))
                    / (quantity + trade.quantity) as f64;
                quantity += trade.quantity;
            } else {
                quantity -= trade.quantity;
                if quantity == 0 {
                    // When the total quantity is 0, we have sold all the shares, which mean we need
                    // to reset the average_purchase_price back to 0.
                    average_purchase_price = 0.0;
                }
            }
        }

        average_purchase_price
    }

    pub fn buy(&mut self, quantity: u32, price: f64, datetime: OffsetDateTime) {
        let trade = Trade {
            quantity,
            price,
            datetime,
            kind: TradeKind::Buy,
        };

        self.add_trade(trade)
    }

    pub fn sell(&mut self, quantity: u32, price: f64, datetime: OffsetDateTime) -> Result<f64> {
        ensure!(
            quantity <= self.quantity(datetime),
            "Not enough shares to sell"
        );

        let trade = Trade {
            quantity,
            price,
            datetime,
            kind: TradeKind::Sell,
        };

        let profit = self.calculate_profit(&trade);

        self.add_trade(trade);

        Ok(profit)
    }

    fn calculate_profit(&self, trade: &Trade) -> f64 {
        let average_purchase_price = self.average_purchase_price(trade.datetime);

        (trade.price - average_purchase_price) * trade.quantity as f64
    }

    pub fn get_profit_by_month(&self, year: i32) -> [MonthSummary; 12] {
        let mut profit_by_month = [MonthSummary::default(); 12];

        for trade in &self.trades {
            if trade.kind != TradeKind::Sell {
                continue;
            }

            if trade.datetime.year() != year {
                continue;
            }

            let month = trade.datetime.month() as usize - 1;

            profit_by_month[month].sold_amount += trade.price * trade.quantity as f64;
            profit_by_month[month].profit += self.calculate_profit(trade);
        }

        profit_by_month
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);

        // We ensure that the trades are sorted by date so that we can iterate over all the trades
        // in chronological order.
        self.trades.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    }
}
