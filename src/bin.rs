use structopt::{clap::arg_enum, StructOpt};

fn main() {
    let command = Arguments::from_args();
    println!("{:?}", command)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Stocks", about = "A simple CLI to manage stocks.")]
struct Arguments {
    #[structopt(
        name="COMMAND",
        possible_values=&Command::variants(),
        case_insensitive=true, help="The command that will be executed."
    )]
    command: Command,
    #[structopt(name = "SYMBOL", help = "The Stock ticker (e.g. BBAS3).")]
    symbol: String,
    #[structopt(
        name = "VALUE",
        help = "How much it is going to be bought or sold (e.g. 100)."
    )]
    value: u32,
}

arg_enum! {
    #[derive(Debug)]
    enum Command {
        Buy,
        Sell,
    }
}
