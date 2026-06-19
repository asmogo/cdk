#![allow(missing_docs)]
//! Environment variables module
//!
//! This module contains all environment variable definitions and parsing logic
//! organized by component.

mod common;
mod database;
mod info;
mod limits;
mod mint_info;
mod payment;

mod auth;
#[cfg(feature = "bdk")]
mod bdk;
#[cfg(feature = "cln")]
mod cln;
#[cfg(feature = "fakewallet")]
mod fake_wallet;
#[cfg(feature = "grpc-processor")]
mod grpc_processor;
#[cfg(feature = "ldk-node")]
mod ldk_node;
#[cfg(feature = "lnbits")]
mod lnbits;
#[cfg(feature = "lnd")]
mod lnd;
#[cfg(feature = "management-rpc")]
mod management_rpc;
#[cfg(feature = "prometheus")]
mod prometheus;

use std::env;
use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
pub use auth::*;
#[cfg(feature = "bdk")]
pub use bdk::*;
#[cfg(feature = "cln")]
pub use cln::*;
pub use common::*;
pub use database::*;
#[cfg(feature = "fakewallet")]
pub use fake_wallet::*;
#[cfg(feature = "grpc-processor")]
pub use grpc_processor::*;
#[cfg(feature = "ldk-node")]
pub use ldk_node::*;
pub use limits::*;
#[cfg(feature = "lnbits")]
pub use lnbits::*;
#[cfg(feature = "lnd")]
pub use lnd::*;
#[cfg(feature = "management-rpc")]
pub use management_rpc::*;
pub use mint_info::*;
pub use payment::*;
#[cfg(feature = "prometheus")]
pub use prometheus::*;

use crate::config::{DatabaseEngine, PaymentBackend, PaymentBackendKind, Settings};

impl Settings {
    pub fn from_env(&mut self) -> Result<Self> {
        if let Ok(database) = env::var(DATABASE_ENV_VAR) {
            let engine = DatabaseEngine::from_str(&database).map_err(|err| anyhow!(err))?;
            self.database.engine = engine;
        }

        // Parse PostgreSQL-specific configuration from environment variables
        if self.database.engine == DatabaseEngine::Postgres {
            self.database.postgres = Some(
                self.database
                    .postgres
                    .clone()
                    .unwrap_or_default()
                    .from_env(),
            );
        }

        // Parse auth database configuration from environment variables
        self.auth_database = Some(crate::config::AuthDatabase {
            postgres: Some(
                self.auth_database
                    .clone()
                    .unwrap_or_default()
                    .postgres
                    .unwrap_or_default()
                    .from_env(),
            ),
        });

        self.info = self.info.clone().from_env();
        self.mint_info = self.mint_info.clone().from_env();
        if payment_env_is_set() {
            // Common payment env vars only apply when there is exactly one entry.
            // Multi-backend setups must choose units and backends in the config file.
            match self.payment_backends.len() {
                0 => {
                    let backend = PaymentBackend::default().from_env();
                    if backend.backend != PaymentBackendKind::None {
                        self.payment_backends.push(backend);
                    }
                }
                1 => {
                    self.payment_backends[0] = self.payment_backends[0].clone().from_env();
                }
                _ => {
                    tracing::warn!(
                        "CDK_MINTD_PAYMENT_* environment variables ignored: multiple [[payment_backend]] entries configured"
                    );
                }
            }
        } else {
            let has_legacy_ln_env = legacy_ln_env_is_set();
            if has_legacy_ln_env {
                let index = self
                    .payment_backends
                    .iter()
                    .position(|entry| !is_bdk_backend(entry));
                match index {
                    Some(index) => {
                        self.payment_backends[index] =
                            self.payment_backends[index].clone().apply_legacy_ln_env();
                    }
                    None => {
                        let backend = PaymentBackend::default().apply_legacy_ln_env();
                        if backend.backend != PaymentBackendKind::None {
                            self.payment_backends.push(backend);
                        }
                    }
                }
            }

            if legacy_onchain_env_is_set() {
                let index = self
                    .payment_backends
                    .iter()
                    .position(is_bdk_backend)
                    .or_else(|| {
                        (!has_legacy_ln_env && self.payment_backends.len() == 1).then_some(0)
                    })
                    .or_else(|| {
                        (self.payment_backends.len() > 1).then(|| self.payment_backends.len() - 1)
                    });
                match index {
                    Some(index) => {
                        self.payment_backends[index] = self.payment_backends[index]
                            .clone()
                            .apply_legacy_onchain_env();
                    }
                    None => {
                        let backend = PaymentBackend::default().apply_legacy_onchain_env();
                        if backend.backend != PaymentBackendKind::None
                            && !is_duplicate_fake_wallet(&self.payment_backends, &backend)
                        {
                            self.payment_backends.push(backend);
                        }
                    }
                }
            }
        }
        self.limits = self.limits.clone().from_env();

        {
            // Check env vars for auth config even if None
            let auth = self.auth.clone().unwrap_or_default().from_env();

            // Only set auth if auth_enabled flag is true
            if auth.auth_enabled {
                self.auth = Some(auth);
            } else {
                self.auth = None;
            }
        }

        #[cfg(feature = "management-rpc")]
        {
            self.mint_management_rpc = Some(
                self.mint_management_rpc
                    .clone()
                    .unwrap_or_default()
                    .from_env(),
            );
        }

        #[cfg(feature = "prometheus")]
        {
            self.prometheus = Some(self.prometheus.clone().unwrap_or_default().from_env());
        }

        #[cfg(feature = "cln")]
        {
            let cln = self.cln.clone().unwrap_or_default().from_env();
            if cln.rpc_path.as_os_str().is_empty() {
                self.cln = None;
            } else {
                self.cln = Some(cln);
            }
        }

        #[cfg(feature = "lnbits")]
        {
            let lnbits = self.lnbits.clone().unwrap_or_default().from_env();
            if lnbits.admin_api_key.is_empty() {
                self.lnbits = None;
            } else {
                self.lnbits = Some(lnbits);
            }
        }

        #[cfg(feature = "fakewallet")]
        {
            // Fake wallet has defaults so it is always Some if feature enabled
            let fake_wallet_supported_units_from_env =
                env::var(ENV_FAKE_WALLET_SUPPORTED_UNITS).is_ok();
            let fake_wallet = self.fake_wallet.clone().unwrap_or_default().from_env();
            let supported_units_configured =
                fake_wallet.supported_units != vec![cdk::nuts::CurrencyUnit::Sat];

            if fake_wallet_supported_units_from_env || supported_units_configured {
                self.expand_single_fake_wallet_entry(&fake_wallet);
            }

            self.fake_wallet = Some(fake_wallet);
        }

        #[cfg(feature = "lnd")]
        {
            let lnd = self.lnd.clone().unwrap_or_default().from_env();
            if lnd.address.is_empty() {
                self.lnd = None;
            } else {
                self.lnd = Some(lnd);
            }
        }

        #[cfg(feature = "ldk-node")]
        {
            let ldk_node = self.ldk_node.clone().unwrap_or_default().from_env();
            if ldk_node.bitcoin_network.is_none() && ldk_node.esplora_url.is_none() {
                self.ldk_node = None;
            } else {
                self.ldk_node = Some(ldk_node);
            }
        }

        #[cfg(feature = "grpc-processor")]
        {
            let grpc_processor = self.grpc_processor.clone().unwrap_or_default().from_env();
            let grpc_processor_configured = self
                .payment_backends
                .iter()
                .any(|entry| entry.backend == PaymentBackendKind::GrpcProcessor);
            if grpc_processor.supported_units.is_empty() && !grpc_processor_configured {
                self.grpc_processor = None;
            } else {
                self.grpc_processor = Some(grpc_processor);
            }
        }

        #[cfg(feature = "bdk")]
        {
            let bdk = self.bdk.clone().unwrap_or_default().from_env();
            if bdk.network.is_none() && bdk.mnemonic.is_none() {
                self.bdk = None;
            } else {
                self.bdk = Some(bdk);
            }
        }

        for entry in &self.payment_backends {
            match entry.backend {
                #[cfg(feature = "cln")]
                PaymentBackendKind::Cln => {}
                #[cfg(feature = "lnbits")]
                PaymentBackendKind::LNbits => {}
                #[cfg(feature = "fakewallet")]
                PaymentBackendKind::FakeWallet => {}
                #[cfg(feature = "lnd")]
                PaymentBackendKind::Lnd => {}
                #[cfg(feature = "ldk-node")]
                PaymentBackendKind::LdkNode => {}
                #[cfg(feature = "grpc-processor")]
                PaymentBackendKind::GrpcProcessor => {}
                #[cfg(feature = "bdk")]
                PaymentBackendKind::Bdk => {}
                PaymentBackendKind::None => {}
                #[allow(unreachable_patterns)]
                _ => bail!("Selected payment backend is not enabled in this build"),
            }
        }

        let has_configured_backend = self
            .payment_backends
            .iter()
            .any(|entry| entry.backend != PaymentBackendKind::None);
        if !has_configured_backend {
            bail!("At least one payment backend must be set");
        }

        self.validate_backend_pairing()
            .map_err(|err| anyhow!(err))?;

        Ok(self.clone())
    }

    #[cfg(feature = "fakewallet")]
    fn expand_single_fake_wallet_entry(&mut self, fake_wallet: &crate::config::FakeWallet) {
        let fake_wallet_indices = self
            .payment_backends
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                (entry.backend == PaymentBackendKind::FakeWallet).then_some(index)
            })
            .collect::<Vec<_>>();

        if fake_wallet_indices.len() != 1 {
            return;
        }

        let mut units = Vec::new();
        for unit in &fake_wallet.supported_units {
            if !units.contains(unit) {
                units.push(unit.clone());
            }
        }

        if units.is_empty() {
            return;
        }

        let index = fake_wallet_indices[0];
        let base_backend = self.payment_backends[index].clone();
        let expanded_backends = units.into_iter().map(|unit| PaymentBackend {
            unit,
            ..base_backend.clone()
        });

        self.payment_backends
            .splice(index..=index, expanded_backends);
    }
}

fn is_bdk_backend(entry: &PaymentBackend) -> bool {
    #[cfg(feature = "bdk")]
    {
        entry.backend == PaymentBackendKind::Bdk
    }
    #[cfg(not(feature = "bdk"))]
    {
        let _ = entry;
        false
    }
}

fn is_duplicate_fake_wallet(configured: &[PaymentBackend], candidate: &PaymentBackend) -> bool {
    #[cfg(feature = "fakewallet")]
    {
        candidate.backend == PaymentBackendKind::FakeWallet
            && configured
                .iter()
                .any(|entry| entry.backend == PaymentBackendKind::FakeWallet)
    }
    #[cfg(not(feature = "fakewallet"))]
    {
        let _ = (configured, candidate);
        false
    }
}
