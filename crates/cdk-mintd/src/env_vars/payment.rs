//! Payment backend common environment variables.

use std::env;
use std::sync::Once;

use crate::config::PaymentBackend;

pub const ENV_PAYMENT_BACKEND: &str = "CDK_MINTD_PAYMENT_BACKEND";
pub const ENV_PAYMENT_INVOICE_DESCRIPTION: &str = "CDK_MINTD_PAYMENT_INVOICE_DESCRIPTION";
pub const ENV_PAYMENT_MIN_MINT: &str = "CDK_MINTD_PAYMENT_MIN_MINT";
pub const ENV_PAYMENT_MAX_MINT: &str = "CDK_MINTD_PAYMENT_MAX_MINT";
pub const ENV_PAYMENT_MIN_MELT: &str = "CDK_MINTD_PAYMENT_MIN_MELT";
pub const ENV_PAYMENT_MAX_MELT: &str = "CDK_MINTD_PAYMENT_MAX_MELT";

// Deprecated aliases retained for existing deployments.
pub const ENV_LN_BACKEND: &str = "CDK_MINTD_LN_BACKEND";
pub const ENV_LN_INVOICE_DESCRIPTION: &str = "CDK_MINTD_LN_INVOICE_DESCRIPTION";
pub const ENV_LN_MIN_MINT: &str = "CDK_MINTD_LN_MIN_MINT";
pub const ENV_LN_MAX_MINT: &str = "CDK_MINTD_LN_MAX_MINT";
pub const ENV_LN_MIN_MELT: &str = "CDK_MINTD_LN_MIN_MELT";
pub const ENV_LN_MAX_MELT: &str = "CDK_MINTD_LN_MAX_MELT";
pub const ENV_ONCHAIN_BACKEND: &str = "CDK_MINTD_ONCHAIN_BACKEND";
pub const ENV_ONCHAIN_MIN_MINT: &str = "CDK_MINTD_ONCHAIN_MIN_MINT";
pub const ENV_ONCHAIN_MAX_MINT: &str = "CDK_MINTD_ONCHAIN_MAX_MINT";
pub const ENV_ONCHAIN_MIN_MELT: &str = "CDK_MINTD_ONCHAIN_MIN_MELT";
pub const ENV_ONCHAIN_MAX_MELT: &str = "CDK_MINTD_ONCHAIN_MAX_MELT";

static LEGACY_ENV_WARNING: Once = Once::new();

fn env_with_legacy(current: &str, legacy: &[&str]) -> Option<String> {
    if let Ok(value) = env::var(current) {
        return Some(value);
    }

    for legacy_name in legacy {
        if let Ok(value) = env::var(legacy_name) {
            warn_legacy_env();
            return Some(value);
        }
    }

    None
}

impl PaymentBackend {
    pub fn from_env(mut self) -> Self {
        if let Some(backend_str) =
            env_with_legacy(ENV_PAYMENT_BACKEND, &[ENV_LN_BACKEND, ENV_ONCHAIN_BACKEND])
        {
            match backend_str.parse() {
                Ok(backend) => self.backend = backend,
                Err(_) => tracing::warn!(
                    "Unknown payment backend set in env var; using config value: {backend_str}"
                ),
            }
        }

        if let Some(description) = env_with_legacy(
            ENV_PAYMENT_INVOICE_DESCRIPTION,
            &[ENV_LN_INVOICE_DESCRIPTION],
        ) {
            self.invoice_description = Some(description);
        }

        if let Some(amount) = parse_amount(
            ENV_PAYMENT_MIN_MINT,
            &[ENV_LN_MIN_MINT, ENV_ONCHAIN_MIN_MINT],
        ) {
            self.min_mint = amount.into();
        }
        if let Some(amount) = parse_amount(
            ENV_PAYMENT_MAX_MINT,
            &[ENV_LN_MAX_MINT, ENV_ONCHAIN_MAX_MINT],
        ) {
            self.max_mint = amount.into();
        }
        if let Some(amount) = parse_amount(
            ENV_PAYMENT_MIN_MELT,
            &[ENV_LN_MIN_MELT, ENV_ONCHAIN_MIN_MELT],
        ) {
            self.min_melt = amount.into();
        }
        if let Some(amount) = parse_amount(
            ENV_PAYMENT_MAX_MELT,
            &[ENV_LN_MAX_MELT, ENV_ONCHAIN_MAX_MELT],
        ) {
            self.max_melt = amount.into();
        }

        self
    }

    pub(crate) fn apply_legacy_ln_env(mut self) -> Self {
        warn_legacy_env();
        apply_backend(&mut self, ENV_LN_BACKEND);
        apply_description(&mut self, ENV_LN_INVOICE_DESCRIPTION);
        apply_amounts(
            &mut self,
            ENV_LN_MIN_MINT,
            ENV_LN_MAX_MINT,
            ENV_LN_MIN_MELT,
            ENV_LN_MAX_MELT,
        );
        self
    }

    pub(crate) fn apply_legacy_onchain_env(mut self) -> Self {
        warn_legacy_env();
        apply_backend(&mut self, ENV_ONCHAIN_BACKEND);
        apply_amounts(
            &mut self,
            ENV_ONCHAIN_MIN_MINT,
            ENV_ONCHAIN_MAX_MINT,
            ENV_ONCHAIN_MIN_MELT,
            ENV_ONCHAIN_MAX_MELT,
        );
        self
    }
}

fn parse_amount(current: &str, legacy: &[&str]) -> Option<u64> {
    env_with_legacy(current, legacy).and_then(|value| value.parse().ok())
}

pub(crate) fn payment_env_is_set() -> bool {
    [
        ENV_PAYMENT_BACKEND,
        ENV_PAYMENT_INVOICE_DESCRIPTION,
        ENV_PAYMENT_MIN_MINT,
        ENV_PAYMENT_MAX_MINT,
        ENV_PAYMENT_MIN_MELT,
        ENV_PAYMENT_MAX_MELT,
    ]
    .iter()
    .any(|name| env::var_os(name).is_some())
}

pub(crate) fn legacy_ln_env_is_set() -> bool {
    [
        ENV_LN_BACKEND,
        ENV_LN_INVOICE_DESCRIPTION,
        ENV_LN_MIN_MINT,
        ENV_LN_MAX_MINT,
        ENV_LN_MIN_MELT,
        ENV_LN_MAX_MELT,
    ]
    .iter()
    .any(|name| env::var_os(name).is_some())
}

pub(crate) fn legacy_onchain_env_is_set() -> bool {
    [
        ENV_ONCHAIN_BACKEND,
        ENV_ONCHAIN_MIN_MINT,
        ENV_ONCHAIN_MAX_MINT,
        ENV_ONCHAIN_MIN_MELT,
        ENV_ONCHAIN_MAX_MELT,
    ]
    .iter()
    .any(|name| env::var_os(name).is_some())
}

fn warn_legacy_env() {
    LEGACY_ENV_WARNING.call_once(|| {
        tracing::warn!(
            "CDK_MINTD_LN_* and CDK_MINTD_ONCHAIN_* are deprecated; use CDK_MINTD_PAYMENT_*"
        );
    });
}

fn apply_backend(backend: &mut PaymentBackend, name: &str) {
    if let Ok(value) = env::var(name) {
        match value.parse() {
            Ok(kind) => backend.backend = kind,
            Err(_) => tracing::warn!(
                "Unknown payment backend set in env var; using config value: {value}"
            ),
        }
    }
}

fn apply_description(backend: &mut PaymentBackend, name: &str) {
    if let Ok(value) = env::var(name) {
        backend.invoice_description = Some(value);
    }
}

fn apply_amounts(
    backend: &mut PaymentBackend,
    min_mint: &str,
    max_mint: &str,
    min_melt: &str,
    max_melt: &str,
) {
    if let Ok(amount) = env::var(min_mint).unwrap_or_default().parse::<u64>() {
        backend.min_mint = amount.into();
    }
    if let Ok(amount) = env::var(max_mint).unwrap_or_default().parse::<u64>() {
        backend.max_mint = amount.into();
    }
    if let Ok(amount) = env::var(min_melt).unwrap_or_default().parse::<u64>() {
        backend.min_melt = amount.into();
    }
    if let Ok(amount) = env::var(max_melt).unwrap_or_default().parse::<u64>() {
        backend.max_melt = amount.into();
    }
}
