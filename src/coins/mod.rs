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

use serde::{Deserialize, Serialize};
use std::fmt;
use async_trait::async_trait;

pub mod coinmarketcap;
pub mod coingecko;

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
}

/// Supported cryptocurrencies.
///
/// This enum represents the cryptocurrencies supported by the price providers.
/// Each coin has corresponding IDs for different API providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coin {
    /// Ethereum
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
        }
    }

    /// Returns all supported currencies.
    ///
    /// This is useful for fetching prices in all available currencies.
    pub fn all() -> &'static [Currency] {
        &[Currency::USD, Currency::EUR, Currency::CHF]
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

/// A cryptocurrency price quote at a specific point in time.
///
/// Contains the coin, currency, price, and timestamp for when the quote was fetched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// The cryptocurrency being quoted
    pub coin: Coin,
    /// The fiat currency the price is denominated in
    pub currency: Currency,
    /// The price of the cryptocurrency in the specified currency
    pub price: f64,
    /// When this quote was fetched
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
    async fn get_quotes(&self, coin: Coin, currencies: &[Currency]) -> Result<Vec<Quote>, Self::Error>;
}