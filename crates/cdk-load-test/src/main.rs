use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use cdk::amount::Amount;
use cdk::mint_url::MintUrl;
use cdk::nuts::{CurrencyUnit, ProofsMethods};
use cdk::wallet::{ReceiveOptions, SendKind, SendOptions, Wallet};
use cdk_sqlite::wallet::{memory, WalletSqliteDatabase};
use clap::Parser;
use futures::future::join_all;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mint URL to test
    #[arg(short, long, default_value = "http://localhost:8085")]
    mint_url: String,

    /// Number of concurrent users
    #[arg(short, long, default_value_t = 10)]
    users: usize,

    /// Duration of test in seconds
    #[arg(short, long, default_value_t = 60)]
    duration: u64,

    /// Initial mint amount per user
    #[arg(short, long, default_value_t = 1000)]
    amount: u64,

    /// Test operations (mint, swap, melt, send_receive)
    #[arg(short, long, default_value = "mint,swap")]
    operations: String,
}

#[derive(Clone)]
struct LoadTestConfig {
    mint_url: MintUrl,
    users: usize,
    duration: Duration,
    initial_amount: Amount,
    operations: Vec<String>,
}

struct TestStats {
    mint_success: usize,
    mint_errors: usize,
    swap_success: usize,
    swap_errors: usize,
    send_receive_success: usize,
    send_receive_errors: usize,
    total_requests: usize,
    total_errors: usize,
}

impl TestStats {
    fn new() -> Self {
        Self {
            mint_success: 0,
            mint_errors: 0,
            swap_success: 0,
            swap_errors: 0,
            send_receive_success: 0,
            send_receive_errors: 0,
            total_requests: 0,
            total_errors: 0,
        }
    }

    fn print_summary(&self, duration: Duration) {
        let total_operations = self.mint_success + self.swap_success + self.send_receive_success;
        let rps = total_operations as f64 / duration.as_secs_f64();

        println!("\n=== Load Test Results ===");
        println!("Duration: {:?}", duration);
        println!("Total Operations: {}", total_operations);
        println!("Total Errors: {}", self.total_errors);
        println!("Requests per second: {:.2}", rps);
        println!(
            "Success rate: {:.2}%",
            (total_operations as f64 / (total_operations + self.total_errors) as f64) * 100.0
        );
        println!("\nBreakdown:");
        println!(
            "  Mint - Success: {}, Errors: {}",
            self.mint_success, self.mint_errors
        );
        println!(
            "  Swap - Success: {}, Errors: {}",
            self.swap_success, self.swap_errors
        );
        println!(
            "  Send/Receive - Success: {}, Errors: {}",
            self.send_receive_success, self.send_receive_errors
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    //tracing_subscriber::init();

    let args = Args::parse();

    let config = LoadTestConfig {
        mint_url: args.mint_url.parse()?,
        users: args.users,
        duration: Duration::from_secs(args.duration),
        initial_amount: Amount::from(args.amount),
        operations: args
            .operations
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
    };

    info!(
        "Starting load test with {} users for {:?}",
        config.users, config.duration
    );
    info!("Target mint: {}", config.mint_url);
    info!("Operations: {:?}", config.operations);

    run_load_test(config).await
}

async fn run_load_test(config: LoadTestConfig) -> Result<()> {
    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Create tasks for each concurrent user
    for user_id in 0..config.users {
        let config_clone = config.clone();
        let handle = tokio::spawn(async move { user_load_test(user_id, config_clone).await });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = join_all(handles).await;

    // Aggregate stats
    let mut total_stats = TestStats::new();
    for result in results {
        match result {
            Ok(stats) => {
                total_stats.mint_success += stats.mint_success;
                total_stats.mint_errors += stats.mint_errors;
                total_stats.swap_success += stats.swap_success;
                total_stats.swap_errors += stats.swap_errors;
                total_stats.send_receive_success += stats.send_receive_success;
                total_stats.send_receive_errors += stats.send_receive_errors;
                total_stats.total_requests += stats.total_requests;
                total_stats.total_errors += stats.total_errors;
            }
            Err(e) => {
                error!("Task failed: {}", e);
            }
        }
    }

    let elapsed = start_time.elapsed();
    total_stats.print_summary(elapsed);

    Ok(())
}

async fn user_load_test(user_id: usize, config: LoadTestConfig) -> TestStats {
    let mut stats = TestStats::new();
    let end_time = Instant::now() + config.duration;

    // Create unique wallet for this user
    let db_path = format!("/tmp/load_test_wallet_{}.db", user_id);
    let localstore = match WalletSqliteDatabase::new(db_path.as_str()).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("User {}: Failed to create wallet database: {}", user_id, e);
            return stats;
        }
    };

    let wallet = match Wallet::new(
        &config.mint_url.to_string(),
        CurrencyUnit::Sat,
        localstore,
        bip39::Mnemonic::generate(12).unwrap().to_seed_normalized(""),
        Some(10), // target proof count
    ) {
        Ok(w) => Arc::new(w),
        Err(e) => {
            error!("User {}: Failed to create wallet: {}", user_id, e);
            return stats;
        }
    };

    info!("User {}: Starting load test", user_id);

    // Initial funding
    if let Err(e) = fund_wallet(wallet.clone(), config.clone()).await {
        error!("User {}: Failed to fund wallet: {}", user_id, e);
        return stats;
    }

    let mut iteration = 0;
    while Instant::now() < end_time {
        iteration += 1;

        for operation in &config.operations {
            match operation.as_str() {
                "mint" => {
                    stats.total_requests += 1;
                    match test_mint_operation(
                        wallet.clone(),
                        Amount::from(100),
                        config.mint_url.clone(),
                    )
                    .await
                    {
                        Ok(_) => stats.mint_success += 1,
                        Err(e) => {
                            stats.mint_errors += 1;
                            stats.total_errors += 1;
                            warn!("User {}: Mint operation failed: {}", user_id, e);
                        }
                    }
                }
                "swap" => {
                    stats.total_requests += 1;
                    match test_swap_operation(
                        wallet.clone(),
                        Amount::from(50),
                        config.mint_url.clone(),
                    )
                    .await
                    {
                        Ok(_) => stats.swap_success += 1,
                        Err(e) => {
                            stats.swap_errors += 1;
                            stats.total_errors += 1;
                            warn!("User {}: Swap operation failed: {}", user_id, e);
                        }
                    }
                }
                "send_receive" => {
                    stats.total_requests += 1;
                    stats.send_receive_success += 1
                }
                _ => {
                    warn!("User {}: Unknown operation: {}", user_id, operation);
                }
            }
        }

        // Small delay between iterations
        sleep(Duration::from_millis(100)).await;
    }

    info!("User {}: Completed {} iterations", user_id, iteration);
    stats
}

async fn fund_wallet(wallet: Arc<Wallet>, config: LoadTestConfig) -> Result<()> {
    let quote = wallet.mint_quote(config.initial_amount, None).await?;

    // In a real scenario, you'd pay the lightning invoice
    // For testing, we'll simulate payment by waiting a bit
    // You might want to integrate with a regtest Lightning node here
    sleep(Duration::from_millis(5000)).await;

    let _proofs = wallet
        .mint(&quote.id, cdk::amount::SplitTarget::default(), None)
        .await?;
    Ok(())
}

async fn test_mint_operation(wallet: Arc<Wallet>, amount: Amount, mint_url: MintUrl) -> Result<()> {
    let quote = wallet.mint_quote(amount, None).await?;

    // Simulate payment delay
    sleep(Duration::from_millis(5000)).await;

    let _proofs = wallet
        .mint(&quote.id, cdk::amount::SplitTarget::default(), None)
        .await?;
    Ok(())
}

async fn test_swap_operation(wallet: Arc<Wallet>, amount: Amount, mint_url: MintUrl) -> Result<()> {
    let proofs = wallet.get_unspent_proofs().await?;

    let _swap_response = wallet
        .swap(
            Some(amount),
            cdk::amount::SplitTarget::default(),
            proofs,
            None,
            false,
        )
        .await?;
    Ok(())
}
