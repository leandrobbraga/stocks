use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

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

        let portfolio = serde_json::from_reader(reader)?;

        Ok(portfolio)
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
