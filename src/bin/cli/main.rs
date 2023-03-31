mod commands;
#[macro_use]
mod log;
mod render;
use anyhow::{Context, Result};
use stocks::portfolio::Portfolio;
use stocks::stock_market::StockMarket;
use time::{format_description, Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

enum Command {
    Buy {
        symbol: String,
        quantity: u32,
        price: f64,
        datetime: OffsetDateTime,
    },
    Sell {
        symbol: String,
        quantity: u32,
        price: f64,
        datetime: OffsetDateTime,
    },
    Summary {
        date: Date,
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
            error!("{err}: {}", err.root_cause());
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

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command> {
    let command = args.next().context("No subcommand provided")?;

    match command.as_str() {
        "buy" | "sell" => {
            let symbol = args.next().context("No stock symbol provided")?;

            let quantity = args.next().context("No quantity provided")?;

            let quantity = quantity
                .parse::<u32>()
                .context("Could not parse quantity")?;

            let price = args.next().context("No price provided.")?;
            let price = price.parse::<f64>().context("Could not parse price")?;

            let datetime = parse_datetime(args.next()).context("Could not parse datetime")?;

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
            let date = parse_date(args.next()).context("Could not parse date")?;

            Ok(Command::Summary { date })
        }
        "profit-summary" => {
            let year = match args.next() {
                Some(year) => year.parse::<i32>().context("Could not parse year")?,
                None => OffsetDateTime::now_utc().year() as i32,
            };

            Ok(Command::ProfitSummary { year })
        }
        "-h" | "--help" => Ok(Command::Help),
        _ => anyhow::bail!("Unknown subcommand `{command}`"),
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

fn parse_datetime(arg: Option<String>) -> Result<OffsetDateTime> {
    Ok(match arg {
        Some(date) => PrimitiveDateTime::parse(
            date.as_str(),
            &format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?,
        )?
        .assume_offset(UtcOffset::UTC),
        None => OffsetDateTime::now_utc(),
    })
}

fn parse_date(arg: Option<String>) -> Result<Date> {
    Ok(match arg {
        Some(date) => Date::parse(
            date.as_str(),
            &format_description::parse("[year]-[month]-[day]")?,
        )?,
        None => OffsetDateTime::now_utc().date(),
    })
}
