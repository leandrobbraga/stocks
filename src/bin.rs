use std::path::Path;

use stocks::{AssetClass, Portfolio, PricedAsset, StockMarket};
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
                StockCLI::display_summary(prices)
            }
        }
    }

    fn display_summary(summary: Vec<PricedAsset>) {
        let mut stocks: Vec<PricedAsset> = summary
            .iter()
            .cloned()
            .filter(|asset| asset.class == AssetClass::Stock)
            .collect();

        let mut fiis: Vec<PricedAsset> = summary
            .iter()
            .cloned()
            .filter(|asset| asset.class == AssetClass::FII)
            .collect();

        println!(
            "----------------------------------------------------------------------------------------------------------------------------"
        );
        println!(
            "                                                     Portfolio  Summary                                                     "
        );
        println!(
            "----------------------------------------------------------------------------------------------------------------------------"
        );

        println!(
            "Name\t    Quantity\t\t   Price\t       Value\t\t      Change\t       Average Price  Current Profit"
        );

        let mut stocks_total_value: f64 = 0.0;
        let mut stocks_total_change: f64 = 0.0;
        let mut stocks_total_profit: f64 = 0.0;

        stocks.sort_by_key(|asset| asset.name.clone());
        for stock in stocks {
            let value = stock.quantity as f64 * stock.price;
            let change = (stock.price - stock.last_price) * stock.quantity as f64;

            let profit = stock.quantity as f64 * (stock.price - stock.average_price);

            stocks_total_value += value;
            stocks_total_change += change;
            stocks_total_profit += profit;

            println!(
                "{}\t\t{}\t\tR${:6.2}\tR${value:10.2}\t\tR${change:10.2}\t\tR${:10.2}\tR${profit:10.2}",
                stock.name, stock.quantity, stock.price, stock.average_price,
            )
        }

        println!(
            "............................................................................................................................"
        );
        println!(
            "Stocks\t\t\t\t\t\tR${stocks_total_value:10.2}\t\tR${stocks_total_change:10.2}\t\t\t        R${stocks_total_profit:10.2}",
        );
        println!(
            "----------------------------------------------------------------------------------------------------------------------------"
        );

        let mut fiis_total_value: f64 = 0.0;
        let mut fiis_total_change: f64 = 0.0;
        let mut fiis_total_profit: f64 = 0.0;

        fiis.sort_by_key(|asset| asset.name.clone());
        for fii in fiis {
            let value = fii.quantity as f64 * fii.price;
            let change = (fii.price - fii.last_price) * fii.quantity as f64;

            let profit = fii.quantity as f64 * (fii.price - fii.average_price);

            fiis_total_value += value;
            fiis_total_change += change;
            fiis_total_profit += profit;

            println!(
                "{}\t\t{}\t\tR${:6.2}\tR${value:10.2}\t\tR${change:10.2}\t\tR${:10.2}\tR${profit:10.2}",
                fii.name, fii.quantity, fii.price, fii.average_price,
            )
        }

        println!(
            "............................................................................................................................"
        );
        println!(
            "FIIs\t\t\t\t\t\tR${fiis_total_value:10.2}\t\tR${fiis_total_change:10.2}\t\t\t        R${fiis_total_profit:10.2}",
        );
        println!(
            "----------------------------------------------------------------------------------------------------------------------------"
        );
        println!(
            "Total\t\t\t\t\t\tR${:10.2}\t\tR${:10.2}\t\t\t        R${:10.2}",
            stocks_total_value + fiis_total_value,
            stocks_total_change + fiis_total_change,
            stocks_total_profit + fiis_total_profit
        );
        println!(
            "----------------------------------------------------------------------------------------------------------------------------"
        );
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
