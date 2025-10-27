//! CoinMarketCap price provider implementation.
//!
//! This module provides a price provider that fetches cryptocurrency prices
//! from the CoinMarketCap API. It requires a valid API key set in the
//! `COINMARKETCAP_API_KEY` environment variable.
//!
//! # Examples
//!
//! ```rust
//! use boltzmann::crypto::{Coin, Currency};
//! use boltzmann::crypto::coinmarketcap::CoinMarketCap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = CoinMarketCap::new()?;
//! let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD]).await?;
//!
//! println!("ETH price: ${:.2}", quotes[0].price);
//! # Ok(())
//! # }
//! ```

use super::{Coin, Currency, PriceProvider, Quote, QuotePerAmount, ProviderSource};
use crate::core::errors::{Result, ErrorContext};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use anyhow::Context;


/// CoinMarketCap price provider.
///
/// This struct handles fetching cryptocurrency prices from the CoinMarketCap API.
/// It requires a valid API key to be set in the `COINMARKETCAP_API_KEY` environment variable.
///
/// # Examples
///
/// ```rust
/// use boltzmann::crypto::coinmarketcap::CoinMarketCap;
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
    /// Returns an error if the API key is not set or invalid,
    /// or if the HTTP client cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boltzmann::crypto::coinmarketcap::CoinMarketCap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = CoinMarketCap::new("api_key".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(api_key: String) -> Result<Self> {
        if api_key.is_empty() {
            anyhow::bail!("CoinMarketCap API key cannot be empty");
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-CMC_PRO_API_KEY",
            reqwest::header::HeaderValue::from_str(&api_key)
                .context("Invalid API key format")?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .crypto_context("creating HTTP client for CoinMarketCap")?;

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
    /// Returns an error if the request fails or the response cannot be parsed.
    async fn fetch_quotes(
        &self,
        coin: Coin,
        currencies: &[Currency],
    ) -> Result<Vec<Quote>> {
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
            .crypto_context("sending request to CoinMarketCap API")?;

        let body = response
            .text()
            .await
            .crypto_context("reading response body from CoinMarketCap API")?;

        let json: Value = serde_json::from_str(&body)
            .context("parsing JSON response from CoinMarketCap API")?;

        let mut quotes = Vec::new();
        let timestamp = chrono::Utc::now();

        let coin_id_str = coin_id.to_string();
        let coin_data = &json["data"][&coin_id_str];

        if coin_data.is_null() {
            anyhow::bail!("No data found for coin {} in CoinMarketCap response", coin);
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
                anyhow::bail!("Price not found for {} in {} from CoinMarketCap", coin, currency);
            }
        }

        Ok(quotes)
    }
}

#[async_trait]
impl PriceProvider for CoinMarketCap {
    type Error = anyhow::Error;

    async fn get_quotes(
        &self,
        coin: Coin,
        currencies: &[Currency],
    ) -> std::result::Result<Vec<Quote>, Self::Error> {
        self.fetch_quotes(coin, currencies).await
    }
}
