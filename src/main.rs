// Boltzmann API server

use axum::{Router, routing::get, Json, response::IntoResponse, http::StatusCode};
use axum::extract::Query;
use serde::Deserialize;
use serde_json::Value;

mod coins;
mod gas;

use crate::gas::price::{GasOracle, GasQuote, GasOracleSource};
use crate::gas::price::etherscan::EtherscanGasOracle;
use crate::gas::price::alloy::AlloyGasOracle;
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

#[derive(Deserialize)]
struct GasPriceParams {
    #[serde(default = "default_gas_provider")]
    provider: GasOracleSource,
}

fn default_amount() -> usize {
    1
}

fn default_currency() -> Currency {
    USD
}

fn default_gas_provider() -> GasOracleSource {
    GasOracleSource::Etherscan
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

async fn get_gas_prices(Query(params): Query<GasPriceParams>) -> impl IntoResponse {
    println!("⛽ Boltzmann Gas Price Fetcher");
    println!("Fetching gas prices from {:?} provider...\n", params.provider);

    let gas_quote = match params.provider {
        GasOracleSource::Etherscan => {
            match EtherscanGasOracle::new() {
                Ok(oracle) => {
                    match oracle.get_gas_prices().await {
                        Ok(gas_price) => Some(GasQuote {
                            gas_price,
                            provider: GasOracleSource::Etherscan,
                        }),
                        Err(e) => {
                            eprintln!("❌ Etherscan gas oracle failed: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Etherscan gas oracle initialization failed: {}", e);
                    None
                }
            }
        }
        GasOracleSource::Alloy => {
            match AlloyGasOracle::new() {
                Ok(oracle) => {
                    match oracle.get_gas_prices().await {
                        Ok(gas_price) => Some(GasQuote {
                            gas_price,
                            provider: GasOracleSource::Alloy,
                        }),
                        Err(e) => {
                            eprintln!("❌ Alloy gas oracle failed: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Alloy gas oracle initialization failed: {}", e);
                    None
                }
            }
        }
    };

    println!("\n✅ Gas price fetching complete!");

    match gas_quote {
        Some(quote) => (StatusCode::OK, Json(serde_json::to_value(quote).unwrap_or_default())),
        None => {
            let error_json = serde_json::json!({"error": "Failed to fetch gas prices from provider"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/quotes", get(get_quotes))
        .route("/gas-price", get(get_gas_prices));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
