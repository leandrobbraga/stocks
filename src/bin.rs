use std::path::Path;

use stocks::Portfolio;
use structopt::StructOpt;

static FILEPATH: &str = "portfolio.json";

fn main() {
    let command = Arguments::from_args().command;
    let filepath = Path::new(FILEPATH);

    let mut stock = Stock::load_portfolio(filepath);
    stock.run_command(command);
    stock.save_portfolio(filepath);
}

#[derive(Default)]
struct Stock {
    portfolio: Portfolio,
}

impl Stock {
    fn load_portfolio(filepath: &Path) -> Self {
        let portfolio = match Portfolio::from_file(filepath) {
            Ok(portfolio) => portfolio,
            Err(_) => Portfolio::new(),
        };

        Stock { portfolio }
    }

    fn save_portfolio(&self, filepath: &Path) {
        self.portfolio.to_file(filepath).unwrap_or_else(|_| {
            println!(
            "Was not possible to save the file. If there was any modification it could be lost."
        )
        })
    }

    fn run_command(&mut self, command: Command) {
        match command {
            Command::Buy { symbol, quantity } => {
                self.portfolio.buy(&symbol, quantity);
                self.portfolio.summary();
            }
            Command::Sell { symbol, quantity } => {
                if self.portfolio.sell(&symbol, quantity).is_err() {
                    println!(
                        "Your portfolio didn't had enough {} to sell.",
                        symbol.to_uppercase()
                    )
                };
                self.portfolio.summary();
            }
            Command::Summary => self.portfolio.summary(),
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
        #[structopt(name = "VALUE", help = "How much it is going to be bought (e.g. 100).")]
        quantity: u32,
    },
    #[structopt(about = "Sells a stock.")]
    Sell {
        #[structopt(name = "SYMBOL", help = "The Stock ticker (e.g. BBAS3).")]
        symbol: String,
        #[structopt(name = "VALUE", help = "How much it is going to be sold (e.g. 100).")]
        quantity: u32,
    },
    #[structopt(about = "Summarizes the current portfolio.")]
    Summary,
}
