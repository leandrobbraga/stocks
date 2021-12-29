use std::path::Path;

use stocks::Portfolio;
use structopt::StructOpt;

static FILEPATH: &str = "portfolio.json";

fn main() {
    let arguments = Arguments::from_args();
    let filepath = Path::new(FILEPATH);

    let mut portfolio = match Portfolio::from_file(filepath) {
        Ok(portfolio) => portfolio,
        Err(_) => Portfolio::new(),
    };

    run_command(&mut portfolio, arguments.command);

    portfolio.to_file(filepath).unwrap_or_else(|_| {
        println!(
            "Was not possible to save the file. If there was any modification it could be lost."
        )
    })
}

fn run_command(portfolio: &mut Portfolio, command: Command) {
    match command {
        Command::Buy { symbol, quantity } => {
            portfolio.buy(&symbol, quantity);
            portfolio.summary();
        }
        Command::Sell { symbol, quantity } => {
            if portfolio.sell(&symbol, quantity).is_err() {
                println!(
                    "Your portfolio didn't had enough {} to sell.",
                    symbol.to_uppercase()
                )
            };
            portfolio.summary();
        }
        Command::Summary => portfolio.summary(),
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
