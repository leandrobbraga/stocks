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

mod render {
    use std::error::Error;

    use cli_table::{
        format::{Border, Justify, Separator},
        print_stdout, Cell, CellStruct, Color, Style, Table,
    };
    use stocks::portfolio::PricedAsset;

    pub struct Data {
        name: String,
        quantity: u32,
        current_price: f64,
        current_value: f64,
        change: f64,
        change_percentage: f64,
        average_price: f64,
        profit: f64,
        profit_percentage: f64,
        last_value: f64,
        original_cost: f64,
    }

    impl Data {
        fn from_asset(asset: PricedAsset) -> Self {
            let current_value = asset.price * asset.quantity as f64;
            let last_value = asset.last_price * asset.quantity as f64;
            let original_cost = asset.quantity as f64 * asset.average_price;

            Self {
                name: asset.name,
                quantity: asset.quantity,
                current_price: asset.price,
                current_value,
                change: current_value - last_value,
                change_percentage: (current_value / last_value - 1.0) * 100.0,
                average_price: asset.average_price,
                profit: current_value - original_cost,
                profit_percentage: (current_value / original_cost - 1.0) * 100.0,
                last_value,
                original_cost,
            }
        }
    }

    pub fn build_data(assets: Vec<PricedAsset>) -> Vec<Data> {
        assets.into_iter().map(Data::from_asset).collect()
    }

    pub fn render_table(mut assets: Vec<Data>) -> Result<(), Box<dyn Error>> {
        assets.sort_by(|a, b| a.name.cmp(&b.name));

        let mut contents: Vec<Vec<CellStruct>> = assets.iter().map(format_row).collect();
        contents.push(format_totals(&assets));

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

        Ok(print_stdout(table)?)
    }

    fn format_row(data: &Data) -> Vec<CellStruct> {
        let change_color = get_color(data.change);
        let profit_color = get_color(data.profit);

        vec![
            data.name.clone().cell().justify(Justify::Left),
            data.quantity.cell().justify(Justify::Right),
            format!("R$ {:10.2}", data.current_price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {:10.2}", data.current_value)
                .cell()
                .justify(Justify::Right),
            format!("R$ {:10.2}", data.change)
                .cell()
                .justify(Justify::Right)
                .foreground_color(change_color),
            format!("{:6.2}%", data.change_percentage)
                .cell()
                .justify(Justify::Right)
                .foreground_color(change_color),
            format!("R$ {:10.2}", data.average_price)
                .cell()
                .justify(Justify::Right),
            format!("R$ {:10.2}", data.profit)
                .cell()
                .justify(Justify::Right)
                .foreground_color(profit_color),
            format!("{:6.2}%", data.profit_percentage)
                .cell()
                .justify(Justify::Right)
                .foreground_color(profit_color),
        ]
    }

    fn format_totals(data: &[Data]) -> Vec<CellStruct> {
        let current_value: f64 = data.iter().map(|data| data.current_value).sum();
        let original_cost: f64 = data.iter().map(|data| data.original_cost).sum();
        let last_value: f64 = data.iter().map(|data| data.last_value).sum();
        let change: f64 = data.iter().map(|data| data.change).sum();
        let profit: f64 = data.iter().map(|data| data.profit).sum();

        let change_color = get_color(change);
        let profit_color = get_color(profit);

        vec![
            "Total".cell().justify(Justify::Left).bold(true),
            "".cell(),
            "".cell(),
            format!("R$ {:10.2}", current_value)
                .cell()
                .justify(Justify::Right)
                .bold(true),
            format!("R$ {change:10.2}")
                .cell()
                .justify(Justify::Right)
                .foreground_color(change_color),
            format!("{:6.2}%", (change / last_value) * 100.0)
                .cell()
                .justify(Justify::Right)
                .bold(true)
                .foreground_color(change_color),
            "".cell(),
            format!("R$ {:10.2}", profit)
                .cell()
                .justify(Justify::Right)
                .foreground_color(profit_color),
            format!("{:6.2}%", (profit / original_cost) * 100.0)
                .cell()
                .justify(Justify::Right)
                .bold(true)
                .foreground_color(profit_color),
        ]
    }

    fn get_color(value: f64) -> Option<Color> {
        match value.partial_cmp(&0.0).unwrap() {
            std::cmp::Ordering::Less => Some(Color::Red),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(Color::Green),
        }
    }
}
