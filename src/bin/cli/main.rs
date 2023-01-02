mod app;
mod render;

use anyhow::{Context, Result};
use app::App;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{error, info, warn};
use stocks::portfolio::Portfolio;
use stocks::stock_market::StockMarket;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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
    setup_logger();

    let command = Arguments::parse().command;

    let portfolio = match Portfolio::load() {
        Ok(portfolio) => portfolio,
        Err(e) => {
            warn!("Could not load portfolio: {e:?}", e = e);
            info!("Creating a new portfolio.");
            Portfolio::new()
        }
    };
    let stock_market = StockMarket::new();

    let mut app = App::new(portfolio, stock_market);

    match command {
        Command::Buy {
            symbol,
            quantity,
            price,
            datetime,
        } => app.buy(&symbol.to_uppercase(), quantity, price, datetime),
        Command::Sell {
            symbol,
            quantity,
            price,
            datetime,
        } => app.sell(&symbol.to_uppercase(), quantity, price, datetime),
        Command::Summary { date } => {
            match app.summarize(date).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Could not summarize portfolio: {e:?}", e = e);
                    std::process::exit(1)
                }
            };
        }
        Command::ProfitSummary { year } => {
            let year = u16::try_from(year)?;
            app.profit_summary(year);
        }
    };

    Ok(())
}

fn setup_logger() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
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
