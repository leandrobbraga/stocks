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
    assets: HashMap<String, Asset<Unpriced>>,
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            assets: HashMap::new(),
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

    pub fn buy(&mut self, symbol: &str, quantity: u32) {
        match self.assets.get_mut(&symbol.to_uppercase()) {
            Some(entry) => entry.quantity += quantity,
            None => {
                let class = if symbol.ends_with("11") {
                    AssetClass::FII
                } else {
                    AssetClass::Stock
                };

                self.assets.insert(
                    symbol.to_uppercase(),
                    Asset::<Unpriced>::new(symbol.to_uppercase(), class, quantity),
                );
            }
        }
    }

    pub fn sell(&mut self, symbol: &str, quantity: u32) -> Result<(), NotEnoughAssetToSell> {
        if let Some(entry) = self.assets.get_mut(&symbol.to_uppercase()) {
            match (*entry).quantity.cmp(&quantity) {
                std::cmp::Ordering::Less => Err(NotEnoughAssetToSell),
                std::cmp::Ordering::Equal => {
                    self.assets.remove(symbol);
                    Ok(())
                }
                std::cmp::Ordering::Greater => {
                    (*entry).quantity -= quantity;
                    Ok(())
                }
            }
        } else {
            Err(NotEnoughAssetToSell)
        }
    }

    pub fn assets(&self) -> Vec<Asset<Unpriced>> {
        self.assets.values().cloned().collect()
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset<PriceInfo> {
    pub name: String,
    pub class: AssetClass,
    pub quantity: u32,
    pub price_info: PriceInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Unpriced;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Priced {
    #[serde(rename = "lastPrice")]
    pub price: f64,
    #[serde(rename = "closingPrice")]
    pub last_price: f64,
}

impl Asset<Unpriced> {
    fn new(name: String, class: AssetClass, quantity: u32) -> Self {
        Asset {
            name,
            class,
            quantity,
            price_info: Unpriced,
        }
    }
}

impl Asset<Priced> {
    fn new(name: String, class: AssetClass, quantity: u32, price_info: Priced) -> Self {
        Asset {
            name,
            class,
            quantity,
            price_info,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AssetClass {
    FII,
    Stock,
}

#[derive(Debug)]
pub struct NotEnoughAssetToSell;

pub struct StockMarket {
    runtime: tokio::runtime::Runtime,
    client: reqwest::Client,
}

impl StockMarket {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(StockMarket {
            runtime: tokio::runtime::Runtime::new()?,
            client: reqwest::Client::new(),
        })
    }

    pub fn fetch_assets_price(
        &self,
        assets: Vec<Asset<Unpriced>>,
    ) -> Result<Vec<Asset<Priced>>, Box<dyn Error>> {
        self.runtime.block_on(self.async_fetch_assets_info(assets))
    }

    async fn async_fetch_assets_info(
        &self,
        assets: Vec<Asset<Unpriced>>,
    ) -> Result<Vec<Asset<Priced>>, Box<dyn Error>> {
        let mut tasks: Vec<JoinHandle<Result<Asset<Priced>, reqwest::Error>>> = vec![];
        let mut result: Vec<Asset<Priced>> = vec![];

        for asset in assets {
            tasks.push(tokio::spawn(StockMarket::async_fetch_asset_info(
                asset,
                self.client.clone(),
            )));
        }

        for task in tasks {
            result.push(task.await??);
        }

        Ok(result)
    }

    async fn async_fetch_asset_info(
        asset: Asset<Unpriced>,
        client: reqwest::Client,
    ) -> Result<Asset<Priced>, reqwest::Error> {
        let api = match asset.class {
            AssetClass::FII => "fiis",
            AssetClass::Stock => "stocks",
        };
        let price_info: Priced = client
            .get(format!(
                "https://mfinance.com.br/api/v1/{}/{}",
                api,
                asset.name.to_lowercase()
            ))
            .send()
            .await?
            .json()
            .await?;

        Ok(Asset::<Priced>::new(
            asset.name,
            asset.class,
            asset.quantity,
            price_info,
        ))
    }
}
