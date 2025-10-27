//! Cryptocurrency price quote endpoints.
//!
//! This module handles requests for cryptocurrency prices from multiple providers.
//! Supports configurable amounts and currencies with fallback between providers.

use axum::{extract::{Query, State}, response::IntoResponse, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::Value;
use utoipa::IntoParams;
use tracing::{info, warn, error};

use crate::core::config::AppState;
use crate::domains::crypto::{Coin, Currency, PriceProvider, Quote};
use crate::domains::crypto::coingecko::CoinGecko;
use crate::domains::crypto::coinmarketcap::CoinMarketCap;

/// Query parameters for price quote requests.
#[derive(Deserialize, IntoParams)]
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
#[utoipa::path(
    get,
    path = "/api/v1/price/prices",
    tag = "crypto",
    params(QuoteQueryParams),
    responses(
        (status = 200, description = "Successful response with price quotes", body = Vec<Quote>),
        (status = 500, description = "No quotes available from any provider")
    )
)]
pub async fn get_crypto_prices(
    State(app_state): State<AppState>,
    Query(params): Query<QuoteQueryParams>,
) -> impl IntoResponse {
    info!("💰 Fetching cryptocurrency prices for {} {} in {}", params.amount, Coin::ETH, params.currency);

    let mut quotes = Vec::new();

    // Try CoinMarketCap
    if let Some(api_key) = &app_state.config.coinmarketcap_api_key {
        match CoinMarketCap::new(api_key.clone()) {
        Ok(cmc_provider) => match cmc_provider.get_quotes(Coin::ETH, &[params.currency]).await {
            Ok(cmc_quotes) => {
                if let Some(quote) = cmc_quotes.first() {
                    quotes.push(quote.with_amount(params.amount as f64));
                    info!("📊 CoinMarketCap: {} {} = {}{:.2} at {}", 
                        quote.coin, 1, quote.currency.symbol(), quote.price, quote.timestamp);
                }
            }
            Err(e) => {
                warn!("CoinMarketCap API failed: {}", e);
            }
        },
            Err(e) => {
                error!("CoinMarketCap initialization failed: {}", e);
            }
        }
    } else {
        info!("CoinMarketCap API key not configured, skipping provider");
    }


    // Try CoinGecko
    match CoinGecko::new(app_state.config.coingecko_api_key.clone()) {
        Ok(cg_provider) => match cg_provider.get_quotes(Coin::ETH, &[params.currency]).await {
            Ok(cg_quotes) => {
                if let Some(quote) = cg_quotes.first() {
                    quotes.push(quote.with_amount(params.amount as f64));
                    info!("🦎 CoinGecko: {} {} = {}{:.2} at {}", 
                        quote.coin, 1, quote.currency.symbol(), quote.price, quote.timestamp);
                }
            }
            Err(e) => {
                warn!("CoinGecko API failed: {}", e);
            }
        },
        Err(e) => {
            error!("CoinGecko initialization failed: {}", e);
        }
    }

    info!("Price fetching completed. Retrieved {} quotes", quotes.len());

    let result: (StatusCode, Json<Value>) = if quotes.is_empty() {
        let error_json = serde_json::json!({"error": "No quotes available from any provider"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
    } else {
        (StatusCode::OK, Json(serde_json::to_value(quotes).unwrap_or_default()))
    };

    result
}