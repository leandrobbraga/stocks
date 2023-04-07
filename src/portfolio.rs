use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Serialize, Deserialize)]
pub struct Portfolio {
    pub stocks: HashMap<String, Stock>,
}

#[derive(Serialize, Deserialize)]
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
    /// The list of splits allow us to adjust the stock quantity according to the reference date
    /// (i.e. if the reference date is before or after the split date).
    pub splits: Vec<Split>,
}

#[derive(Serialize, Deserialize)]
pub struct Split {
    pub ratio: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub datetime: OffsetDateTime,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeKind {
    Buy,
    Sell,
}

#[derive(Default)]
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

    pub fn split(&mut self, symbol: &str, ratio: f64, datetime: OffsetDateTime) {
        let stock = self
            .stocks
            .entry(symbol.to_string())
            .or_insert_with(|| Stock::new(symbol.to_string()));

        stock.split(ratio, datetime);
    }

    pub fn buy(&mut self, symbol: &str, quantity: u32, price: f64, datetime: OffsetDateTime) {
        let stock = self
            .stocks
            .entry(symbol.to_string())
            .or_insert_with(|| Stock::new(symbol.to_string()));

        stock.buy(quantity, price, datetime);
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
        let mut profit_by_month: [MonthSummary; 12] = Default::default();

        for stock in self.stocks.values() {
            stock.update_profit_by_month(&mut profit_by_month, year);
        }

        profit_by_month
    }
}

impl Stock {
    fn new(symbol: String) -> Self {
        Self {
            symbol,
            trades: vec![],
        }
    }

    fn split(&mut self, ratio: f64, datetime: OffsetDateTime) {
        for trade in &mut self.trades {
            if trade.datetime >= datetime {
                break;
            }

            trade.splits.push(Split { ratio, datetime });
        }
    }

    /// Dynamically calculate the total quantity of the stock at a given date.
    pub fn quantity(&self, date: OffsetDateTime) -> u32 {
        let mut quantity = 0;

        for trade in &self.trades {
            if trade.datetime >= date {
                break;
            }

            let signal = if trade.kind == TradeKind::Buy {
                1.0
            } else {
                -1.0
            };

            quantity += (trade.quantity(date) as f64 * signal) as i32;
        }

        quantity as u32
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
                average_purchase_price = ((average_purchase_price * f64::from(quantity))
                    + (trade.price(date) * trade.quantity(date) as f64))
                    / f64::from(quantity + trade.quantity(date));
                quantity += trade.quantity(date);
            } else {
                quantity -= trade.quantity(date);
                if quantity == 0 {
                    // When the total quantity is 0, we have sold all the shares, which mean we need
                    // to reset the average_purchase_price back to 0.
                    average_purchase_price = 0.0;
                }
            }
        }

        average_purchase_price
    }

    fn buy(&mut self, quantity: u32, price: f64, datetime: OffsetDateTime) {
        let trade = Trade {
            quantity,
            price,
            datetime,
            kind: TradeKind::Buy,
            splits: vec![],
        };

        self.add_trade(trade);
    }

    fn sell(&mut self, quantity: u32, price: f64, datetime: OffsetDateTime) -> Result<f64> {
        ensure!(
            quantity <= self.quantity(datetime),
            "Not enough shares to sell"
        );

        let trade = Trade {
            quantity,
            price,
            datetime,
            kind: TradeKind::Sell,
            splits: vec![],
        };

        let profit = self.calculate_profit(&trade);

        self.add_trade(trade);

        Ok(profit)
    }

    fn calculate_profit(&self, trade: &Trade) -> f64 {
        let average_purchase_price = self.average_purchase_price(trade.datetime);

        (trade.price - average_purchase_price) * f64::from(trade.quantity)
    }

    fn update_profit_by_month(&self, profit_by_month: &mut [MonthSummary; 12], year: i32) {
        for trade in &self.trades {
            if trade.kind != TradeKind::Sell {
                continue;
            }

            if trade.datetime.year() != year {
                continue;
            }

            let month = trade.datetime.month() as usize - 1;

            profit_by_month[month].sold_amount += trade.price * f64::from(trade.quantity);
            profit_by_month[month].profit += self.calculate_profit(trade);
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);

        // We ensure that the trades are sorted by date so that we can iterate over all the trades
        // in chronological order.
        self.trades.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    }
}

impl Trade {
    fn quantity(&self, datetime: OffsetDateTime) -> u32 {
        let split_ratio = self
            .splits
            .iter()
            .filter(|split| split.datetime < datetime)
            .fold(1.0, |acc, split| acc * split.ratio);

        (self.quantity as f64 * split_ratio) as u32
    }

    fn price(&self, datetime: OffsetDateTime) -> f64 {
        let split_ratio = self
            .splits
            .iter()
            .filter(|split| split.datetime < datetime)
            .fold(1.0, |acc, split| acc * split.ratio);

        self.price / split_ratio
    }
}
