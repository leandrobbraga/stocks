#![cfg(test)]
use stocks::portfolio::*;

#[test]
fn test_portfolio_buy_and_sell() {
    let mut portfolio = Portfolio::default();
    let bbas3 = "BBAS3";

    portfolio.buy(bbas3, 100, 100);
    portfolio.buy(bbas3, 200, 150);

    assert_eq!(portfolio.stock(bbas3).unwrap(), Stock::new(bbas3, 300));

    portfolio.sell(bbas3, 200, 50).unwrap();
    assert_eq!(portfolio.stock(bbas3).unwrap(), Stock::new(bbas3, 100));

    // If we try to sell more than we have we get a NotEnoughStockToSell error
    assert_eq!(
        portfolio.sell(bbas3, 150, 50).err().unwrap(),
        NotEnoughStockToSell
    );

    // Now we are going to close the position in BBAS3 and the portfolio should return no Stock when
    // asked to.
    portfolio.sell(bbas3, 100, 50).unwrap();
    assert_eq!(portfolio.stock(bbas3), None);

    // If we try to sell something that we don't have we get a NotEnoughStockToSell error as well
    assert_eq!(
        portfolio.sell(bbas3, 100, 50).err().unwrap(),
        NotEnoughStockToSell
    );
    assert_eq!(portfolio.stock(bbas3), None);
}
