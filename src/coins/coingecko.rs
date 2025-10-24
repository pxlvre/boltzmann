use reqwest::Client;
use serde_json::Value;
use async_trait::async_trait;
use crate::coins::{PriceProvider, Quote, Coin, Currency};

#[derive(Debug)]
pub enum CoinGeckoError {
    ApiError(String),
    RequestError(reqwest::Error),
    ParseError(serde_json::Error),
    RateLimitError,
}

impl std::fmt::Display for CoinGeckoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinGeckoError::ApiError(msg) => write!(f, "API Error: {}", msg),
            CoinGeckoError::RequestError(e) => write!(f, "Request Error: {}", e),
            CoinGeckoError::ParseError(e) => write!(f, "Parse Error: {}", e),
            CoinGeckoError::RateLimitError => write!(f, "Rate limit exceeded"),
        }
    }
}

impl std::error::Error for CoinGeckoError {}

pub struct CoinGecko {
    client: Client,
    api_key: Option<String>,
}

impl CoinGecko {
    pub fn new() -> Result<Self, CoinGeckoError> {
        let api_key = std::env::var("COINGECKO_API_KEY").ok();
        
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(ref key) = api_key {
            headers.insert(
                "x-cg-demo-api-key",
                reqwest::header::HeaderValue::from_str(key)
                    .map_err(|_| CoinGeckoError::ApiError("Invalid API key format".to_string()))?
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(CoinGeckoError::RequestError)?;

        Ok(Self { client, api_key })
    }

    fn currency_to_coingecko_id(&self, currency: Currency) -> &'static str {
        match currency {
            Currency::USD => "usd",
            Currency::EUR => "eur", 
            Currency::CHF => "chf",
        }
    }

    async fn fetch_quotes(&self, coins: &[Coin], currencies: &[Currency]) -> Result<Vec<Quote>, CoinGeckoError> {
        if coins.is_empty() || currencies.is_empty() {
            return Ok(Vec::new());
        }

        let coin_ids: Vec<String> = coins.iter().map(|c| c.coingecko_id().to_string()).collect();
        let currency_codes: Vec<String> = currencies.iter()
            .map(|c| self.currency_to_coingecko_id(*c).to_string())
            .collect();
        
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}&include_last_updated_at=true",
            coin_ids.join(","),
            currency_codes.join(",")
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(CoinGeckoError::RequestError)?;

        if response.status() == 429 {
            return Err(CoinGeckoError::RateLimitError);
        }

        let body = response
            .text()
            .await
            .map_err(CoinGeckoError::RequestError)?;

        let json: Value = serde_json::from_str(&body)
            .map_err(CoinGeckoError::ParseError)?;

        let mut quotes = Vec::new();
        let timestamp = chrono::Utc::now();

        for &coin in coins {
            let coin_id = coin.coingecko_id();
            let coin_data = &json[coin_id];
            
            if coin_data.is_null() {
                return Err(CoinGeckoError::ApiError(
                    format!("No data found for coin {}", coin)
                ));
            }

            for &currency in currencies {
                let currency_code = self.currency_to_coingecko_id(currency);
                
                if let Some(price) = coin_data[currency_code].as_f64() {
                    quotes.push(Quote {
                        coin,
                        currency,
                        price,
                        timestamp,
                    });
                } else {
                    return Err(CoinGeckoError::ApiError(
                        format!("Price not found for {} in {}", coin, currency)
                    ));
                }
            }
        }

        Ok(quotes)
    }
}

#[async_trait]
impl PriceProvider for CoinGecko {
    type Error = CoinGeckoError;

    async fn get_quotes(&self, coins: &[Coin], currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error> {
        self.fetch_quotes(coins, currencies).await
    }
}

impl CoinGecko {
    // Convenience methods for common use cases
    pub async fn get_quote(&self, coin: Coin, currency: Currency) -> Result<Quote, CoinGeckoError> {
        let quotes = self.get_quotes(&[coin], &[currency]).await?;
        quotes.into_iter().next()
            .ok_or_else(|| CoinGeckoError::ApiError("No quote returned".to_string()))
    }

    pub async fn get_coin_in_multiple_currencies(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, CoinGeckoError> {
        self.get_quotes(&[coin], currencies).await
    }

    pub async fn get_multiple_coins_in_currency(&self, coins: &[Coin], currency: Currency) -> Result<Vec<Quote>, CoinGeckoError> {
        self.get_quotes(coins, &[currency]).await
    }
}