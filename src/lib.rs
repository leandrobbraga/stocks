use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

static FILEPATH: &str = "portfolio.json";

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

    pub fn from_file() -> Result<Self, IOError> {
        let filepath = Path::new(FILEPATH);

        if !filepath.exists() {
            return Err(IOError);
        }

        let file = File::open(filepath).map_err(|_| IOError)?;
        let reader = BufReader::new(file);

        let portfolio = serde_json::from_reader(reader).map_err(|_| IOError)?;

        Ok(portfolio)
    }

    pub fn save(&self) -> Result<(), IOError> {
        let filepath = Path::new(FILEPATH);

        let file = File::create(filepath).map_err(|_| IOError)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, &self).map_err(|_| IOError)
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

    pub fn summary(&self) {
        println!("Portfolio Summary");
        println!("-----------------");
        for (name, quantity) in &self.stocks {
            println!("Stock: {}, Quantity: {}", name.to_uppercase(), quantity);
        }
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct NotEnoughStockToSell;

#[derive(Debug)]
pub struct IOError;
