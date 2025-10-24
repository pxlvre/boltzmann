// Boltzmann API server

mod coins;
use coins::{Coin, Currency, PriceProvider};
use coins::coinmarketcap::CoinMarketCap;
use coins::coingecko::CoinGecko;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Boltzmann Price Fetcher");
    println!("Fetching ETH price from multiple providers...\n");

    // Try CoinMarketCap
    match CoinMarketCap::new() {
        Ok(cmc_provider) => {
            match cmc_provider.get_quotes(Coin::ETH, &[Currency::USD]).await {
                Ok(quotes) => {
                    if let Some(quote) = quotes.first() {
                        println!("📊 CoinMarketCap: 1 {} = {}{:.2}", 
                            quote.coin, 
                            quote.currency.symbol(), 
                            quote.price
                        );
                        println!("   Timestamp: {}", quote.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                }
                Err(e) => {
                    eprintln!("❌ CoinMarketCap failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ CoinMarketCap initialization failed: {}", e);
        }
    }

    println!();

    // Try CoinGecko
    match CoinGecko::new() {
        Ok(cg_provider) => {
            match cg_provider.get_quotes(Coin::ETH, &[Currency::USD]).await {
                Ok(quotes) => {
                    if let Some(quote) = quotes.first() {
                        println!("🦎 CoinGecko: 1 {} = {}{:.2}", 
                            quote.coin, 
                            quote.currency.symbol(), 
                            quote.price
                        );
                        println!("   Timestamp: {}", quote.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                }
                Err(e) => {
                    eprintln!("❌ CoinGecko failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ CoinGecko initialization failed: {}", e);
        }
    }

    println!("\n✅ Price fetching complete!");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}