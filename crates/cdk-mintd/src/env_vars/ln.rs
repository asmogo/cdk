//! Lightning Network common environment variables

use std::env;

use crate::config::{LnBackendConfig, Database};

// LN environment variables
pub const ENV_LN_BACKEND: &str = "CDK_MINTD_LN_BACKEND";
pub const ENV_LN_INVOICE_DESCRIPTION: &str = "CDK_MINTD_LN_INVOICE_DESCRIPTION";
pub const ENV_LN_MIN_MINT: &str = "CDK_MINTD_LN_MIN_MINT";
pub const ENV_LN_MAX_MINT: &str = "CDK_MINTD_LN_MAX_MINT";
pub const ENV_LN_MIN_MELT: &str = "CDK_MINTD_LN_MIN_MELT";
pub const ENV_LN_MAX_MELT: &str = "CDK_MINTD_LN_MAX_MELT";

/// Tries to construct a `LnBackendConfig` from environment variables.
/// This allows for a single backend to be configured via env vars (backward compatibility).
pub fn parse_backend_from_env() -> Option<LnBackendConfig> {
    let backend_str = env::var(ENV_LN_BACKEND).ok()?;
    let backend_type = backend_str.to_lowercase();

    let currency_unit = cdk::nuts::CurrencyUnit::Sat;

    // Let's parse limits
    let min_mint = env::var(ENV_LN_MIN_MINT).ok().and_then(|s| s.parse().ok()).map(|v: u64| v.into()).unwrap_or(1.into());
    let max_mint = env::var(ENV_LN_MAX_MINT).ok().and_then(|s| s.parse().ok()).map(|v: u64| v.into()).unwrap_or(500_000.into());
    let min_melt = env::var(ENV_LN_MIN_MELT).ok().and_then(|s| s.parse().ok()).map(|v: u64| v.into()).unwrap_or(1.into());
    let max_melt = env::var(ENV_LN_MAX_MELT).ok().and_then(|s| s.parse().ok()).map(|v: u64| v.into()).unwrap_or(500_000.into());

    let database = Database::default();

    match backend_type.as_str() {
        #[cfg(feature = "cln")]
        "cln" => {
            let mut cln = crate::config::Cln::default();
            cln = cln.from_env();
            Some(LnBackendConfig::Cln {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                cln,
            })
        }
        #[cfg(feature = "lnbits")]
        "lnbits" => {
            let mut lnbits = crate::config::LNbits::default();
            lnbits = lnbits.from_env();
            Some(LnBackendConfig::Lnbits {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                lnbits,
            })
        }
        #[cfg(feature = "fakewallet")]
        "fakewallet" => {
            let mut fake_wallet = crate::config::FakeWallet::default();
            fake_wallet = fake_wallet.from_env();
            Some(LnBackendConfig::FakeWallet {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                fake_wallet,
            })
        }
        #[cfg(feature = "lnd")]
        "lnd" => {
            let mut lnd = crate::config::Lnd::default();
            lnd = lnd.from_env();
            Some(LnBackendConfig::Lnd {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                lnd,
            })
        }
        #[cfg(feature = "ldk-node")]
        "ldknode" | "ldk-node" => {
            let mut ldk_node = crate::config::LdkNode::default();
            ldk_node = ldk_node.from_env();
             Some(LnBackendConfig::LdkNode {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                ldk_node,
            })
        }
        #[cfg(feature = "grpc-processor")]
        "grpcprocessor" | "grpc-processor" => {
            let mut grpc_processor = crate::config::GrpcProcessor::default();
            grpc_processor = grpc_processor.from_env();
            Some(LnBackendConfig::GrpcProcessor {
                currency_unit,
                database,
                min_mint,
                max_mint,
                min_melt,
                max_melt,
                grpc_processor,
            })
        }
        _ => {
            tracing::warn!("Unknown or unsupported backend in env var: {}", backend_type);
            None
        }
    }
}
