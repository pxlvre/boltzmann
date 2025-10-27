//! Cryptocurrency price fetching module.
//!
//! This module provides a unified interface for fetching cryptocurrency prices
//! from multiple providers like CoinMarketCap and CoinGecko.
//!
//! # Examples
//!
//! ```rust
//! use boltzmann::coins::{Coin, Currency, PriceProvider};
//! use boltzmann::coins::coinmarketcap::CoinMarketCap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = CoinMarketCap::new()?;
//! let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD, Currency::EUR]).await?;
//!
//! for quote in quotes {
//!     println!("{} = {}{}", quote.coin, quote.currency.symbol(), quote.price);
//! }
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod coingecko;
pub mod coinmarketcap;

/// Supported fiat currencies for price conversion.
///
/// This enum represents the fiat currencies that can be used to fetch
/// cryptocurrency prices from supported providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// US Dollar
    USD,
    /// Euro
    EUR,
    /// Swiss Franc  
    CHF,
    /// Chinese Yuan
    CNY,
    /// British Pound Sterling
    GBP,
    /// Japanese Yen
    JPY,
    /// Canadian Dollar
    CAD,
    /// Australian Dollar
    AUD,
}

/// Supported cryptocurrencies.
///
/// This enum represents the cryptocurrencies supported by the price providers.
/// Each coin has corresponding IDs for different API providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coin {
    /// Ethereum
    ETH,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::CHF => write!(f, "CHF"),
            Currency::CNY => write!(f, "CNY"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
        }
    }
}

impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coin::ETH => write!(f, "ETH"),
        }
    }
}

impl Currency {
    /// Returns the currency symbol for display purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boltzmann::coins::Currency;
    ///
    /// assert_eq!(Currency::USD.symbol(), "$");
    /// assert_eq!(Currency::EUR.symbol(), "€");
    /// ```
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::CHF => "CHF",
            Currency::CNY => "¥",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
        }
    }

    /// Returns all supported currencies.
    ///
    /// This is useful for fetching prices in all available currencies.
    pub fn all() -> &'static [Currency] {
        &[
            Currency::USD, Currency::EUR, Currency::CHF,
            Currency::CNY, Currency::GBP, Currency::JPY,
            Currency::CAD, Currency::AUD
        ]
    }
}

impl Coin {
    /// Returns the CoinMarketCap API ID for this cryptocurrency.
    ///
    /// Used internally for CoinMarketCap API requests.
    pub fn coinmarketcap_id(&self) -> u32 {
        match self {
            Coin::ETH => 1027,
        }
    }

    /// Returns the CoinGecko API ID for this cryptocurrency.
    ///
    /// Used internally for CoinGecko API requests.
    pub fn coingecko_id(&self) -> &'static str {
        match self {
            Coin::ETH => "ethereum",
        }
    }

    /// Returns all supported cryptocurrencies.
    ///
    /// This is useful for fetching prices for all available coins.
    pub fn all() -> &'static [Coin] {
        &[Coin::ETH]
    }
}

/// Information about a specific amount and its total price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotePerAmount {
    /// The amount of cryptocurrency
    pub amount: f64,
    /// The total price for this amount
    pub total_price: f64,
}

/// Supported price provider sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderSource {
    #[serde(rename = "coinmarketcap")]
    CoinMarketCap,
    #[serde(rename = "coingecko")]
    CoinGecko,
}

/// A cryptocurrency price quote at a specific point in time.
///
/// Contains the coin, currency, unit price, and quote information for a specific amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// The cryptocurrency being quoted
    pub coin: Coin,
    /// The fiat currency the price is denominated in
    pub currency: Currency,
    /// The price per single unit of the cryptocurrency
    pub price: f64,
    /// The provider that supplied this quote
    pub provider: ProviderSource,
    /// When this quote was fetched
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Quote information for a specific amount
    pub quote_per_amount: QuotePerAmount,
}

impl Quote {
    /// Creates a new quote with quote information for the given amount.
    ///
    /// This is useful for calculating the total value of a specific amount
    /// of cryptocurrency at the current market price.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount of cryptocurrency to calculate the value for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boltzmann::coins::{Quote, QuotePerAmount, Coin, Currency};
    /// use chrono::Utc;
    ///
    /// let quote = Quote {
    ///     coin: Coin::ETH,
    ///     currency: Currency::USD,
    ///     price: 2000.0,
    ///     timestamp: Utc::now(),
    ///     quote_per_amount: QuotePerAmount { amount: 1.0, total_price: 2000.0 },
    /// };
    ///
    /// let total_value = quote.with_amount(2.5);
    /// assert_eq!(total_value.quote_per_amount.total_price, 5000.0);
    /// ```
    pub fn with_amount(&self, amount: f64) -> Self {
        Self {
            coin: self.coin,
            currency: self.currency,
            price: self.price,
            provider: self.provider.clone(),
            timestamp: self.timestamp,
            quote_per_amount: QuotePerAmount {
                amount,
                total_price: self.price * amount,
            },
        }
    }
}

/// Trait for cryptocurrency price providers.
///
/// This trait defines the interface that all price providers must implement.
/// It allows for fetching prices of a single cryptocurrency in multiple currencies
/// with a single API call for efficiency.
///
/// # Examples
///
/// ```rust
/// use boltzmann::coins::{Coin, Currency, PriceProvider};
/// use boltzmann::coins::coinmarketcap::CoinMarketCap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = CoinMarketCap::new()?;
///
/// // Fetch ETH price in USD and EUR
/// let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD, Currency::EUR]).await?;
///
/// for quote in quotes {
///     println!("{} = {}{:.2}", quote.coin, quote.currency.symbol(), quote.price);
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait PriceProvider {
    /// The error type returned by this provider
    type Error;

    /// Fetches prices for a single coin in multiple currencies.
    ///
    /// This method is optimized to make a single API call when fetching
    /// multiple currency prices for the same cryptocurrency.
    ///
    /// # Arguments
    ///
    /// * `coin` - The cryptocurrency to fetch prices for
    /// * `currencies` - The fiat currencies to get prices in
    ///
    /// # Returns
    ///
    /// A vector of `Quote` objects, one for each requested currency.
    /// The quotes are returned in the same order as the input currencies.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the API request fails, the response cannot be parsed,
    /// or the requested coin/currency combination is not supported.
    async fn get_quotes(
        &self,
        coin: Coin,
        currencies: &[Currency],
    ) -> Result<Vec<Quote>, Self::Error>;
}
