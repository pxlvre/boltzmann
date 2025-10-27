//! Cryptocurrency price quote endpoints.
//!
//! This module handles requests for cryptocurrency prices from multiple providers.
//! Supports configurable amounts and currencies with fallback between providers.

use axum::{extract::{Query, State}, response::IntoResponse, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::Value;

use crate::core::config::AppState;
use crate::domains::crypto::{Coin, Currency, PriceProvider};
use crate::domains::crypto::coingecko::CoinGecko;
use crate::domains::crypto::coinmarketcap::CoinMarketCap;

/// Query parameters for price quote requests.
#[derive(Deserialize)]
pub struct QuoteQueryParams {
    /// Number of crypto to get price for (defaults to 1)
    #[serde(default = "default_amount")]
    pub amount: usize,
    /// Currency to get price in (defaults to USD)
    #[serde(default = "default_currency")]
    pub currency: Currency,
}

fn default_amount() -> usize {
    1
}

fn default_currency() -> Currency {
    Currency::USD
}

/// Get cryptocurrency price quotes from available providers.
///
/// This endpoint fetches ETH prices from configured providers (CoinMarketCap, CoinGecko)
/// and returns quotes adjusted for the requested amount and currency.
///
/// # Query Parameters
///
/// * `amount` - Number of crypto (default: 1)
/// * `currency` - Target currency (default: USD)
///
/// # Examples
///
/// * `/api/v1/price/prices` - Get 1 ETH price in USD
/// * `/api/v1/price/prices?amount=5&currency=EUR` - Get 5 ETH price in EUR
///
/// # Returns
///
/// JSON array of quote objects or error if no providers are available.
pub async fn get_crypto_prices(
    State(app_state): State<AppState>,
    Query(params): Query<QuoteQueryParams>,
) -> impl IntoResponse {
    println!("🚀 Boltzmann Price Fetcher");
    println!("Fetching ETH price from multiple providers...\n");

    let mut quotes = Vec::new();

    // Try CoinMarketCap
    if let Some(api_key) = &app_state.config.coinmarketcap_api_key {
        match CoinMarketCap::new(api_key.clone()) {
        Ok(cmc_provider) => match cmc_provider.get_quotes(Coin::ETH, &[params.currency]).await {
            Ok(cmc_quotes) => {
                if let Some(quote) = cmc_quotes.first() {
                    quotes.push(quote.with_amount(params.amount as f64));
                    println!(
                        "📊 CoinMarketCap: 1 {} = {}{:.2}",
                        quote.coin,
                        quote.currency.symbol(),
                        quote.price
                    );
                    println!(
                        "   Timestamp: {}",
                        quote.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ CoinMarketCap failed: {}", e);
            }
        },
            Err(e) => {
                eprintln!("❌ CoinMarketCap initialization failed: {}", e);
            }
        }
    } else {
        eprintln!("❌ CoinMarketCap API key not configured");
    }

    println!();

    // Try CoinGecko
    match CoinGecko::new(app_state.config.coingecko_api_key.clone()) {
        Ok(cg_provider) => match cg_provider.get_quotes(Coin::ETH, &[params.currency]).await {
            Ok(cg_quotes) => {
                if let Some(quote) = cg_quotes.first() {
                    quotes.push(quote.with_amount(params.amount as f64));
                    println!(
                        "🦎 CoinGecko: 1 {} = {}{:.2}",
                        quote.coin,
                        quote.currency.symbol(),
                        quote.price
                    );
                    println!(
                        "   Timestamp: {}",
                        quote.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ CoinGecko failed: {}", e);
            }
        },
        Err(e) => {
            eprintln!("❌ CoinGecko initialization failed: {}", e);
        }
    }

    println!("\n✅ Price fetching complete!");

    let result: (StatusCode, Json<Value>) = if quotes.is_empty() {
        let error_json = serde_json::json!({"error": "No quotes available from any provider"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
    } else {
        (StatusCode::OK, Json(serde_json::to_value(quotes).unwrap_or_default()))
    };

    result
}