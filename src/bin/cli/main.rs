mod commands;
#[macro_use]
mod log;
mod render;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use stocks::portfolio::Portfolio;
use stocks::stock_market::StockMarket;

enum Command {
    Buy {
        symbol: String,
        quantity: u32,
        price: f64,
        datetime: NaiveDateTime,
    },
    Sell {
        symbol: String,
        quantity: u32,
        price: f64,
        datetime: NaiveDateTime,
    },
    Summary {
        date: NaiveDate,
    },
    ProfitSummary {
        year: i32,
    },
    Help,
}

fn main() -> Result<()> {
    let mut args = std::env::args();

    let Some(program) = args.next() else {
        error!("Could not get program name.");
        std::process::exit(1);
    };

    let command = match parse_command(args) {
        Ok(command) => command,
        Err(err) => {
            usage(&program);
            println!();
            error!("{err}");
            std::process::exit(1);
        }
    };

    let mut portfolio = Portfolio::load().unwrap_or_else(|err| {
        warn!("Could not load portfolio: {err}");
        info!("Creating a new portfolio.");
        Portfolio::new()
    });

    match command {
        Command::Buy {
            symbol,
            quantity,
            price,
            datetime,
        } => commands::buy(
            &mut portfolio,
            &symbol.to_uppercase(),
            quantity,
            price,
            datetime,
        ),
        Command::Sell {
            symbol,
            quantity,
            price,
            datetime,
        } => commands::sell(
            &mut portfolio,
            &symbol.to_uppercase(),
            quantity,
            price,
            datetime,
        ),
        Command::Summary { date } => {
            let stock_market = StockMarket::new();
            commands::summarize(&portfolio, &stock_market, date)
        }
        Command::ProfitSummary { year } => {
            let year = u16::try_from(year)?;
            commands::profit_summary(&portfolio, year)
        }
        Command::Help => {
            usage(&program);
            Ok(())
        }
    }
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let Some(command) = args.next() else {
        return Err("No subcommand provided.".into())
    };

    match command.as_str() {
        "buy" | "sell" => {
            let Some(symbol) = args.next() else {
                return Err("No stock symbol provided.".into())
            };

            let Some(quantity) = args.next() else {
                return Err("No quantity provided.".into());
            };

            let Ok(quantity) = quantity.parse::<u32>() else {
                return Err("Could not parse quantity.".into());
            };

            let Some(price) = args.next() else {
                return Err("No price provided.".into());
            };

            let Ok(price) = price.parse::<f64>() else {
                return Err("Could not parse price.".into());
            };

            let Ok(datetime) = parse_datetime(args.next()) else {
                return Err("Could not parse datetime.".into());
            };

            return Ok(match command.as_str() {
                "buy" => Command::Buy {
                    symbol,
                    quantity,
                    price,
                    datetime,
                },
                "sell" => Command::Sell {
                    symbol,
                    quantity,
                    price,
                    datetime,
                },
                _ => unreachable!(),
            });
        }
        "summary" => {
            let Ok(date) = parse_date(args.next()) else {
                return Err("Could not parse date.".into());
            };

            Ok(Command::Summary { date })
        }
        "profit-summary" => {
            let year = match args.next() {
                Some(year) => {
                    let Ok(year) = year.parse::<i32>() else {
                        return Err("Could not parse year.".into());
                    };
                    year
                }
                None => chrono::Local::now().date_naive().year() as i32,
            };

            Ok(Command::ProfitSummary { year })
        }
        "-h" | "--help" => Ok(Command::Help),
        _ => Err(format!("Unknown subcommand `{command}`")),
    }
}

fn usage(program: &str) {
    eprintln!("A simple tool to monitor a stock portfolio directly from terminal.\n");
    eprintln!("\x1b[4;1mUSAGE\x1b[0m: {program} <SUBCOMMAND> [OPTIONS]\n");
    eprintln!("\x1b[4;1mCOMMANDS\x1b[0m:");
    eprintln!("  \x1b[4mbuy\x1b[0m <STOCK> <QUANTITY> <PRICE> [DATETIME]          add the <STOCK> <QUANTITY> to the portfolio at a given <PRICE>, the default [DATETIME] is now");
    eprintln!("  \x1b[4msell\x1b[0m <STOCK> <QUANTITY> <PRICE> [DATETIME]         remove the <STOCK> <QUANTITY> from the portfolio at a given <PRICE>, the default [DATETIME] is now");
    eprintln!("  \x1b[4msummary\x1b[0m [DATE]                                     show the state of the portfolio at a given [DATE], the default [DATE] is now");
    eprintln!("  \x1b[4mprofit-summary\x1b[0m [YEAR]                              show the month-by-month portfolio profit for a given [YEAR], the default [YEAR] is the current year");
}

fn parse_datetime(arg: Option<String>) -> Result<NaiveDateTime> {
    Ok(match arg {
        Some(date) => NaiveDateTime::parse_from_str(date.as_str(), "%Y-%m-%d %H:%M:%S")?,
        None => chrono::Local::now().naive_local(),
    })
}

fn parse_date(arg: Option<String>) -> Result<NaiveDate> {
    Ok(match arg {
        Some(date) => NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")?,
        None => chrono::Local::now().date_naive(),
    })
}
