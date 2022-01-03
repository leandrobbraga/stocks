use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Debug)]
pub struct Portfolio {
    stocks: HashMap<String, u32>,
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            stocks: HashMap::new(),
        }
    }

    pub fn from_file(filepath: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }

    pub fn to_file(&self, filepath: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);

        Ok(serde_json::to_writer(writer, &self)?)
    }

    pub fn buy(&mut self, symbol: &str, quantity: u32) -> u32 {
        let entry = self.stocks.entry(symbol.to_uppercase()).or_insert(0);
        *entry += quantity;
        *entry
    }

    pub fn sell(&mut self, symbol: &str, quantity: u32) -> Result<u32, NotEnoughStockToSell> {
        if let Some(entry) = self.stocks.get_mut(&symbol.to_uppercase()) {
            match (*entry).cmp(&quantity) {
                std::cmp::Ordering::Less => Err(NotEnoughStockToSell),
                std::cmp::Ordering::Equal => {
                    self.stocks.remove(symbol);
                    Ok(0)
                }
                std::cmp::Ordering::Greater => {
                    *entry -= quantity;
                    Ok(*entry)
                }
            }
        } else {
            Err(NotEnoughStockToSell)
        }
    }

    pub fn summary(&self) -> Result<Vec<Stock>, Box<dyn Error>> {
        let tokio = tokio::runtime::Runtime::new()?;
        let client = reqwest::Client::new();
        tokio.block_on(self.fetch_stock_prices(client))
    }

    async fn fetch_stock_prices(&self, client: Client) -> Result<Vec<Stock>, Box<dyn Error>> {
        let mut tasks: Vec<JoinHandle<Result<Stock, reqwest::Error>>> = vec![];
        let mut stocks: Vec<Stock> = vec![];

        for (name, quantity) in &self.stocks {
            tasks.push(tokio::spawn(Stock::from_api(
                name.clone(),
                client.clone(),
                *quantity,
            )));
        }

        for task in tasks {
            stocks.push(task.await??);
        }

        Ok(stocks)
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct Stock {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub price: f64,
    #[serde(rename = "closingPrice")]
    pub last_price: f64,
    #[serde(skip)]
    pub quantity: u32,
}

impl Stock {
    async fn from_api(
        symbol: String,
        client: Client,
        quantity: u32,
    ) -> Result<Stock, reqwest::Error> {
        let api = if symbol.chars().into_iter().next_back() == Some('1') {
            "fiis"
        } else {
            "stocks"
        };

        let mut stock: Stock = client
            .get(format!(
                "https://mfinance.com.br/api/v1/{}/{}",
                api,
                symbol.to_lowercase()
            ))
            .send()
            .await?
            .json()
            .await?;

        stock.quantity = quantity;
        Ok(stock)
    }
}

#[derive(Debug)]
pub struct NotEnoughStockToSell;
