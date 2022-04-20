use crate::render::{render_table, Data};
use log::{error, info, warn};
use std::path::Path;
use stocks::{
    portfolio::{Portfolio, PricedAsset},
    stock_market::StockMarket,
};
use structopt::StructOpt;

pub struct App {
    portfolio: Portfolio,
}

impl App {
    pub fn load_portfolio(filepath: &Path) -> Self {
        let portfolio = match Portfolio::from_file(filepath) {
            Ok(portfolio) => portfolio,
            Err(_) => Portfolio::new(),
        };

        Self { portfolio }
    }

    pub fn save_portfolio(&self, filepath: &Path) {
        self.portfolio.to_file(filepath).unwrap();
    }

    pub fn run_command(&mut self, command: Command) {
        match command {
            Command::Buy {
                symbol,
                quantity,
                price,
            } => self.buy(&symbol, quantity, price),
            Command::Sell {
                symbol,
                quantity,
                price,
            } => self.sell(&symbol, quantity, price),
            Command::Summary => self.summarize(),
        }
    }

    fn buy(&mut self, symbol: &str, quantity: u32, price: f64) {
        let stock_market = StockMarket::new();
        if let Some(class) = stock_market.asset_class(symbol) {
            self.portfolio.buy(symbol, class, quantity, price)
        } else {
            error!("Currently there is no {symbol} available in the API.");
            std::process::exit(1)
        }
    }

    fn sell(&mut self, symbol: &str, quantity: u32, price: f64) {
        let symbol = symbol.to_uppercase();

        if let Some(asset) = self.portfolio.stock(&symbol) {
            let profit = quantity as f64 * (price - asset.average_price);

            if self.portfolio.sell(&symbol, quantity).is_err() {
                warn!(
                    "You tried to sell more {symbol} than you currently posses. We could not 
                execute the desired command."
                );
                std::process::exit(1)
            } else {
                info!("You sold {quantity} {symbol} profiting R${profit:10.2}.")
            };
        } else {
            warn!(
                "Currently there is no {symbol} in your portfolio. Because of that we could not 
            execute the sell command."
            );
            std::process::exit(1)
        }
    }

    fn summarize(&self) {
        let unpriced_assets = self.portfolio.assets();
        let stock_market = StockMarket::new();

        let data = stock_market
            .fetch_assets_price(unpriced_assets)
            .into_iter()
            // We are trowing away any asset that we could not fetch the price.
            .filter_map(|maybe_asset| maybe_asset.ok())
            .map(|priced_asset| priced_asset.into())
            .collect();

        render_table(data).unwrap();
    }
}

impl From<PricedAsset> for Data {
    fn from(asset: PricedAsset) -> Self {
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

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(about = "Buy an asset.")]
    Buy {
        #[structopt(name = "SYMBOL", help = "The asset ticker (e.g. BBAS3).")]
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
    #[structopt(about = "Sell an asset.")]
    Sell {
        #[structopt(name = "SYMBOL", help = "The asset ticker (e.g. BBAS3).")]
        symbol: String,
        #[structopt(name = "VALUE", help = "How much it is going to be sold (e.g. 100).")]
        quantity: u32,
        #[structopt(
            name = "PRICE",
            help = "The price which the asset was sold (e.g. 10.0)."
        )]
        price: f64,
    },
    #[structopt(about = "Summarizes the current portfolio.")]
    Summary,
}
