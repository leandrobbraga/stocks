mod commands;
#[macro_use]
mod log;
mod render;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use clap::Parser;
use stocks::portfolio::Portfolio;
use stocks::stock_market::StockMarket;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    /// Buys an asset
    Buy {
        /// The ticker of the stock (e.g. BBAS3)
        symbol: String,
        /// How many stocks were purchased (e.g. 100)
        quantity: u32,
        /// How much was the average cost of the purchase (e.g. 33.21)
        price: f64,
        #[clap(value_parser=parse_datetime)]
        #[arg(default_value_t = chrono::Local::now().naive_local())]
        datetime: NaiveDateTime,
    },
    /// Sells an asset
    Sell {
        /// The ticker of the stock (e.g. BBAS3)
        symbol: String,
        /// How many stocks was sold (e.g. 100)
        quantity: u32,
        /// How much was the average cost of the sell (e.g. 33.21)
        price: f64,
        #[clap(value_parser=parse_datetime)]
        #[arg(default_value_t = chrono::Local::now().naive_local())]
        datetime: NaiveDateTime,
    },
    /// Print a summary of the portfolio
    Summary {
        /// The reference date of the output summary (e.g. 2020-12-31 means that the summary will
        /// show all the assets in the portfolio as of 2020-12-31, inclusive)
        #[clap(value_parser=parse_date)]
        #[arg(default_value_t = chrono::Local::now().date_naive())]
        date: NaiveDate,
    },
    ProfitSummary {
        #[arg(default_value_t = chrono::Local::now().date_naive().year())]
        year: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = Command::parse();

    let mut portfolio = Portfolio::load().unwrap_or_else(|err| {
        warn!("Could not load portfolio: {err}");
        info!("Creating a new portfolio.");
        Portfolio::new()
    });

    match command {
        Command::Buy {
            symbol,
            quantity,
            price,
            datetime,
        } => commands::buy(
            &mut portfolio,
            &symbol.to_uppercase(),
            quantity,
            price,
            datetime,
        ),
        Command::Sell {
            symbol,
            quantity,
            price,
            datetime,
        } => commands::sell(
            &mut portfolio,
            &symbol.to_uppercase(),
            quantity,
            price,
            datetime,
        ),
        Command::Summary { date } => {
            let stock_market = StockMarket::new();
            commands::summarize(&portfolio, &stock_market, date).await
        }
        Command::ProfitSummary { year } => {
            let year = u16::try_from(year)?;
            commands::profit_summary(&portfolio, year)
        }
    }
}

fn parse_datetime(arg: &str) -> Result<NaiveDateTime> {
    // The default `to_string` from NaiveDateTime contains the fraction of second, but it would be
    // cumbersome to ask the user to provide it. So we just remove it.
    let arg = arg.split('.').next().unwrap();
    Ok(NaiveDateTime::parse_from_str(arg, "%Y-%m-%d %H:%M:%S")?)
}

fn parse_date(arg: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(arg, "%Y-%m-%d")?)
}
