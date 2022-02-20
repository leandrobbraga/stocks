mod app;
mod render;

use app::{App, Command};
use env_logger::Env;
use std::path::Path;
use structopt::StructOpt;

static FILEPATH: &str = "portfolio.json";

fn main() {
    setup_logger();

    let command = Arguments::from_args().command;
    let filepath = Path::new(FILEPATH);

    let mut app = App::load_portfolio(filepath);
    app.run_command(command);
    app.save_portfolio(filepath);
}

fn setup_logger() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Stocks", about = "A simple CLI to manage stock market assets.")]
struct Arguments {
    #[structopt(subcommand)]
    command: Command,
}
