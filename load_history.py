"""This is an auxiliary script to load the history from the `history.csv` file and execute the
trades in the CLI."""
import argparse
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Load the trade history from a file and automatically executes the trades in the CLI."
    )

    parser.add_argument(
        "filepath",
        type=str,
        default="history.csv",
        help="The path to the file containing the trade history.",
    )

    args = parser.parse_args()

    history = load_history(filepath=args.filepath)

    for trade in history:
        command = trade["kind"]

        subprocess.run(
            f"./target/release/cli {command} {trade['symbol']} {trade['quantity']} {trade['price']} '{trade['date']}'",
            shell=True,
            check=True,
        )


def load_history(filepath: str) -> list:
    with open(filepath, "r") as file:
        lines = file.read().splitlines()
        history = []
        for i, line in enumerate(lines):
            if i == 0:
                continue

            symbol, date, kind, quantity, price = line.split(";")
            history.append(
                {
                    "symbol": symbol,
                    "date": date,
                    "kind": kind,
                    "price": price,
                    "quantity": quantity,
                }
            )
        return history


if __name__ == "__main__":
    main()
