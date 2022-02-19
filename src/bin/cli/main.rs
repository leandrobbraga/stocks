mod render;

use env_logger::Env;
use log::{error, info, warn};
use render::{build_data, render_table};
use std::path::Path;
use stocks::{portfolio::Portfolio, stock_market::StockMarket};
use structopt::StructOpt;

static FILEPATH: &str = "portfolio.json";

fn main() {
    setup_logger();

    let command = Arguments::from_args().command;
    let filepath = Path::new(FILEPATH);

    let mut stock = StockCLI::load_portfolio(filepath);
    stock.run_command(command);
    stock.save_portfolio(filepath);
}

fn setup_logger() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
}

struct StockCLI {
    portfolio: Portfolio,
}

impl StockCLI {
    fn load_portfolio(filepath: &Path) -> Self {
        let portfolio = match Portfolio::from_file(filepath) {
            Ok(portfolio) => portfolio,
            Err(_) => Portfolio::new(),
        };

        StockCLI { portfolio }
    }

    fn save_portfolio(&self, filepath: &Path) {
        self.portfolio.to_file(filepath).unwrap();
    }

    fn run_command(&mut self, command: Command) {
        match command {
            Command::Buy {
                symbol,
                quantity,
                price,
            } => self.buy(&symbol, quantity, price),
            Command::Sell {
                symbol,
                quantity,
                price,
            } => self.sell(&symbol, quantity, price),
            Command::Summary => self.summarize(),
        }
    }

    fn buy(&mut self, symbol: &str, quantity: u32, price: f64) {
        let stock_market = StockMarket::new();
        if let Some(class) = stock_market.asset_class(symbol) {
            self.portfolio.buy(symbol, class, quantity, price)
        } else {
            error!("Currently there is no {symbol} available in the API.");
            std::process::exit(1)
        }
    }

    fn sell(&mut self, symbol: &str, quantity: u32, price: f64) {
        let symbol = symbol.to_uppercase();

        if let Some(asset) = self.portfolio.stock(&symbol) {
            let profit = quantity as f64 * (price - asset.average_price);

            if self.portfolio.sell(&symbol, quantity).is_err() {
                warn!(
                    "You tried to sell more {symbol} than you currently posses. We could not 
                execute the desired command."
                );
                std::process::exit(1)
            } else {
                info!("You sold {quantity} {symbol} profiting R${profit:10.2}.")
            };
        } else {
            warn!(
                "Currently there is no {symbol} in your portfolio. Because of that we could not 
            execute the sell command."
            );
            std::process::exit(1)
        }
    }

    fn summarize(&self) {
        let unpriced_assets = self.portfolio.assets();
        let stock_market = StockMarket::new();

        let priced_assets = stock_market
            .fetch_assets_price(unpriced_assets)
            .into_iter()
            // We are trowing away any asset that we could not fetch the price.
            .filter_map(|asset| asset.ok())
            .collect();

        let data = build_data(priced_assets);
        render_table(data).unwrap();
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Stocks", about = "A simple CLI to manage stocks.")]
struct Arguments {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(about = "Buys a stock.")]
    Buy {
        #[structopt(name = "SYMBOL", help = "The Stock ticker (e.g. BBAS3).")]
        symbol: String,
        #[structopt(
            name = "QUANTITY",
            help = "How much it is going to be bought (e.g. 100)."
        )]
        quantity: u32,
        #[structopt(
            name = "PRICE",
            help = "The price which the asset was bought (e.g. 10.0)."
        )]
        price: f64,
    },
    #[structopt(about = "Sells a stock.")]
    Sell {
        #[structopt(name = "SYMBOL", help = "The Stock ticker (e.g. BBAS3).")]
        symbol: String,
        #[structopt(name = "VALUE", help = "How much it is going to be sold (e.g. 100).")]
        quantity: u32,
        #[structopt(
            name = "PRICE",
            help = "The price which the asset was bought (e.g. 10.0)."
        )]
        price: f64,
    },
    #[structopt(about = "Summarizes the current portfolio.")]
    Summary,
}
