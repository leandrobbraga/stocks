# Stocks

`Stocks` facilitates the monitoring of the portfolio directly from the terminal. With the `summary`
command it is possible to visualize the daily valuation of the stocks and also the valuation in
relation to the average price.

![Contains the `summary` command output, a table with the data for each owned stock](resources/summary.png)

It is also possible to calculate how much profit/loss was month by month for a specified year with
the command `profit-summary`.

![Contains the `profit-summary` command output, a table with the profit for each month of a given year](resources/profit-summary.png)

NOTES:

1. **The current implementation is aimed to stocks listed in the Brazilian market (BOVESPA).**
1. **It does not support day trades, it will consider all transactions as being swing trades.**

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

## Usage Example

### Purchasing stocks

```shell
cargo run -- buy BBAS3 100 34.50
[2022-12-03T22:48:09Z INFO  cli::app] You bought 100 BBAS3 at R$     34.50.
```

It is also possible to define an optional previous date.

```shell
cargo run -- buy ITSA3 100 10.12 2022-01-01 10:00:00
[2022-12-03T22:48:09Z INFO  cli::app] You bought 100 ITSA3 at R$     10.12.
```

### Summarizing current position

```shell
cargo run -- summary
> Name  Quantity  Current Price Current Value Change (Day)  % Change (Day)  Average Price Profit    % Profit 
> BBAS3 100  R$   36.03         R$ 3603.00    R$ 0.00       0.00%           R$ 34.50      R$ 151.00 4,43% 
```

It is also possible to see the summary for a specific reference date. This is useful for calculating
end of year position for tax purposes.

```shell
cargo run -- summary 2022-12-31
```

NOTE: Be careful when using this command, as the `Current Price` will not be the reference date
price, but the actual current price.

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

### Performing Stock Split

In case of stock split events, it is possible to update the stock quantity and average purchase
price to reflect the new quantity.

```shell
cargo run -- split BBAS3 2
> [2022-12-03T22:48:09Z INFO  cli::app] You performed a 2:1 stock split for BBAS3.
```

NOTE: The current implementation will update all the trade history for the stock, losing the correct
information about the original quantity / purchase price at a given time. Be careful when using this
to report the correct information to the tax authorities.
