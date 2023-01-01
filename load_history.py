"""This is an auxiliary script to load the history from the `history.csv` file and execute the
trades in the CLI."""
import datetime
import subprocess


def main() -> None:
    history = load_history()

    for i, trade in enumerate(history):
        command = "buy" if trade["kind"] == "C" else "sell"

        date = datetime.datetime.strptime(trade['date'], "%d/%m/%Y")
        # Add 1 second to each trade to avoid the same timestamp
        date = date + datetime.timedelta(seconds=i)

        subprocess.run(f"./target/release/cli {command} {trade['symbol']} {trade['quantity']} {trade['price']} '{date.strftime('%Y-%m-%d %H:%M:%S')}'", shell=True, check=True)



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