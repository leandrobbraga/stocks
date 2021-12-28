use std::{process::exit, str::FromStr};

fn main() {
    let command = match parse_arguments() {
        Ok(command) => command,
        Err(err) => {
            println!("Error: {}", err.0);
            println!();
            println!("Usage: ./stocks COMMAND SYMBOL VALUE");
            println!("Example: ./stocks buy bbas3 100");
            exit(1)
        }
    };

    println!("{:?}", command)
}

fn parse_arguments() -> Result<Arguments, ParseArgumentError> {
    let command: Command = std::env::args()
        .nth(1)
        .ok_or_else(|| ParseArgumentError("Missing COMMAND.".into()))?
        .parse()?;
    let symbol: String = std::env::args()
        .nth(2)
        .ok_or_else(|| ParseArgumentError("Missing SYMBOL.".into()))?;
    let value: u32 = std::env::args()
        .nth(3)
        .ok_or_else(|| ParseArgumentError("Missing VALUE.".into()))?
        .parse()
        .map_err(|_| {
            ParseArgumentError("Please provide a valid VALUE (i.e. an integer number).".into())
        })?;

    Ok(Arguments {
        command,
        symbol,
        value,
    })
}

#[derive(Debug)]
struct Arguments {
    command: Command,
    symbol: String,
    value: u32,
}

#[derive(Debug)]
enum Command {
    Buy,
    Sell,
}

impl FromStr for Command {
    type Err = ParseArgumentError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        match command.to_lowercase().as_ref() {
            "buy" => Ok(Command::Buy),
            "sell" => Ok(Command::Sell),
            _ => Err(ParseArgumentError(
                "This command was not implemented yet, try 'buy' or 'sell'.".into(),
            )),
        }
    }
}

#[derive(Debug)]
struct ParseArgumentError(String);
