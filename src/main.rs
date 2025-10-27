// Boltzmann API server

use axum::{Router, routing::get, Json, response::IntoResponse, http::StatusCode};
use axum::extract::Query;
use std::net::SocketAddr;
use alloy_primitives::uint;
use async_trait::async_trait;
use axum::serve::Serve;
use serde::Deserialize;
use serde_json::Value;

mod coins;
use crate::coins::Quote;
use coins::coingecko::CoinGecko;
use coins::coinmarketcap::CoinMarketCap;
use coins::{Coin, Currency, PriceProvider};
use crate::coins::Currency::USD;

#[derive(Deserialize)]
struct QueryParams {
    #[serde(default = "default_amount")]
    amount: usize,
    #[serde(default = "default_currency")]
    currency: Currency,
}

fn default_amount() -> usize {
    1
}

fn default_currency() -> Currency {
    USD
}

async fn get_quotes(Query(params): Query<QueryParams>) -> impl IntoResponse {
    println!("🚀 Boltzmann Price Fetcher");
    println!("Fetching ETH price from multiple providers...\n");

    let mut quotes = Vec::new();

    // Try CoinMarketCap
    match CoinMarketCap::new() {
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

    println!();

    // Try CoinGecko
    match CoinGecko::new() {
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

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/quotes", get(get_quotes));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
