use std::env;
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use async_trait::async_trait;
use crate::coins::{PriceProvider, Quote, Coin, Currency};

#[derive(Debug)]
pub enum CoinMarketCapError {
    ApiError(String),
    RequestError(reqwest::Error),
    ParseError(serde_json::Error),
    EnvError(String),
}

impl std::fmt::Display for CoinMarketCapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinMarketCapError::ApiError(msg) => write!(f, "API Error: {}", msg),
            CoinMarketCapError::RequestError(e) => write!(f, "Request Error: {}", e),
            CoinMarketCapError::ParseError(e) => write!(f, "Parse Error: {}", e),
            CoinMarketCapError::EnvError(msg) => write!(f, "Environment Error: {}", msg),
        }
    }
}

impl std::error::Error for CoinMarketCapError {}

pub struct CoinMarketCap {
    api_key: String,
    client: Client,
}

impl CoinMarketCap {
    pub fn new() -> Result<Self, CoinMarketCapError> {
        dotenv().ok();
        
        let api_key = env::var("API_KEY")
            .map_err(|_| CoinMarketCapError::EnvError("API_KEY not set in environment".to_string()))?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-CMC_PRO_API_KEY", 
            reqwest::header::HeaderValue::from_str(&api_key)
                .map_err(|_| CoinMarketCapError::EnvError("Invalid API key format".to_string()))?
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(CoinMarketCapError::RequestError)?;

        Ok(Self { api_key, client })
    }

    async fn fetch_quote(&self, coin: Coin, currency: Currency) -> Result<Quote, CoinMarketCapError> {
        let coin_id = coin.coinmarketcap_id();
        let currency_code = currency.to_string().to_uppercase();
        
        let url = format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={}&convert={}",
            coin_id, currency_code
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(CoinMarketCapError::RequestError)?;

        let body = response
            .text()
            .await
            .map_err(CoinMarketCapError::RequestError)?;

        let json: Value = serde_json::from_str(&body)
            .map_err(CoinMarketCapError::ParseError)?;

        let price = json["data"][coin_id.to_string()]["quote"][currency_code]["price"]
            .as_f64()
            .ok_or_else(|| CoinMarketCapError::ApiError("Price not found in response".to_string()))?;

        Ok(Quote {
            coin,
            currency,
            price,
            timestamp: chrono::Utc::now(),
        })
    }
}

#[async_trait]
impl PriceProvider for CoinMarketCap {
    type Error = CoinMarketCapError;

    async fn get_quote(&self, coin: Coin, currency: Currency) -> Result<Quote, Self::Error> {
        self.fetch_quote(coin, currency).await
    }

    async fn get_multiple_quotes(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error> {
        let mut quotes = Vec::new();
        
        for &currency in currencies {
            let quote = self.fetch_quote(coin, currency).await?;
            quotes.push(quote);
        }
        
        Ok(quotes)
    }
}

// Legacy function for backward compatibility
pub async fn get_quote() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let provider = CoinMarketCap::new()?;
    let quote = provider.get_quote(Coin::ETH, Currency::USD).await?;
    
    println!("1 {} is {}{}", quote.coin, quote.currency.symbol(), quote.price);
    
    Ok(serde_json::json!({
        "coin": quote.coin,
        "currency": quote.currency,
        "price": quote.price,
        "timestamp": quote.timestamp
    }))
}