use std::path::Path;

use cli_table::{format::Justify, print_stdout, Cell, CellStruct, Style, Table};
use stocks::{Portfolio, PricedAsset, StockMarket};
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
        let mut contents: Vec<Vec<CellStruct>> = summary.iter().map(StockCLI::format_row).collect();
        contents.push(StockCLI::format_totals(&summary));

        let table = contents.table().title(vec![
            "Name".cell().bold(true).justify(Justify::Center),
            "Quantity".cell().bold(true).justify(Justify::Center),
            "Current Price".cell().bold(true).justify(Justify::Center),
            "Current Value".cell().bold(true).justify(Justify::Center),
            "Change (Day)".cell().bold(true).justify(Justify::Center),
            "Average Price".cell().bold(true).justify(Justify::Center),
            "Profit".cell().bold(true).justify(Justify::Center),
        ]);

        print_stdout(table).unwrap();
    }

    fn format_row(asset: &PricedAsset) -> Vec<CellStruct> {
        let value = asset.quantity as f64 * asset.price;
        let change = (asset.price - asset.last_price) * asset.quantity as f64;
        let profit = asset.quantity as f64 * (asset.price - asset.average_price);

        return vec![
            asset.name.clone().cell().justify(Justify::Center),
            asset.quantity.cell().justify(Justify::Right),
            format!("R$ {:10.2}", asset.price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {value:10.2}").cell().justify(Justify::Right),
            format!("R$ {change:10.2}").cell().justify(Justify::Right),
            format!("R$ {:10.2}", asset.average_price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {profit:10.2}").cell().justify(Justify::Right),
        ];
    }

    fn format_totals(assets: &Vec<PricedAsset>) -> Vec<CellStruct> {
        let total_value: f64 = assets
            .iter()
            .map(|asset| asset.quantity as f64 * asset.price)
            .sum();

        let total_change: f64 = assets
            .iter()
            .map(|asset| (asset.price - asset.last_price) * asset.quantity as f64)
            .sum();

        let total_profit: f64 = assets
            .iter()
            .map(|asset| asset.quantity as f64 * (asset.price - asset.average_price))
            .sum();

        return vec![
            "Total".cell().justify(Justify::Center).bold(true),
            "".cell(),
            "".cell(),
            format!("R$ {total_value:10.2}")
                .cell()
                .justify(Justify::Right)
                .bold(true),
            format!("R$ {total_change:10.2}")
                .cell()
                .justify(Justify::Right)
                .bold(true),
            "".cell(),
            format!("R$ {total_profit:10.2}")
                .cell()
                .justify(Justify::Right)
                .bold(true),
        ];
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
