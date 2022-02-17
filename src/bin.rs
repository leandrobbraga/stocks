use std::path::Path;

use cli_table::{
    format::{Border, Justify, Separator},
    print_stdout, Cell, CellStruct, Color, Style, Table,
};
use stocks::{
    portfolio::{Portfolio, PricedAsset},
    stock_market::StockMarket,
};
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

    fn display_summary(mut summary: Vec<PricedAsset>) {
        // This makes the output table stable (i.e. the order of the assets are always the same.)
        summary.sort_by(|a, b| a.name.cmp(&b.name));

        let mut contents: Vec<Vec<CellStruct>> = summary.iter().map(StockCLI::format_row).collect();
        contents.push(StockCLI::format_totals(&summary));

        let table = contents
            .table()
            .title(vec![
                "Name".cell().bold(true).justify(Justify::Left),
                "Quantity".cell().bold(true).justify(Justify::Center),
                "Current Price".cell().bold(true).justify(Justify::Center),
                "Current Value".cell().bold(true).justify(Justify::Center),
                "Change (Day)".cell().bold(true).justify(Justify::Center),
                "% Change (Day)".cell().bold(true).justify(Justify::Center),
                "Average Price".cell().bold(true).justify(Justify::Center),
                "Profit".cell().bold(true).justify(Justify::Center),
                "% Profit".cell().bold(true).justify(Justify::Center),
            ])
            .separator(Separator::builder().build())
            .border(Border::builder().build());

        print_stdout(table).unwrap();
    }

    fn format_row(asset: &PricedAsset) -> Vec<CellStruct> {
        let value = asset.quantity as f64 * asset.price;
        let original_value = asset.last_price * asset.quantity as f64;
        let current_value = asset.price * asset.quantity as f64;
        let change = current_value - original_value;
        let change_percentage = (current_value / original_value - 1.0) * 100.0;
        let cost = asset.quantity as f64 * asset.average_price;
        let profit = current_value - cost;
        let profit_percentage = (current_value / cost - 1.0) * 100.0;

        return vec![
            asset.name.clone().cell().justify(Justify::Left),
            asset.quantity.cell().justify(Justify::Right),
            format!("R$ {:10.2}", asset.price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {value:10.2}").cell().justify(Justify::Right),
            format!("R$ {change:10.2}")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(change)),
            format!("{change_percentage:6.2}%")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(change_percentage)),
            format!("R$ {:10.2}", asset.average_price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {profit:10.2}")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(profit)),
            format!("{profit_percentage:6.2}%")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(profit_percentage)),
        ];
    }

    fn format_totals(assets: &[PricedAsset]) -> Vec<CellStruct> {
        let current_value: f64 = assets
            .iter()
            .map(|asset| asset.quantity as f64 * asset.price)
            .sum();
        let original_value: f64 = assets
            .iter()
            .map(|asset| asset.quantity as f64 * asset.last_price)
            .sum();
        let cost: f64 = assets
            .iter()
            .map(|asset| asset.quantity as f64 * asset.average_price)
            .sum();

        let change = current_value - original_value;
        let change_percentage = (current_value / original_value - 1.0) * 100.0;
        let profit = current_value - cost;
        let profit_percentage = (current_value / cost - 1.0) * 100.0;

        return vec![
            "Total".cell().justify(Justify::Left).bold(true),
            "".cell(),
            "".cell(),
            format!("R$ {current_value:10.2}")
                .cell()
                .justify(Justify::Right)
                .bold(true),
            format!("R$ {change:10.2}")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(change)),
            format!("{change_percentage:6.2}%")
                .cell()
                .justify(Justify::Right)
                .bold(true)
                .foreground_color(StockCLI::get_color(change_percentage)),
            "".cell(),
            format!("R$ {profit:10.2}")
                .cell()
                .justify(Justify::Right)
                .foreground_color(StockCLI::get_color(profit)),
            format!("{profit_percentage:6.2}%")
                .cell()
                .justify(Justify::Right)
                .bold(true)
                .foreground_color(StockCLI::get_color(profit_percentage)),
        ];
    }

    fn get_color(value: f64) -> Option<Color> {
        match value.partial_cmp(&0.0).unwrap() {
            std::cmp::Ordering::Less => Some(Color::Red),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(Color::Green),
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
