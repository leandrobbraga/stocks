use crate::render::{render_profit_by_month, render_summary, ProfitSummaryData, SummaryData};
use anyhow::Result;
use chrono::NaiveDate;
use log::{error, info, warn};
use stocks::{
    portfolio::{Portfolio, Stock},
    stock_market::{PricedStock, StockMarket},
};

pub struct App {
    portfolio: Portfolio,
    stock_market: StockMarket,
}

impl App {
    pub fn new(portfolio: Portfolio, stock_market: StockMarket) -> Self {
        Self {
            portfolio,
            stock_market,
        }
    }

    pub fn buy(&mut self, symbol: &str, quantity: u32, price: f64, date: NaiveDate) {
        if let Err(e) = self.portfolio.buy(symbol, quantity, price, date) {
            error!("Could not buy {symbol} because {e:?}",);
            std::process::exit(1)
        }

        info!(
            "You bought {quantity} {symbol} at R${price:10.2}.",
            quantity = quantity,
            symbol = symbol,
            price = price
        );

        match self.portfolio.save() {
            Ok(_) => {}
            Err(e) => {
                error!("Could not save portfolio: {e:?}", e = e);
                std::process::exit(1)
            }
        }
    }

    pub fn sell(&mut self, symbol: &str, quantity: u32, price: f64, date: NaiveDate) {
        match self.portfolio.sell(symbol, quantity, price, date) {
            Ok(profit) => info!(
                "You sold {quantity} {symbol} profiting R${profit:10.2}.",
                quantity = quantity,
                symbol = symbol,
                profit = profit
            ),
            Err(e) => {
                error!("Could not sell {symbol} because {e:?}",);
                std::process::exit(1)
            }
        }

        match self.portfolio.save() {
            Ok(_) => (),
            Err(e) => {
                error!("Could not save portfolio: {e:?}", e = e);
                std::process::exit(1)
            }
        }
    }

    pub async fn summarize(&self) -> Result<()> {
        let stocks: Vec<&Stock> = self
            .portfolio
            .stocks
            .values()
            .filter(|stock| stock.quantity > 0)
            .collect();

        let priced_stocks = self.stock_market.get_stock_prices(&stocks).await?;

        let mut data = Vec::with_capacity(priced_stocks.len());
        for priced_stock in priced_stocks {
            match priced_stock {
                Ok(stock) => data.push(stock.into()),
                Err(e) => {
                    warn!("Could not fetch stock price: {e:?}", e = e);
                }
            }
        }

        if let Err(e) = render_summary(data) {
            error!("Could not render table: {e:?}", e = e);
            std::process::exit(1)
        };

        Ok(())
    }

    pub fn profit_summary(&self, year: u16) {
        let profit_by_month = self.portfolio.profit_by_month(year as i32);

        let mut data = Vec::with_capacity(12);

        for (month, summary) in profit_by_month.iter().enumerate() {
            data.push(ProfitSummaryData {
                month: month as u8,
                sold_amount: summary.sold_amount,
                profit: summary.profit,
            })
        }

        if let Err(e) = render_profit_by_month(data) {
            error!("Could not render table: {e:?}", e = e);
            std::process::exit(1)
        };
    }
}

impl From<PricedStock> for SummaryData {
    fn from(stock: PricedStock) -> Self {
        let current_value = stock.price * stock.quantity as f64;
        let last_value = stock.last_price * stock.quantity as f64;
        let original_cost = stock.quantity as f64 * stock.average_price;

        Self {
            name: stock.symbol,
            quantity: stock.quantity,
            current_price: stock.price,
            current_value,
            change: current_value - last_value,
            change_percentage: (current_value / last_value - 1.0) * 100.0,
            average_price: stock.average_price,
            profit: current_value - original_cost,
            profit_percentage: (current_value / original_cost - 1.0) * 100.0,
            last_value,
            original_cost,
        }
    }
}
