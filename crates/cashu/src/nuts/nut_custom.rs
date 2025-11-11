//! Custom Payment Methods
//!
//! This module defines generic request and response types for custom payment methods.
//! Unlike Bolt11/Bolt12, custom payment methods use opaque JSON data that is passed
//! directly to the payment processor without validation at the mint layer.
//!
//! ## Field Naming Conventions
//!
//! Custom payment methods should namespace their fields to avoid collisions with
//! standard fields. For example:
//! - PayPal fields: `paypal_transaction_id`, `paypal_payer_email`, etc.
//! - Venmo fields: `venmo_user_id`, `venmo_payment_url`, etc.
//!
//! The `data` field uses `#[serde(flatten)]` so custom fields appear at the top level
//! of the JSON object, making it easy to promote custom methods to official ones without
//! breaking client code.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CurrencyUnit, PublicKey};
use crate::nut05::QuoteState as MeltQuoteState;
use crate::nut23::QuoteState as MintQuoteState;
#[cfg(feature = "mint")]
use crate::quote_id::QuoteId;
use crate::Amount;

/// Custom payment method mint quote request
///
/// This is a generic request type that works for any custom payment method.
/// The `data` field is flattened, so method-specific fields appear at the top level.
///
/// ## Example JSON
/// ```json
/// {
///   "amount": 1000,
///   "unit": "sat",
///   "paypal_email": "user@example.com",
///   "paypal_return_url": "https://example.com/return"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct MintQuoteCustomRequest {
    /// Amount to mint
    pub amount: Amount,
    /// Currency unit
    pub unit: CurrencyUnit,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// NUT-19 Pubkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
    /// Method-specific data (flattened into top-level fields)
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Custom payment method mint quote response
///
/// This is a generic response type for custom payment methods.
/// The `data` field is flattened, so method-specific fields appear at the top level.
///
/// ## Example JSON
/// ```json
/// {
///   "quote": "abc123",
///   "request": "paypal://checkout/xyz",
///   "amount": 1000,
///   "unit": "sat",
///   "state": "UNPAID",
///   "expiry": 1234567890,
///   "paypal_transaction_id": "xyz789",
///   "paypal_redirect_url": "https://paypal.com/checkout/xyz"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[serde(bound = "Q: Serialize + for<'a> Deserialize<'a>")]
pub struct MintQuoteCustomResponse<Q> {
    /// Quote ID
    pub quote: Q,
    /// Payment request string (method-specific format)
    pub request: String,
    /// Amount
    pub amount: Option<Amount>,
    /// Currency unit
    pub unit: Option<CurrencyUnit>,
    /// Quote State
    pub state: MintQuoteState,
    /// Unix timestamp until the quote is valid
    pub expiry: Option<u64>,
    /// NUT-19 Pubkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
    /// Method-specific response data (flattened into top-level fields)
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

#[cfg(feature = "mint")]
impl<Q: ToString> MintQuoteCustomResponse<Q> {
    /// Convert the MintQuoteCustomResponse with a quote type Q to a String
    pub fn to_string_id(&self) -> MintQuoteCustomResponse<String> {
        MintQuoteCustomResponse {
            quote: self.quote.to_string(),
            request: self.request.clone(),
            amount: self.amount,
            state: self.state,
            unit: self.unit.clone(),
            expiry: self.expiry,
            pubkey: self.pubkey,
            data: self.data.clone(),
        }
    }
}

#[cfg(feature = "mint")]
impl From<MintQuoteCustomResponse<QuoteId>> for MintQuoteCustomResponse<String> {
    fn from(value: MintQuoteCustomResponse<QuoteId>) -> Self {
        Self {
            quote: value.quote.to_string(),
            request: value.request,
            amount: value.amount,
            unit: value.unit,
            expiry: value.expiry,
            state: value.state,
            pubkey: value.pubkey,
            data: value.data,
        }
    }
}

/// Custom payment method melt quote request
///
/// This is a generic request type for melting tokens with custom payment methods.
/// The `data` field is flattened, so method-specific fields appear at the top level.
///
/// ## Example JSON
/// ```json
/// {
///   "method": "paypal",
///   "request": "user@example.com",
///   "unit": "sat",
///   "paypal_memo": "Payment for services"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct MeltQuoteCustomRequest {
    /// Custom payment method name
    pub method: String,
    /// Payment request string (method-specific format)
    pub request: String,
    /// Currency unit
    pub unit: CurrencyUnit,
    /// Method-specific data (flattened into top-level fields)
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// Custom payment method melt quote response
///
/// This is a generic response type for custom payment methods.
/// The `data` field is flattened, so method-specific fields appear at the top level.
///
/// ## Example JSON
/// ```json
/// {
///   "quote": "def456",
///   "amount": 950,
///   "fee_reserve": 50,
///   "state": "PENDING",
///   "expiry": 1234567890,
///   "paypal_transaction_id": "abc123",
///   "paypal_status": "pending"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[serde(bound = "Q: Serialize + for<'a> Deserialize<'a>")]
pub struct MeltQuoteCustomResponse<Q> {
    /// Quote ID
    pub quote: Q,
    /// Amount
    pub amount: Amount,
    /// Fee reserve
    pub fee_reserve: Amount,
    /// Quote State
    pub state: MeltQuoteState,
    /// Unix timestamp until the quote is valid
    pub expiry: Option<u64>,
    /// Payment preimage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,
    /// Method-specific response data (flattened into top-level fields)
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

#[cfg(feature = "mint")]
impl<Q: ToString> MeltQuoteCustomResponse<Q> {
    /// Convert the MeltQuoteCustomResponse with a quote type Q to a String
    pub fn to_string_id(&self) -> MeltQuoteCustomResponse<String> {
        MeltQuoteCustomResponse {
            quote: self.quote.to_string(),
            amount: self.amount,
            fee_reserve: self.fee_reserve,
            state: self.state,
            expiry: self.expiry,
            payment_preimage: self.payment_preimage.clone(),
            data: self.data.clone(),
        }
    }
}

#[cfg(feature = "mint")]
impl From<MeltQuoteCustomResponse<QuoteId>> for MeltQuoteCustomResponse<String> {
    fn from(value: MeltQuoteCustomResponse<QuoteId>) -> Self {
        Self {
            quote: value.quote.to_string(),
            amount: value.amount,
            fee_reserve: value.fee_reserve,
            state: value.state,
            expiry: value.expiry,
            payment_preimage: value.payment_preimage,
            data: value.data,
        }
    }
}
