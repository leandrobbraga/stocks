use crate::render::{render_profit_by_month, render_summary, ProfitSummaryData, SummaryData};
use crate::{info, warn};
use anyhow::Result;
use stocks::{
    portfolio::{Portfolio, Stock},
    stock_market::{PricedStock, StockMarket},
};
use time::{Date, OffsetDateTime, UtcOffset};

pub fn buy(
    portfolio: &mut Portfolio,
    symbol: &str,
    quantity: u32,
    price: f64,
    datetime: OffsetDateTime,
) -> Result<()> {
    portfolio.buy(symbol, quantity, price, datetime);

    info!("You bought {quantity} {symbol} at R${price:10.2}.");

    portfolio.save()
}

pub fn sell(
    portfolio: &mut Portfolio,
    symbol: &str,
    quantity: u32,
    price: f64,
    datetime: OffsetDateTime,
) -> Result<()> {
    let profit = portfolio.sell(symbol, quantity, price, datetime)?;

    info!("You sold {quantity} {symbol} profiting R${profit:10.2}.");

    portfolio.save()
}

pub fn summarize(portfolio: &Portfolio, stock_market: &StockMarket, date: Date) {
    let date = date
        .with_time(time::Time::from_hms(23, 59, 59).expect("BUG: Should be a valid time"))
        .assume_offset(UtcOffset::UTC);

    let stocks: Vec<&Stock> = portfolio
        .stocks
        .values()
        // To ensure that we only show stocks that we own
        .filter(|stock| stock.quantity(date) > 0)
        .collect();

    let priced_stocks = stock_market.get_stock_prices(&stocks, date);
    let stock_count = priced_stocks.len();
    let data: Vec<SummaryData> = priced_stocks
        .into_iter()
        .filter_map(|maybe_stock| maybe_stock.map(|stock| stock.into()).ok())
        .collect();

    if stock_count > data.len() {
        warn!("Could not get prices for all stocks");
    }

    render_summary(data)
}

pub fn profit_summary(portfolio: &Portfolio, year: u16) {
    let profit_by_month = portfolio.profit_by_month(year as i32);

    let mut data = Vec::with_capacity(12);

    for (month, summary) in profit_by_month.iter().enumerate() {
        let tax = if summary.sold_amount > 20000.0 && summary.profit > 0.0 {
            summary.profit * 0.15
        } else {
            0.0
        };

        data.push(ProfitSummaryData {
            month: month as u8,
            sold_amount: summary.sold_amount,
            profit: summary.profit,
            tax,
        })
    }

    render_profit_by_month(data)
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
