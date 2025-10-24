// Boltzmann API server

mod coins;
use coins::coinmarketcap::get_quote;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_quote().await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}