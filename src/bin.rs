use std::path::Path;

use stocks::{Portfolio, Stock};
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
            }
            Command::Sell { symbol, quantity } => {
                if self.portfolio.sell(&symbol, quantity).is_err() {
                    println!(
                        "Your portfolio didn't had enough {} to sell.",
                        symbol.to_uppercase()
                    )
                };
            }
            Command::Summary => {
                match self.portfolio.summary() {
                    Ok(stocks) => StockCLI::display_summary(stocks),
                    Err(err) => eprintln!("{:?}", err),
                };
            }
        }
    }

    fn display_summary(mut summary: Vec<Stock>) {
        let mut total_value: f64 = 0.0;
        let mut total_change: f64 = 0.0;

        println!(
            "                               Portfolio  Summary                               "
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!("Name\t\tQuantity\tPrice\t\tValue\t\t\tChange");

        summary.sort_by_key(|a| a.symbol.clone());
        for stock in summary {
            let value = stock.quantity as f64 * stock.price;
            let change = (stock.price - stock.last_price) * stock.quantity as f64;

            total_value += value;
            total_change += change;

            println!(
                "{}\t\t{}\t\t{:.2}\t\t{:.2}\t\t{:.2}",
                stock.symbol, stock.quantity, stock.price, value, change,
            )
        }

        println!("Total\t\t\t\t\t\t{:.2}\t\t{:.2}", total_value, total_change)
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
