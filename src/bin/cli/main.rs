mod app;
mod render;

use app::App;
use clap::{Parser, Subcommand};
use env_logger::Env;
use std::path::Path;

static FILEPATH: &str = "portfolio.json";

#[tokio::main]
async fn main() {
    setup_logger();

    let command = Arguments::parse().command;
    let filepath = Path::new(FILEPATH);

    let mut app = App::load_portfolio(filepath);
    run_command(&mut app, command);
    app.save_portfolio(filepath);
}

fn setup_logger() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Buys an asset
    Buy {
        /// The ticker of the stock (e.g. BBAS3)
        symbol: String,
        /// How many stocks were purchased (e.g. 100)
        quantity: u32,
        /// How much was the average cost of the purchase (e.g. 33.21)
        price: f64,
    },
    /// Sells an asset
    Sell {
        /// The ticker of the stock (e.g. BBAS3)
        symbol: String,
        /// How many stocks was sold (e.g. 100)
        quantity: u32,
        /// How much was the average cost of the sell (e.g. 33.21)
        price: f64,
    },
    /// Print a summary of the portfolio
    Summary,
}

pub fn run_command(app: &mut App, command: Command) {
    match command {
        Command::Buy {
            symbol,
            quantity,
            price,
        } => app.buy(&symbol, quantity, price),
        Command::Sell {
            symbol,
            quantity,
            price,
        } => app.sell(&symbol, quantity, price),
        Command::Summary => app.summarize(),
    }
}
