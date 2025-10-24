// Boltzmann API server
use dotenvy::dotenv;
use std::env;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
        dotenv().ok();

    // read API key from environment
    let api_key = env::var("API_KEY").expect("API_KEY not set in environment");

    // fallback URL (or read from env similarly)
    let url = env::var("API_URL").unwrap_or_else(|_| "https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest?id=1027".to_string());

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert("X-CMC_PRO_API_KEY", reqwest::header::HeaderValue::from_str(&api_key).unwrap());

    let req = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let res = req.get(url).send().await?;

    let body = res.text().await?;

    eprintln!("Response: {}", &body);

    let usd_quote = serde_json::from_str::<serde_json::Value>(&body)
        .unwrap()["data"]["1027"]["quote"]["USD"]["price"]
        .clone();

    println!("1 ETH is {usd_quote} dollars.");

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}