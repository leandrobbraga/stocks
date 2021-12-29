use std::collections::HashMap;

pub struct Portfolio {
    stocks: HashMap<String, u32>,
}

impl Portfolio {
    pub fn new() -> Self {
        Portfolio {
            stocks: HashMap::new(),
        }
    }

    pub fn buy(&mut self, symbol: &str, quantity: u32) {
        let entry = self.stocks.entry(symbol.into()).or_insert(0);
        *entry += quantity
    }

    pub fn sell(&mut self, symbol: &str, quantity: u32) -> Result<(), NotEnoughStockToSell> {
        if let Some(entry) = self.stocks.get_mut(symbol) {
            match (*entry).cmp(&quantity) {
                std::cmp::Ordering::Less => Err(NotEnoughStockToSell),
                std::cmp::Ordering::Equal => {
                    *entry = 0;
                    Ok(())
                }
                std::cmp::Ordering::Greater => {
                    *entry -= quantity;
                    Ok(())
                }
            }
        } else {
            Err(NotEnoughStockToSell)
        }
    }

    pub fn summary(&self) {
        println!("Summary");
        println!("-------");
        for (name, quantity) in &self.stocks {
            println!("Stock: {}, Quantity: {}", name, quantity);
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
