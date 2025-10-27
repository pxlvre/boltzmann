//! CoinMarketCap price provider implementation.
//!
//! This module provides a price provider that fetches cryptocurrency prices
//! from the CoinMarketCap API. It requires a valid API key set in the
//! `COINMARKETCAP_API_KEY` environment variable.
//!
//! # Examples
//!
//! ```rust
//! use boltzmann::coins::{Coin, Currency};
//! use boltzmann::coins::coinmarketcap::CoinMarketCap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = CoinMarketCap::new()?;
//! let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD]).await?;
//!
//! println!("ETH price: ${:.2}", quotes[0].price);
//! # Ok(())
//! # }
//! ```

use crate::coins::{Coin, Currency, PriceProvider, Quote, QuotePerAmount, ProviderSource};
use async_trait::async_trait;
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use std::env;

/// Error types that can occur when using the CoinMarketCap provider.
#[derive(Debug)]
pub enum CoinMarketCapError {
    /// API returned an error or unexpected response format
    ApiError(String),
    /// HTTP request failed (network, timeout, etc.)
    RequestError(reqwest::Error),
    /// Failed to parse JSON response
    ParseError(serde_json::Error),
    /// Environment variable missing or invalid
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

/// CoinMarketCap price provider.
///
/// This struct handles fetching cryptocurrency prices from the CoinMarketCap API.
/// It requires a valid API key to be set in the `COINMARKETCAP_API_KEY` environment variable.
///
/// # Examples
///
/// ```rust
/// use boltzmann::coins::coinmarketcap::CoinMarketCap;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = CoinMarketCap::new()?;
/// # Ok(())
/// # }
/// ```
pub struct CoinMarketCap {
    #[allow(dead_code)] // Used indirectly via client default headers
    api_key: String,
    client: Client,
}

impl CoinMarketCap {
    /// Creates a new CoinMarketCap provider instance.
    ///
    /// Reads the API key from the `COINMARKETCAP_API_KEY` environment variable
    /// and sets up an HTTP client with the required headers.
    ///
    /// # Errors
    ///
    /// Returns `CoinMarketCapError::EnvError` if the API key is not set or invalid.
    /// Returns `CoinMarketCapError::RequestError` if the HTTP client cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boltzmann::coins::coinmarketcap::CoinMarketCap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = CoinMarketCap::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self, CoinMarketCapError> {
        dotenv().ok();

        let api_key = env::var("COINMARKETCAP_API_KEY").map_err(|_| {
            CoinMarketCapError::EnvError("COINMARKETCAP_API_KEY not set in environment".to_string())
        })?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-CMC_PRO_API_KEY",
            reqwest::header::HeaderValue::from_str(&api_key)
                .map_err(|_| CoinMarketCapError::EnvError("Invalid API key format".to_string()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(CoinMarketCapError::RequestError)?;

        Ok(Self { api_key, client })
    }

    /// Internal method to fetch quotes from CoinMarketCap API.
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
    /// Returns various `CoinMarketCapError` types if the request fails or
    /// the response cannot be parsed.
    async fn fetch_quotes(
        &self,
        coin: Coin,
        currencies: &[Currency],
    ) -> Result<Vec<Quote>, CoinMarketCapError> {
        if currencies.is_empty() {
            return Ok(Vec::new());
        }

        let coin_id = coin.coinmarketcap_id();
        let currency_codes: Vec<String> = currencies
            .iter()
            .map(|c| c.to_string().to_uppercase())
            .collect();

        let url = format!(
            "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id={}&convert={}",
            coin_id,
            currency_codes.join(",")
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(CoinMarketCapError::RequestError)?;

        let body = response
            .text()
            .await
            .map_err(CoinMarketCapError::RequestError)?;

        let json: Value = serde_json::from_str(&body).map_err(CoinMarketCapError::ParseError)?;

        let mut quotes = Vec::new();
        let timestamp = chrono::Utc::now();

        let coin_id_str = coin_id.to_string();
        let coin_data = &json["data"][&coin_id_str];

        if coin_data.is_null() {
            return Err(CoinMarketCapError::ApiError(format!(
                "No data found for coin {}",
                coin
            )));
        }

        for &currency in currencies {
            let currency_code = currency.to_string().to_uppercase();
            let quote_data = &coin_data["quote"][&currency_code];

            if let Some(price) = quote_data["price"].as_f64() {
                quotes.push(Quote {
                    coin,
                    currency,
                    price,
                    provider: ProviderSource::CoinMarketCap,
                    timestamp,
                    quote_per_amount: QuotePerAmount {
                        amount: 1.0,
                        total_price: price,
                    },
                });
            } else {
                return Err(CoinMarketCapError::ApiError(format!(
                    "Price not found for {} in {}",
                    coin, currency
                )));
            }
        }

        Ok(quotes)
    }
}

#[async_trait]
impl PriceProvider for CoinMarketCap {
    type Error = CoinMarketCapError;

    async fn get_quotes(
        &self,
        coin: Coin,
        currencies: &[Currency],
    ) -> Result<Vec<Quote>, Self::Error> {
        self.fetch_quotes(coin, currencies).await
    }
}
