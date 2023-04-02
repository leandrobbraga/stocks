#[macro_use]
mod log;
mod render;
use crate::render::{render_profit_by_month, render_summary, ProfitSummaryData, SummaryData};
use anyhow::{Context, Result};
use stocks::portfolio::Portfolio;
use stocks::stock_market::StockMarket;
use stocks::{portfolio::Stock, stock_market::PricedStock};
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
        } => {
            portfolio.buy(symbol.as_str(), quantity, price, datetime);
            info!("You bought {quantity} {symbol} at R${price:10.2}.");
            portfolio.save()?;
        }
        Command::Sell {
            symbol,
            quantity,
            price,
            datetime,
        } => {
            let profit = portfolio.sell(symbol.as_str(), quantity, price, datetime)?;
            info!("You sold {quantity} {symbol} profiting R${profit:10.2}.");
            portfolio.save()?;
        }
        Command::Summary { date } => {
            let stock_market = StockMarket::new();

            let date = date
                .with_time(time::Time::from_hms(23, 59, 59).expect("BUG: Should be a valid time"))
                .assume_offset(UtcOffset::UTC);

            let stocks: Vec<&Stock> = portfolio
                .stocks
                .values()
                // To ensure that we only show stocks that we own
                .filter(|stock| stock.quantity(date) > 0)
                .collect();

            let priced_stocks = stock_market.get_stock_prices(&stocks, date);

            let stock_count = priced_stocks.len();
            let data: Vec<SummaryData> = priced_stocks
                .into_iter()
                .filter_map(|maybe_stock| maybe_stock.map(|stock| stock.into()).ok())
                .collect();

            if stock_count > data.len() {
                warn!("Could not get prices for all stocks");
            }

            render_summary(data)
        }
        Command::ProfitSummary { year } => {
            let profit_by_month = portfolio.profit_by_month(year);

            let mut data = Vec::with_capacity(12);

            for (month, summary) in profit_by_month.iter().enumerate() {
                let tax = if summary.sold_amount > 20000.0 && summary.profit > 0.0 {
                    summary.profit * 0.15
                } else {
                    0.0
                };

                data.push(ProfitSummaryData {
                    month: month as u8,
                    sold_amount: summary.sold_amount,
                    profit: summary.profit,
                    tax,
                })
            }

            render_profit_by_month(data)
        }
        Command::Help => {
            usage(&program);
        }
    }

    Ok(())
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

impl From<PricedStock> for SummaryData {
    fn from(stock: PricedStock) -> Self {
        let current_value = stock.price * stock.quantity as f64;
        let last_value = stock.last_price * stock.quantity as f64;
        let original_cost = stock.quantity as f64 * stock.average_price;

        Self {
            name: stock.symbol,
            quantity: stock.quantity,
            current_price: stock.price,
            current_value,
            change: current_value - last_value,
            change_percentage: (current_value / last_value - 1.0) * 100.0,
            average_price: stock.average_price,
            profit: current_value - original_cost,
            profit_percentage: (current_value / original_cost - 1.0) * 100.0,
            last_value,
            original_cost,
        }
    }
}
