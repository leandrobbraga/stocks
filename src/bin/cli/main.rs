mod render;

use std::path::Path;

use render::{build_data, render_table};
use stocks::{portfolio::Portfolio, stock_market::StockMarket};
use structopt::StructOpt;

static FILEPATH: &str = "portfolio.json";

fn main() {
    let command = Arguments::from_args().command;
    let filepath = Path::new(FILEPATH);

    let mut stock = StockCLI::load_portfolio(filepath);
    stock.run_command(command);
    stock.save_portfolio(filepath);
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
            } => {
                let stock_market = StockMarket::new();
                let class = stock_market.asset_class(&symbol);

                match class {
                    Some(class) => self.portfolio.buy(&symbol, class, quantity, price),
                    None => {
                        println!("We could not find {symbol} asset in the stock market.");
                        std::process::exit(1)
                    }
                }
            }
            Command::Sell {
                symbol,
                quantity,
                price,
            } => {
                if let Some(asset) = self.portfolio.stock(&symbol) {
                    let profit = quantity as f64 * (price - asset.average_price);

                    if self.portfolio.sell(&symbol, quantity).is_err() {
                        println!("Your portfolio didn't had enough {symbol} to sell.");
                        std::process::exit(1)
                    } else {
                        println!("You sold {quantity} {symbol} profiting R${profit:10.2}.")
                    };
                } else {
                    println!("You don't own any {symbol} to sell.");
                    std::process::exit(1)
                }
            }
            Command::Summary => {
                let assets = self.portfolio.assets();
                let stock_market = StockMarket::new();

                let prices = stock_market
                    .fetch_assets_price(assets)
                    .into_iter()
                    .filter_map(|asset| asset.ok())
                    .collect();

                let data = build_data(prices);
                render_table(data).unwrap();
            }
        }
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
