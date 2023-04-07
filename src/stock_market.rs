use super::portfolio::Stock;
use anyhow::Result;
use serde::Deserialize;
use time::OffsetDateTime;
use ureq::Agent;

const API_URL: &str = "https://mfinance.com.br/api/v1/stocks/";

/// Represents the stock market, it's responsible for fetching real stock information.
pub struct StockMarket {
    client: Agent,
}

#[derive(Deserialize)]
pub struct PricedStock {
    pub symbol: String,
    pub quantity: u32,
    pub average_price: f64,
    pub price: f64,
    pub last_price: f64,
}

/// The complete response from the `MFinance` API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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

    pub fn get_stock_prices(
        &self,
        stocks: &[Stock],
        date: OffsetDateTime,
    ) -> Vec<Result<PricedStock>> {
        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(stocks.len());

            for stock in stocks {
                let handle = s.spawn(|| {
                    let response = self
                        .client
                        .get(format!("{API_URL}/{}", stock.symbol).as_str())
                        .call()?;

                    let response: MFinanceResponse = response.into_json()?;

                    Ok(PricedStock {
                        symbol: response.symbol,
                        quantity: stock.quantity(date),
                        average_price: stock.average_purchase_price(date),
                        price: response.last_price,
                        last_price: response.closing_price,
                    })
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
}
