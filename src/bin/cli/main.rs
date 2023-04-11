mod log;
mod render;

use std::path::PathBuf;

use crate::render::{render_profit_by_month, render_summary, ProfitSummaryData, SummaryData};
use anyhow::{Context, Result};
use stocks::portfolio::Portfolio;
use stocks::stock_market::PricedStock;
use stocks::stock_market::StockMarket;
use time::{format_description, Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

enum Command {
    Buy {
        stock: String,
        quantity: u32,
        price: f64,
        datetime: Option<OffsetDateTime>,
    },
    Sell {
        stock: String,
        quantity: u32,
        price: f64,
        datetime: Option<OffsetDateTime>,
    },
    Summary {
        date: Option<Date>,
        watch: bool,
    },
    ProfitSummary {
        year: i32,
    },
    Split {
        stock: String,
        ratio: f64,
        date: Option<Date>,
    },
    DumpTrades {
        path: PathBuf,
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
            stock,
            quantity,
            price,
            datetime,
        } => {
            let datetime = datetime.unwrap_or_else(|| {
                OffsetDateTime::now_local().expect("BUG: Could not get the local time.")
            });

            portfolio.buy(stock.as_str(), quantity, price, datetime);
            info!("You bought {quantity} {stock} at R${price:10.2}.");
            portfolio.save()?;
        }
        Command::Sell {
            stock,
            quantity,
            price,
            datetime,
        } => {
            let datetime = datetime.unwrap_or_else(|| {
                OffsetDateTime::now_local().expect("BUG: Could not get the local time.")
            });

            let profit = portfolio.sell(stock.as_str(), quantity, price, datetime)?;
            info!("You sold {quantity} {stock} profiting R${profit:10.2}.");
            portfolio.save()?;
        }
        Command::Summary { date, watch } => {
            let stock_market = StockMarket::new();

            let datetime = date
                .map(|date| {
                    date.with_time(
                        time::Time::from_hms(23, 59, 59).expect("BUG: Should be a valid time"),
                    )
                    .assume_offset(
                        UtcOffset::current_local_offset()
                            .expect("BUG: Could not get the local offset."),
                    )
                })
                .unwrap_or_else(|| {
                    OffsetDateTime::now_local().expect("BUG: Could not get the local time.")
                });

            let stocks: Vec<_> = portfolio
                .stocks
                .into_values()
                // To ensure that we only show stocks that we own
                .filter(|stock| stock.quantity(datetime) > 0)
                .collect();

            loop {
                let priced_stocks = stock_market.get_stock_prices(&stocks, datetime);

                let stock_count = priced_stocks.len();
                let data: Vec<SummaryData> = priced_stocks
                    .into_iter()
                    .filter_map(|maybe_stock| maybe_stock.map(|stock| stock.into()).ok())
                    .collect();

                if stock_count > data.len() {
                    warn!("Could not get prices for all stocks");
                }

                // We opt to not clear the screen here, so we are able to see the changes
                render_summary(data);
                info!(
                    "Summary updated at: {}",
                    OffsetDateTime::now_local()?.format(&format_description::parse(
                        "[year]-[month]-[day] [hour]:[minute]:[second]"
                    )?)?
                );

                if !watch {
                    break;
                }

                // The API that we currently use updates roughly once every 20 minutes
                std::thread::sleep(std::time::Duration::from_secs(20 * 60));
            }
        }
        Command::ProfitSummary { year } => {
            let profit_by_month = portfolio.profit_by_month(year).map(|summary| {
                let tax = if summary.sold_amount > 20000.0 && summary.profit > 0.0 {
                    summary.profit * 0.15
                } else {
                    0.0
                };

                ProfitSummaryData {
                    sold_amount: summary.sold_amount,
                    profit: summary.profit,
                    tax,
                }
            });

            render_profit_by_month(&profit_by_month);
        }
        Command::Split { stock, ratio, date } => {
            let datetime = date
                .map(|date| {
                    date.with_time(
                        time::Time::from_hms(23, 59, 59).expect("BUG: Should be a valid time"),
                    )
                    .assume_offset(
                        UtcOffset::current_local_offset()
                            .expect("BUG: Could not get the local offset."),
                    )
                })
                .unwrap_or_else(|| {
                    OffsetDateTime::now_local().expect("BUG: Could not get the local time.")
                });

            portfolio.split(stock.as_str(), ratio, datetime);

            if ratio > 1.0 {
                info!("You performed a {ratio:.2}:1 stock split for {stock}.");
            } else {
                let ratio = 1.0 / ratio;
                info!("You performed a 1:{ratio:.2} stock split for {stock}.");
            }
            portfolio.save().map_err(|err| {
                error!("Could not save portfolio: {err}");
                err
            })?;
        }
        Command::DumpTrades { path } => {
            let file = std::fs::File::create(&path).map_err(|err| {
                error!("Could not create file {path:?}: {err}");
                err
            })?;

            let mut file = std::io::BufWriter::new(file);

            portfolio.dump_trades(&mut file).map_err(|err| {
                error!("Could not dump trades: {err}");
                err
            })?;

            info!("Trades dumped to {path:?}.");
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
            let stock = args
                .next()
                .context("No stock symbol provided")?
                .to_uppercase();

            let quantity = args.next().context("No quantity provided")?;

            let quantity = quantity.parse().context("Could not parse quantity")?;

            let price = args.next().context("No price provided.")?;
            let price = price.parse().context("Could not parse price")?;

            let datetime = args
                .next()
                .map(|arg| parse_datetime(arg.as_str()))
                .transpose()?;

            return Ok(match command.as_str() {
                "buy" => Command::Buy {
                    stock,
                    quantity,
                    price,
                    datetime,
                },
                "sell" => Command::Sell {
                    stock,
                    quantity,
                    price,
                    datetime,
                },
                _ => unreachable!(),
            });
        }
        "summary" => {
            let date;
            let watch;
            match args.next() {
                Some(s) => match s.as_str() {
                    "-w" | "--watch" => {
                        date = None;
                        watch = true;
                    }
                    _ => {
                        date = Some(parse_date(s.as_str())?);
                        watch = false;
                    }
                },
                None => {
                    date = None;
                    watch = false;
                }
            }

            Ok(Command::Summary { date, watch })
        }
        "profit-summary" => {
            let year = match args.next() {
                Some(year) => year.parse().context("Could not parse year")?,
                None => OffsetDateTime::now_local()?.year(),
            };

            Ok(Command::ProfitSummary { year })
        }
        "split" => {
            let stock = args
                .next()
                .context("No stock stock provided")?
                .to_uppercase();

            let ratio = args.next().context("No ratio provided")?.parse()?;

            let date = args
                .next()
                .map(|arg| parse_date(arg.as_ref()))
                .transpose()?;

            Ok(Command::Split { stock, ratio, date })
        }
        "dump" => {
            let path = PathBuf::from(args.next().context("No path provided")?);

            Ok(Command::DumpTrades { path })
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
    eprintln!("  \x1b[4msummary\x1b[0m [DATE] [-w | --watch]                      show the state of the portfolio at a given [DATE], the default [DATE] is now");
    eprintln!("  \x1b[4mprofit-summary\x1b[0m [YEAR]                              show the month-by-month portfolio profit for a given [YEAR], the default [YEAR] is the current year");
    eprintln!("  \x1b[4msplit\x1b[0m <STOCK> <RATIO> [DATE]                       perform a stock split on a given <STOCK> in a given [DATE] increasing the number of stocks by <RATIO>");
    eprintln!("  \x1b[4mdump\x1b[0m <FILEPATH>                                    dumps the trade history from all stocks to a given <FILEPATH>");
}

fn parse_datetime(date: &str) -> Result<OffsetDateTime> {
    Ok(PrimitiveDateTime::parse(
        date,
        &format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?,
    )?
    .assume_offset(UtcOffset::UTC))
}

fn parse_date(date: &str) -> Result<Date> {
    Ok(Date::parse(
        date,
        &format_description::parse("[year]-[month]-[day]")?,
    )?)
}

impl From<PricedStock> for SummaryData {
    fn from(stock: PricedStock) -> Self {
        let current_value = stock.price * f64::from(stock.quantity);
        let last_value = stock.last_price * f64::from(stock.quantity);
        let original_cost = f64::from(stock.quantity) * stock.average_price;

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
