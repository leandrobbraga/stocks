"""This is an auxiliary script to load the history from the `history.csv` file and execute the
trades in the CLI."""
import subprocess


def main() -> None:
    history = load_history()

    for trade in history:
        command = "buy" if trade["kind"] == "C" else "sell"

        # Switch the date from "01/01/2021" to "2021-01-01
        date = "-".join(trade['date'].split("/")[::-1])
        subprocess.run(f"./target/release/cli {command} {trade['symbol']} {trade['quantity']} {trade['price']} {date}", shell=True, check=True)



def load_history() -> list:
    with open("history.csv", "r") as file:
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