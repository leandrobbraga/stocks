use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Portfolio {
    assets: HashMap<String, UnpricedAsset>,
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

    pub fn buy(&mut self, symbol: &str, class: AssetClass, quantity: u32, price: f64) {
        match self.assets.get_mut(&symbol.to_uppercase()) {
            Some(entry) => {
                let p0 = entry.average_price;
                let q0 = entry.quantity as f64;
                let p1 = price;
                let q1 = quantity as f64;

                entry.average_price = ((p0 * q0) + (p1 * q1)) / (q0 + q1);
                entry.quantity += quantity;
            }
            None => {
                self.assets.insert(
                    symbol.to_uppercase(),
                    UnpricedAsset {
                        name: symbol.to_uppercase(),
                        class,
                        quantity,
                        average_price: price,
                    },
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

    pub fn assets(&self) -> Vec<UnpricedAsset> {
        self.assets.values().cloned().collect()
    }

    pub fn stock(&self, symbol: &str) -> Option<&UnpricedAsset> {
        self.assets.get(&symbol.to_uppercase())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnpricedAsset {
    pub name: String,
    pub class: AssetClass,
    pub quantity: u32,
    pub average_price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PricedAsset {
    pub name: String,
    pub class: AssetClass,
    pub quantity: u32,
    pub average_price: f64,
    pub price: f64,
    pub last_price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceInfo {
    #[serde(rename = "lastPrice")]
    pub price: f64,
    #[serde(rename = "closingPrice")]
    pub last_price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AssetClass {
    FII,
    Stock,
}

#[derive(Debug)]
pub struct NotEnoughAssetToSell;

#[derive(Default)]
pub struct StockMarket {
    client: reqwest::blocking::Client,
}

impl StockMarket {
    pub fn new() -> Self {
        StockMarket {
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn fetch_asset_price(&self, asset: UnpricedAsset) -> Result<PricedAsset, reqwest::Error> {
        let api = match asset.class {
            AssetClass::FII => "fiis",
            AssetClass::Stock => "stocks",
        };
        let price_info: PriceInfo = self
            .client
            .get(format!(
                "https://mfinance.com.br/api/v1/{}/{}",
                api,
                asset.name.to_lowercase()
            ))
            .send()?
            .json()?;

        Ok(PricedAsset {
            name: asset.name,
            class: asset.class,
            quantity: asset.quantity,
            average_price: asset.average_price,
            price: price_info.price,
            last_price: price_info.last_price,
        })
    }

    pub fn fetch_assets_price(
        &self,
        assets: Vec<UnpricedAsset>,
    ) -> Vec<Result<PricedAsset, reqwest::Error>> {
        assets
            .into_par_iter()
            .map(|asset| self.fetch_asset_price(asset))
            .collect()
    }

    pub fn asset_class(&self, asset: &str) -> Option<AssetClass> {
        let apis = ["fiis", "stocks"];

        let result: Vec<bool> = apis
            .into_par_iter()
            .map(|api| {
                if let Ok(list) = self.asset_list(api) {
                    list.contains(&asset.to_uppercase())
                } else {
                    false
                }
            })
            .collect();

        // Rayon maintains the original order
        match result[..] {
            [true, false] => Some(AssetClass::FII),
            [false, true] => Some(AssetClass::Stock),
            _ => None,
        }
    }

    fn asset_list(&self, api: &str) -> Result<Vec<String>, reqwest::Error> {
        let result: Vec<String> = self
            .client
            .get(format!("https://mfinance.com.br/api/v1/{}/symbols/", api,))
            .send()?
            .json()?;

        Ok(result)
    }
}
