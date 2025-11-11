//! Custom Payment Methods
//!
//! This module defines types for custom payment methods that don't require additional fields
//! beyond the standard NUT-04 fields, as well as deprecated HashMap-based types for backwards
//! compatibility.
//!
//! ## Recommended Approach (Trait-Based)
//!
//! For custom payment methods, implement the [`MintQuoteMethodFields`] and
//! [`MintQuoteResponseFields`] traits on your own types, then use the generic
//! [`MintQuoteRequest`] and [`MintQuoteResponse`] types from [`crate::nuts::nut04`].
//!
//! For simple custom methods that don't need additional fields, use the type aliases:
//! - [`SimpleMintQuoteRequest`] - Request with no additional fields
//! - [`SimpleMintQuoteResponse`] - Response with no additional fields
//!
//! ## Deprecated Approach (HashMap-Based)
//!
//! The HashMap-based types ([`MintQuoteCustomRequest`], [`MintQuoteCustomResponse`]) are
//! deprecated and will be removed in a future version. They lack type safety and should
//! not be used for new code.
//!
//! ## Field Naming Conventions
//!
//! Custom payment methods should namespace their fields to avoid collisions with
//! standard fields. For example:
//! - PayPal fields: `paypal_transaction_id`, `paypal_payer_email`, etc.
//! - Venmo fields: `venmo_user_id`, `venmo_payment_url`, etc.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::payment_method::{
    IncomingPaymentMethodData, MeltQuoteMethodFields, MeltQuoteResponseFields,
    MintQuoteMethodFields, MintQuoteResponseFields, OutgoingPaymentMethodData,
};
use super::CurrencyUnit;
use crate::nut05::QuoteState as MeltQuoteState;
use crate::nut23::QuoteState as MintQuoteState;
#[cfg(feature = "mint")]
use crate::quote_id::QuoteId;
use crate::Amount;

/// Zero-sized marker type for custom payment methods that don't require additional fields
///
/// This type implements both [`MintQuoteMethodFields`] and [`MintQuoteResponseFields`]
/// with no additional data, making it suitable for simple custom payment methods that
/// only need the standard NUT-04 fields.
///
/// ## Use Cases
///
/// - Simple custom payment methods that don't require method-specific fields
/// - Testing and development
/// - Migration path from HashMap-based types
///
/// ## Example
///
/// ```rust,ignore
/// use crate::nuts::nut04::MintQuoteResponse;
/// use crate::nuts::nut_custom::NoAdditionalFields;
/// use crate::nuts::CurrencyUnit;
/// use crate::nut23::QuoteState;
///
/// let response = MintQuoteResponse {
///     quote: "abc123".to_string(),
///     request: "custom://payment/xyz".to_string(),
///     unit: CurrencyUnit::Sat,
///     state: QuoteState::Unpaid,
///     expiry: 1234567890,
///     method_fields: NoAdditionalFields,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoAdditionalFields;

impl MintQuoteMethodFields for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MintQuoteResponseFields for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MeltQuoteMethodFields for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MeltQuoteResponseFields for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl IncomingPaymentMethodData for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl OutgoingPaymentMethodData for NoAdditionalFields {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Type alias for mint quote requests with no additional payment method fields
///
/// This is a convenience type for simple custom payment methods that don't require
/// method-specific fields beyond the standard NUT-04 fields (amount, unit).
///
/// ## Example JSON
///
/// ```json
/// {
///   "amount": 1000,
///   "unit": "sat"
/// }
/// ```
pub type SimpleMintQuoteRequest = super::nut04::MintQuoteRequest<NoAdditionalFields>;

/// Type alias for mint quote responses with no additional payment method fields
///
/// This is a convenience type for simple custom payment methods that don't require
/// method-specific response fields beyond the standard NUT-04 fields (quote, request,
/// unit, state, expiry).
///
/// ## Type Parameters
///
/// - `Q`: Quote identifier type (typically `String` or `QuoteId`)
///
/// ## Example JSON
///
/// ```json
/// {
///   "quote": "abc123",
///   "request": "custom://payment/xyz",
///   "unit": "sat",
///   "state": "UNPAID",
///   "expiry": 1234567890
/// }
/// ```
pub type SimpleMintQuoteResponse<Q> = super::nut04::MintQuoteResponse<Q, NoAdditionalFields>;

/// Type alias for melt quote requests with no additional payment method fields
///
/// This is a convenience type for simple custom payment methods that don't require
/// method-specific fields beyond the standard NUT-05 fields (request, unit).
///
/// ## Example JSON
///
/// ```json
/// {
///   "request": "lnbc10u1...",
///   "unit": "sat"
/// }
/// ```
pub type SimpleMeltQuoteRequest = super::nut05::MeltQuoteRequest<NoAdditionalFields>;

/// Type alias for melt quote responses with no additional payment method fields
///
/// This is a convenience type for simple custom payment methods that don't require
/// method-specific response fields beyond the standard NUT-05 fields (quote, amount,
/// unit, state, expiry).
///
/// ## Type Parameters
///
/// - `Q`: Quote identifier type (typically `String` or `QuoteId`)
///
/// ## Example JSON
///
/// ```json
/// {
///   "quote": "abc123",
///   "amount": 1000,
///   "unit": "sat",
///   "state": "UNPAID",
///   "expiry": 1234567890
/// }
/// ```
pub type SimpleMeltQuoteResponse<Q> = super::nut05::MeltQuoteResponse<Q, NoAdditionalFields>;

/// DEPRECATED: Custom payment method mint quote request using HashMap
///
/// This type is deprecated in favor of the trait-based approach using
/// [`MintQuoteRequest`](super::nut04::MintQuoteRequest) with custom types that implement
/// [`MintQuoteMethodFields`].
///
/// For custom methods that don't need additional fields, use [`SimpleMintQuoteRequest`] instead.
///
/// ## Migration Path
///
/// Instead of using `HashMap<String, Value>`, define a proper struct:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyCustomFields {
///     my_field: String,
/// }
///
/// impl MintQuoteMethodFields for MyCustomFields { }
///
/// // Then use: MintQuoteRequest<MyCustomFields>
/// ```
///
/// ## Deprecation Timeline
///
/// This type will be removed in a future major version.
#[deprecated(
    since = "0.5.0",
    note = "Use MintQuoteRequest<M> with custom types implementing MintQuoteMethodFields instead"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct MintQuoteCustomRequest {
    /// Currency unit
    pub unit: CurrencyUnit,
    /// Method-specific data (flattened into top-level fields)
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

/// DEPRECATED: Custom payment method mint quote response using HashMap
///
/// This type is deprecated in favor of the trait-based approach using
/// [`MintQuoteResponse`](super::nut04::MintQuoteResponse) with custom types that implement
/// [`MintQuoteResponseFields`].
///
/// For custom methods that don't need additional fields, use [`SimpleMintQuoteResponse`] instead.
///
/// ## Migration Path
///
/// Instead of using `HashMap<String, Value>`, define a proper struct:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyCustomResponseFields {
///     my_field: String,
/// }
///
/// impl MintQuoteResponseFields for MyCustomResponseFields { }
///
/// // Then use: MintQuoteResponse<Q, MyCustomResponseFields>
/// ```
///
/// ## Important: Missing Required Fields
///
/// This deprecated type is missing the `expiry` field which is REQUIRED by NUT-04.
/// The new [`MintQuoteResponse`](super::nut04::MintQuoteResponse) type includes all
/// required fields.
///
/// ## Deprecation Timeline
///
/// This type will be removed in a future major version.
#[deprecated(
    since = "0.5.0",
    note = "Use MintQuoteResponse<Q, M> with custom types implementing MintQuoteResponseFields instead. Note: This type is missing the required 'expiry' field."
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[serde(bound = "Q: Serialize + for<'a> Deserialize<'a>")]
pub struct MintQuoteCustomResponse<Q> {
    /// Quote ID
    pub quote: Q,
    /// Payment request string (method-specific format)
    pub request: String,
    /// Currency unit
    pub unit: Option<CurrencyUnit>,
    /// Quote State
    pub state: MintQuoteState,
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
            state: self.state,
            unit: self.unit.clone(),
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
            unit: value.unit,
            state: value.state,
            data: value.data,
        }
    }
}

/// DEPRECATED: Custom payment method melt quote request using HashMap
///
/// This type is deprecated in favor of the trait-based approach using
/// [`MeltQuoteRequest`](super::nut05::MeltQuoteRequest) with custom types that implement
/// [`MeltQuoteMethodFields`].
///
/// For custom methods that don't need additional fields, use [`SimpleMeltQuoteRequest`] instead.
///
/// ## Migration Path
///
/// Instead of using `HashMap<String, Value>`, define a proper struct:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyCustomFields {
///     my_field: String,
/// }
///
/// impl MeltQuoteMethodFields for MyCustomFields { }
///
/// // Then use: MeltQuoteRequest<MyCustomFields>
/// ```
///
/// ## Important: Non-Standard Fields
///
/// This deprecated type includes a `method` field which is NOT part of NUT-05.
/// Per NUT-05, the payment method is specified in the URL path, not the request body.
/// The new [`MeltQuoteRequest`](super::nut05::MeltQuoteRequest) type correctly
/// excludes this field.
///
/// ## Deprecation Timeline
///
/// This type will be removed in a future major version.
#[deprecated(
    since = "0.5.0",
    note = "Use MeltQuoteRequest<M> with custom types implementing MeltQuoteMethodFields instead. Note: The 'method' field should be in the URL path, not the request body per NUT-05."
)]
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

/// DEPRECATED: Custom payment method melt quote response using HashMap
///
/// This type is deprecated in favor of the trait-based approach using
/// [`MeltQuoteResponse`](super::nut05::MeltQuoteResponse) with custom types that implement
/// [`MeltQuoteResponseFields`].
///
/// For custom methods that don't need additional fields, use [`SimpleMeltQuoteResponse`] instead.
///
/// ## Migration Path
///
/// Instead of using `HashMap<String, Value>`, define a proper struct:
///
/// ```rust,ignore
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct MyCustomResponseFields {
///     my_field: String,
/// }
///
/// impl MeltQuoteResponseFields for MyCustomResponseFields { }
///
/// // Then use: MeltQuoteResponse<Q, MyCustomResponseFields>
/// ```
///
/// ## Important: Non-Standard Fields
///
/// This deprecated type includes several issues:
/// - `expiry` is Optional but is REQUIRED by NUT-05
/// - `fee_reserve` is included at the base level but is Lightning-specific (NUT-23), not part of NUT-05
/// - `payment_preimage` is Lightning-specific, not part of base NUT-05
/// - `unit` field is missing but is REQUIRED by NUT-05
///
/// The new [`MeltQuoteResponse`](super::nut05::MeltQuoteResponse) type includes all
/// required NUT-05 fields and delegates method-specific fields to the trait implementation.
///
/// ## Deprecation Timeline
///
/// This type will be removed in a future major version.
#[deprecated(
    since = "0.5.0",
    note = "Use MeltQuoteResponse<Q, M> with custom types implementing MeltQuoteResponseFields instead. Note: This type is missing required 'unit' field and has 'expiry' as Optional when it should be required per NUT-05."
)]
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
