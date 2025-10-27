//! Alloy-rs built-in gas price provider implementation.
//!
//! This module implements gas price fetching using alloy-rs built-in functions
//! to connect directly to Ethereum nodes.

use super::{GasOracle, GasPrice};
use crate::core::errors::{Result, ErrorContext};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types::FeeHistory;
use async_trait::async_trait;
use anyhow::Context;


/// Alloy gas price provider using direct Ethereum node connection
pub struct AlloyGasOracle {
    rpc_url: String,
}

impl AlloyGasOracle {
    /// Creates a new Alloy gas oracle instance with the provided RPC URL.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - The Ethereum RPC URL to connect to
    ///
    /// # Errors
    ///
    /// Returns `AlloyError::MissingRpcUrl` if the RPC URL is empty.
    pub fn new(rpc_url: String) -> Result<Self> {
        if rpc_url.is_empty() {
            anyhow::bail!("Ethereum RPC URL cannot be empty");
        }

        Ok(Self { rpc_url })
    }


    /// Calculates gas price percentiles from fee history
    fn calculate_gas_prices(&self, fee_history: &FeeHistory) -> Result<(f64, f64, f64)> {
        println!("💰 Calculating gas prices from fee history...");
        
        if fee_history.base_fee_per_gas.is_empty() {
            anyhow::bail!("No base fee data available in fee history");
        }

        // Get the latest base fee
        let latest_base_fee = fee_history.base_fee_per_gas
            .last()
            .context("No base fee available in fee history")?;

        println!("⛽ Latest base fee: {} wei", latest_base_fee);

        // Convert base fee from wei to gwei (preserve precision)
        let base_fee_gwei = *latest_base_fee as f64 / 1_000_000_000.0;
        println!("⛽ Base fee in Gwei: {:.6}", base_fee_gwei);

        // Calculate priority fees based on historical data
        let mut priority_fees = Vec::new();
        
        if let Some(reward_percentiles) = &fee_history.reward {
            println!("💎 Processing {} reward entries", reward_percentiles.len());
            for (i, rewards) in reward_percentiles.iter().enumerate() {
                if let Some(reward) = rewards.first() {
                    let priority_fee_gwei = *reward as f64 / 1_000_000_000.0;
                    priority_fees.push(priority_fee_gwei);
                    println!("   Block {}: {:.6} Gwei priority fee", i, priority_fee_gwei);
                }
            }
        } else {
            println!("⚠️  No reward percentiles data available");
        }

        // If we don't have enough data, use conservative estimates
        let (low_priority, avg_priority, high_priority) = if priority_fees.is_empty() {
            println!("📋 Using conservative priority fee estimates");
            (1.0f64, 2.0f64, 3.0f64) // Conservative priority fee estimates in gwei
        } else {
            priority_fees.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let len = priority_fees.len();
            let low = priority_fees[len / 4].max(1.0); // 25th percentile, minimum 1 gwei
            let avg = priority_fees[len / 2].max(2.0); // 50th percentile, minimum 2 gwei  
            let high = priority_fees[len * 3 / 4].max(3.0); // 75th percentile, minimum 3 gwei
            println!("📊 Calculated priority fees from {} samples: low={:.6}, avg={:.6}, high={:.6}", len, low, avg, high);
            (low, avg, high)
        };

        // Total gas price = base fee + priority fee
        let low_gas = base_fee_gwei + low_priority;
        let avg_gas = base_fee_gwei + avg_priority;
        let high_gas = base_fee_gwei + high_priority;

        println!("✅ Final gas prices: low={:.6} ({:.6}+{:.6}), avg={:.6} ({:.6}+{:.6}), high={:.6} ({:.6}+{:.6})", 
            low_gas, base_fee_gwei, low_priority,
            avg_gas, base_fee_gwei, avg_priority, 
            high_gas, base_fee_gwei, high_priority
        );

        Ok((low_gas, avg_gas, high_gas))
    }
}

#[async_trait]
impl GasOracle for AlloyGasOracle {
    type Error = anyhow::Error;

    async fn get_gas_prices(&self) -> std::result::Result<GasPrice, Self::Error> {
        // Parse RPC URL
        println!("🔗 Alloy RPC URL: {}", self.rpc_url);
        let url = self.rpc_url.parse()
            .with_context(|| format!("Invalid RPC URL: {}", self.rpc_url))?;

        // Create provider
        println!("🔌 Creating Alloy provider...");
        let provider = ProviderBuilder::new().connect_http(url);

        // Get fee history for the last 20 blocks with 25th, 50th, and 75th percentiles
        println!("📊 Fetching fee history from last 20 blocks...");
        let fee_history = provider
            .get_fee_history(20, alloy_rpc_types::BlockNumberOrTag::Latest, &[25.0, 50.0, 75.0])
            .await
            .gas_context("fetching fee history from Ethereum node")?;

        println!("📈 Fee history received: {} base fees, {} reward entries", 
            fee_history.base_fee_per_gas.len(),
            fee_history.reward.as_ref().map_or(0, |r| r.len())
        );

        let (low, average, high) = self.calculate_gas_prices(&fee_history)?;

        Ok(GasPrice {
            low,
            average,
            high,
            timestamp: chrono::Utc::now(),
        })
    }
}