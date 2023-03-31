use super::portfolio::Stock;
use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use ureq::Agent;

const API_URL: &str = "https://mfinance.com.br/api/v1/stocks/";

/// Represents the stock market, it's responsible for fetching real stock information.
pub struct StockMarket {
    client: Agent,
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
            client: Agent::new(),
        }
    }

    /// Given a slice of stocks, fetches current information about them from the stock market.
    pub fn get_stock_prices(
        &self,
        stocks: &[&Stock],
        date: NaiveDateTime,
    ) -> Vec<Result<PricedStock>> {
        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(stocks.len());

            for stock in stocks {
                let handle = s.spawn(|| {
                    StockMarket::get_stock_price(
                        self.client.clone(),
                        stock.symbol.as_str(),
                        stock.quantity(date),
                        stock.average_purchase_price(date),
                    )
                });
                handles.push(handle);
            }

            handles
                .into_iter()
                .map(|handle| {
                    handle.join().unwrap_or_else(|err| {
                        Err(anyhow::anyhow!("Failed to fetch stock price: {err:?}"))
                    })
                })
                .collect()
        })
    }

    /// Fetches current information about a stock from the stock market.
    fn get_stock_price(
        client: Agent,
        symbol: &str,
        quantity: u32,
        average_price: f64,
    ) -> Result<PricedStock> {
        let response = client.get(format!("{API_URL}/{symbol}").as_str()).call()?;
        let response: MFinanceResponse = response.into_json()?;
        Ok(PricedStock {
            symbol: response.symbol,
            quantity,
            average_price,
            price: response.last_price,
            last_price: response.closing_price,
        })
    }
}
