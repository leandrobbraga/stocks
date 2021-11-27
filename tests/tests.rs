#![cfg(test)]
use stocks::*;

#[test]
fn test_portfolio_buy_and_sell() {
    let mut portfolio = Portfolio::new();
    let bbas3 = "BBAS3".into();
    
    portfolio.buy(&bbas3, 100, 10.0);
    portfolio.buy(&bbas3, 200, 15.0);

    assert_eq!(
        portfolio.stock(&bbas3),
        Stock {
            symbol: "BBAS3".into(),
            quantity: 300
        }
    );

    portfolio.sell(&bbas3, 200, 5.0).unwrap();
    assert_eq!(
        portfolio.stock(&bbas3),
        Stock {
            symbol: "BBAS3".into(),
            quantity: 100
        }
    );

    portfolio.sell(&bbas3, 100, 5.0).unwrap();
    assert_eq!(
        portfolio.stock(&bbas3),
        Stock {
            symbol: "BBAS3".into(),
            quantity: 0
        }
    );

    assert_eq!(
        portfolio.sell(&bbas3, 100, 5.0).err().unwrap(),
        NotEnoughStockToSell
    );
    assert_eq!(
        portfolio.stock(&bbas3),
        Stock {
            symbol: "BBAS3".into(),
            quantity: 0
        }
    );
}
