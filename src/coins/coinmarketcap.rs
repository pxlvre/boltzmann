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
        
        let api_key = env::var("COINMARKETCAP_API_KEY")
            .map_err(|_| CoinMarketCapError::EnvError("COINMARKETCAP_API_KEY not set in environment".to_string()))?;

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

    async fn fetch_quotes(&self, coins: &[Coin], currencies: &[Currency]) -> Result<Vec<Quote>, CoinMarketCapError> {
        if coins.is_empty() || currencies.is_empty() {
            return Ok(Vec::new());
        }

        let coin_ids: Vec<String> = coins.iter().map(|c| c.coinmarketcap_id().to_string()).collect();
        let currency_codes: Vec<String> = currencies.iter().map(|c| c.to_string().to_uppercase()).collect();
        
        let url = format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={}&convert={}",
            coin_ids.join(","),
            currency_codes.join(",")
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

        let mut quotes = Vec::new();
        let timestamp = chrono::Utc::now();

        for &coin in coins {
            let coin_id = coin.coinmarketcap_id().to_string();
            let coin_data = &json["data"][&coin_id];
            
            if coin_data.is_null() {
                return Err(CoinMarketCapError::ApiError(
                    format!("No data found for coin {}", coin)
                ));
            }

            for &currency in currencies {
                let currency_code = currency.to_string().to_uppercase();
                let quote_data = &coin_data["quote"][&currency_code];
                
                if let Some(price) = quote_data["price"].as_f64() {
                    quotes.push(Quote {
                        coin,
                        currency,
                        price,
                        timestamp,
                    });
                } else {
                    return Err(CoinMarketCapError::ApiError(
                        format!("Price not found for {} in {}", coin, currency)
                    ));
                }
            }
        }

        Ok(quotes)
    }
}

#[async_trait]
impl PriceProvider for CoinMarketCap {
    type Error = CoinMarketCapError;

    async fn get_quotes(&self, coins: &[Coin], currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error> {
        self.fetch_quotes(coins, currencies).await
    }
}

impl CoinMarketCap {
    // Convenience methods for common use cases
    pub async fn get_quote(&self, coin: Coin, currency: Currency) -> Result<Quote, CoinMarketCapError> {
        let quotes = self.get_quotes(&[coin], &[currency]).await?;
        quotes.into_iter().next()
            .ok_or_else(|| CoinMarketCapError::ApiError("No quote returned".to_string()))
    }

    pub async fn get_coin_in_multiple_currencies(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, CoinMarketCapError> {
        self.get_quotes(&[coin], currencies).await
    }

    pub async fn get_multiple_coins_in_currency(&self, coins: &[Coin], currency: Currency) -> Result<Vec<Quote>, CoinMarketCapError> {
        self.get_quotes(coins, &[currency]).await
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