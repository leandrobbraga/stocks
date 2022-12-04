# Stocks

`Stocks` facilitates the monitoring of the portfolio directly from the terminal. With the `summary`
command it is possible to visualize the daily valuation of the stocks and also the valuation in
relation to the average price.

![Contains the `summary` command output, a table with the data for each owned stock](resources/summary.png)

It is also possible to calculate how much profit/loss was month by month for a specified year with
the command `profit-summary`.

![Contains the `profit-summary` command output, a table with the profit for each month of a given year](resources/profit-summary.png)

**The current implementation is aimed to stocks listed in the Brazilian market (Bovespa).**

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

## Usage Example

### Purchasing stocks

```shell
cargo run -- buy BBAS3 100 34.50
[2022-12-03T22:48:09Z INFO  cli::app] You bought 100 BBAS3 at R$     34.50.
```

It is also possible to define an optional previous date, but the program will only accept trades
that are more recent that the newest trade in the specific stock's trade history. Otherwise, it
would miscalculate the profit/loss for the trade. That's because the current implementation does not
recalculate the `average_purchase_price` in retrospect.

```shell
cargo run -- buy ITSA3 100 10.12 2022-01-01
[2022-12-03T22:48:09Z INFO  cli::app] You bought 100 ITSA3 at R$     10.12.
```

### Summarizing current position

```shell
cargo run -- summary
> Name  Quantity  Current Price Current Value Change (Day)  % Change (Day)  Average Price Profit    % Profit 
> BBAS3 100  R$   36.03         R$ 3603.00    R$ 0.00       0.00%           R$ 34.50      R$ 151.00 4,43% 
```

### Selling stocks

```shell
cargo run -- sell BBAS3 100 36.03
> [2022-12-03T22:48:09Z INFO  cli::app] You sold 100 BBAS3 profiting R$    151.00.
```

### Summarizing the profits in a year

This command calculates the portfolio profit for every month in a given year.

```shell
cargo run -- profit--summary 2022
> Month     Profit     
> 1      R$     170.00 
> 2      R$      81.00 
> 3      R$    2472.00 
> 4      R$    3333.00 
> 5      R$    4214.00 
> 6      R$     455.20 
> 7      R$       0.00 
> 8      R$     540.00 
> 9      R$       0.00 
> 10     R$   -1178.65 
> 11     R$   -6924.35 
> 12     R$       0.00 
> Total  R$    3162.21 
```
