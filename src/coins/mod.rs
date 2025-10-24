use serde::{Deserialize, Serialize};
use std::fmt;
use async_trait::async_trait;

pub mod coinmarketcap;
pub mod coingecko;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    CHF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coin {
    ETH
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::CHF => write!(f, "CHF"),
        }
    }
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coin::ETH => write!(f, "ETH")
        }
    }
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::CHF => "CHF",
        }
    }

    pub fn all() -> &'static [Currency] {
        &[Currency::USD, Currency::EUR, Currency::CHF]
    }
}

impl Coin {
    pub fn coinmarketcap_id(&self) -> u32 {
        match self {
            Coin::ETH => 1027,
        }
    }

    pub fn coingecko_id(&self) -> &'static str {
        match self {
            Coin::ETH => "ethereum",
        }
    }

    pub fn all() -> &'static [Coin] {
        &[Coin::ETH]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub coin: Coin,
    pub currency: Currency,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait PriceProvider {
    type Error;
    
    async fn get_quotes(&self, coins: &[Coin], currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error>;
}