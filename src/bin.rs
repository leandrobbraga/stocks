use stocks::Portfolio;
use structopt::StructOpt;

fn main() {
    let args = Arguments::from_args();
    let mut portfolio = Portfolio::new();

    match args.command {
        Command::Buy { symbol, quantity } => {
            portfolio.buy(&symbol, quantity);
            portfolio.summary();
        }
        Command::Sell { symbol, quantity } => {
            portfolio
                .sell(&symbol, quantity)
                .expect("Sold more than the current quantity for that stock.");
            portfolio.summary();
        }
        Command::Summary => portfolio.summary(),
    }

    std::process::exit(0)
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
