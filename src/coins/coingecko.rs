//! CoinGecko price provider implementation.
//!
//! This module provides a price provider that fetches cryptocurrency prices
//! from the CoinGecko API. It supports both free and paid tiers - the API key
//! is optional for the free tier.
//!
//! # Examples
//!
//! ```rust
//! use boltzmann::coins::{Coin, Currency};
//! use boltzmann::coins::coingecko::CoinGecko;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = CoinGecko::new()?;
//! let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD]).await?;
//! 
//! println!("ETH price: ${:.2}", quotes[0].price);
//! # Ok(())
//! # }
//! ```

use reqwest::Client;
use serde_json::Value;
use async_trait::async_trait;
use crate::coins::{PriceProvider, Quote, Coin, Currency};

/// Error types that can occur when using the CoinGecko provider.
#[derive(Debug)]
pub enum CoinGeckoError {
    /// API returned an error or unexpected response format
    ApiError(String),
    /// HTTP request failed (network, timeout, etc.)
    RequestError(reqwest::Error),
    /// Failed to parse JSON response
    ParseError(serde_json::Error),
    /// Rate limit exceeded (HTTP 429)
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

/// CoinGecko price provider.
///
/// This struct handles fetching cryptocurrency prices from the CoinGecko API.
/// An API key is optional - the provider will work with the free tier without one,
/// but may have rate limits. Set `COINGECKO_API_KEY` environment variable for paid tier.
///
/// # Examples
///
/// ```rust
/// use boltzmann::coins::coingecko::CoinGecko;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = CoinGecko::new()?;
/// # Ok(())
/// # }
/// ```
pub struct CoinGecko {
    client: Client,
    api_key: Option<String>,
}

impl CoinGecko {
    /// Creates a new CoinGecko provider instance.
    ///
    /// Optionally reads the API key from the `COINGECKO_API_KEY` environment variable.
    /// If no API key is provided, the free tier will be used with rate limits.
    ///
    /// # Errors
    ///
    /// Returns `CoinGeckoError::ApiError` if the API key format is invalid.
    /// Returns `CoinGeckoError::RequestError` if the HTTP client cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boltzmann::coins::coingecko::CoinGecko;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Works with or without API key
    /// let provider = CoinGecko::new()?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Converts our Currency enum to CoinGecko's currency identifier.
    ///
    /// CoinGecko uses lowercase currency codes in their API.
    fn currency_to_coingecko_id(&self, currency: Currency) -> &'static str {
        match currency {
            Currency::USD => "usd",
            Currency::EUR => "eur", 
            Currency::CHF => "chf",
        }
    }

    /// Internal method to fetch quotes from CoinGecko API.
    ///
    /// Makes a single API call to fetch prices for one coin in multiple currencies.
    /// This is more efficient than making separate calls for each currency.
    ///
    /// # Arguments
    ///
    /// * `coin` - The cryptocurrency to fetch prices for
    /// * `currencies` - The fiat currencies to get prices in
    ///
    /// # Returns
    ///
    /// A vector of `Quote` objects, one for each requested currency.
    ///
    /// # Errors
    ///
    /// Returns various `CoinGeckoError` types if the request fails,
    /// rate limit is exceeded, or the response cannot be parsed.
    async fn fetch_quotes(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, CoinGeckoError> {
        if currencies.is_empty() {
            return Ok(Vec::new());
        }

        let coin_id = coin.coingecko_id();
        let currency_codes: Vec<String> = currencies.iter()
            .map(|c| self.currency_to_coingecko_id(*c).to_string())
            .collect();
        
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}&include_last_updated_at=true",
            coin_id,
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

        Ok(quotes)
    }
}

#[async_trait]
impl PriceProvider for CoinGecko {
    type Error = CoinGeckoError;

    async fn get_quotes(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error> {
        self.fetch_quotes(coin, currencies).await
    }
}