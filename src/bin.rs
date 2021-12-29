use stocks::Portfolio;
use structopt::StructOpt;

fn main() {
    let arguments = Arguments::from_args();

    let mut portfolio = match Portfolio::from_file() {
        Ok(portfolio) => portfolio,
        Err(_) => Portfolio::new(),
    };

    match arguments.command {
        Command::Buy { symbol, quantity } => {
            portfolio.buy(&symbol, quantity);
            portfolio.summary();
            if portfolio.save().is_err() {
                println!("Was not possible to save the file. The last modification could be lost.")
            };
        }
        Command::Sell { symbol, quantity } => {
            if portfolio.sell(&symbol, quantity).is_err() {
                println!(
                    "Your portfolio didn't had enough {} to sell.",
                    symbol.to_uppercase()
                )
            };
            portfolio.summary();

            if portfolio.save().is_err() {
                println!("Was not possible to save the file. The last modification could be lost.")
            };
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
