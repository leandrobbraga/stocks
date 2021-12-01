pub mod database;
pub mod portfolio;

use portfolio::{NotEnoughStockToSell, Portfolio, Stock};

pub trait UserInterface {
    fn buy(
        &self,
        portfolio: Portfolio,
        stock: &str,
        quantity: u32,
        value: f64,
    ) -> Result<(), StockNotFound>;
    fn sell(
        &self,
        portfolio: Portfolio,
        stock: &str,
        quantity: u32,
        value: f64,
    ) -> Result<(), NotEnoughStockToSell>;
    fn summarize(&self, portfolio: Portfolio);
    fn load(&self, database: impl Database) -> Result<Portfolio, LoadError>;
    fn save(&self, portfolio: Portfolio, database: impl Database) -> Result<(), SaveError>;
    fn update(
        &self,
        portfolio: Portfolio,
        stock_market: impl StockMarketAPI,
    ) -> Result<Portfolio, UpdateError>;
}

pub trait Database {
    fn save(&self, portfolio: Portfolio) -> Result<(), SaveError>;
    fn load(&self) -> Result<Portfolio, LoadError>;
}

pub trait StockMarketAPI {
    fn fetch(&self, symbol: &str) -> Result<Stock, StockNotFound>;
}

pub struct StockNotFound;
pub struct LoadError;
pub struct SaveError;
pub struct UpdateError;
