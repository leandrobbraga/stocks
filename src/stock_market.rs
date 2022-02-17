use rayon::prelude::*;

use crate::portfolio::{AssetClass, PriceInfo, PricedAsset, UnpricedAsset};

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
