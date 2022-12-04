use super::portfolio::Stock;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::task::JoinHandle;

const API_URL: &str = "https://mfinance.com.br/api/v1/stocks/";

/// Represents the stock market, it's responsible for fetching real stock information.
#[derive(Default)]
pub struct StockMarket {
    client: reqwest::Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PricedStock {
    pub symbol: String,
    pub quantity: u32,
    pub average_price: f64,
    pub price: f64,
    pub last_price: f64,
}

/// The complete response from the MFinance API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MFinanceResponse {
    pub change: f64,
    pub closing_price: f64,
    pub eps: f64,
    pub high: f64,
    pub last_price: f64,
    pub last_year_high: f64,
    pub last_year_low: f64,
    pub low: f64,
    pub market_cap: f64,
    pub name: String,
    pub pe: f64,
    pub price_open: f64,
    pub sector: String,
    pub segment: String,
    pub shares: f64,
    pub sub_sector: String,
    pub symbol: String,
    pub volume: f64,
    pub volume_avg: f64,
}

impl StockMarket {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Given a slice of stocks, fetches current information about them from the stock market.
    pub async fn get_stock_prices(&self, stocks: &[&Stock]) -> Result<Vec<Result<PricedStock>>> {
        let mut handles: Vec<JoinHandle<Result<PricedStock>>> = Vec::with_capacity(stocks.len());

        for stock in stocks {
            let handle = tokio::spawn(StockMarket::get_stock_price(
                self.client.clone(),
                stock.symbol.to_string(),
                stock.quantity,
                stock.average_purchase_price,
            ));

            handles.push(handle);
        }

        let mut results = Vec::with_capacity(stocks.len());
        for handle in handles {
            results.push(handle.await?);
        }

        Ok(results)
    }

    /// Fetches current information about a stock from the stock market.
    async fn get_stock_price(
        client: reqwest::Client,
        symbol: String,
        quantity: u32,
        average_price: f64,
    ) -> Result<PricedStock> {
        let response = client.get(format!("{API_URL}/{symbol}")).send().await?;
        let response: MFinanceResponse = response.json().await?;
        Ok(PricedStock {
            symbol,
            quantity,
            average_price,
            price: response.last_price,
            last_price: response.closing_price,
        })
    }
}
