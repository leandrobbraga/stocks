mod app;
mod render;

use anyhow::Result;
use app::App;
use chrono::NaiveDateTime;
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
        #[clap(value_parser=parse_date)]
        #[arg(default_value_t = chrono::Local::now().naive_local())]
        date: NaiveDateTime,
    },
    /// Sells an asset
    Sell {
        /// The ticker of the stock (e.g. BBAS3)
        symbol: String,
        /// How many stocks was sold (e.g. 100)
        quantity: u32,
        /// How much was the average cost of the sell (e.g. 33.21)
        price: f64,
        #[clap(value_parser=parse_date)]
        #[arg(default_value_t = chrono::Local::now().naive_local())]
        date: NaiveDateTime,
    },
    /// Print a summary of the portfolio
    Summary,
    ProfitSummary {
        year: u16,
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
            date,
        } => {
            app.buy(&symbol.to_uppercase(), quantity, price, date);
        }
        Command::Sell {
            symbol,
            quantity,
            price,
            date,
        } => {
            app.sell(&symbol.to_uppercase(), quantity, price, date);
        }
        Command::Summary => match app.summarize().await {
            Ok(_) => (),
            Err(e) => {
                error!("Could not summarize portfolio: {e:?}", e = e);
                std::process::exit(1)
            }
        },
        Command::ProfitSummary { year } => {
            app.profit_summary(year);
        }
    };

    Ok(())
}

fn setup_logger() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
}

fn parse_date(arg: &str) -> Result<NaiveDateTime> {
    Ok(NaiveDateTime::parse_from_str(arg, "%Y-%m-%d %H:%M:%S")?)
}
