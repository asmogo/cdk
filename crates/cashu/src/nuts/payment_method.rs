//! Payment Method Traits
//!
//! This module defines traits for extending mint quote types with payment method-specific fields.
//! The trait-based approach provides compile-time type safety while maintaining JSON compatibility
//! through serde's flatten mechanism.
//!
//! ## Architecture
//!
//! Instead of using `HashMap<String, Value>` for arbitrary data, we use generic type parameters
//! with trait bounds. This approach:
//! - Provides compile-time type safety
//! - Enables better IDE support and documentation
//! - Maintains JSON backward compatibility via `#[serde(flatten)]`
//! - Allows custom payment methods to be promoted to official ones without breaking changes
//!
//! ## NUT-04 Compliance
//!
//! The base types in [`crate::nuts::nut04`] ensure all NUT-04 required fields are present:
//! - `quote`: Quote identifier (required)
//! - `request`: Payment request string (required)
//! - `unit`: Currency unit (required)
//! - `state`: Quote state (required)
//! - `expiry`: Unix timestamp for quote validity (required)
//!
//! Payment methods add their specific fields via the traits defined here.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Trait for payment method-specific fields in mint quote requests
///
/// Implement this trait to add custom fields to mint quote requests for a specific payment method.
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::MintQuoteMethodFields;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalFields {
///     /// PayPal return URL after payment
///     pub paypal_return_url: String,
///     /// PayPal email address
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub paypal_email: Option<String>,
/// }
///
/// impl MintQuoteMethodFields for PayPalFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_return_url.is_empty() {
///             return Err("PayPal return URL cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "amount": 1000,
///   "unit": "sat",
///   "paypal_return_url": "https://example.com/return",
///   "paypal_email": "user@example.com"
/// }
/// ```
pub trait MintQuoteMethodFields: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for payment method-specific fields in mint quote responses
///
/// Implement this trait to add custom fields to mint quote responses for a specific payment method.
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::MintQuoteResponseFields;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalResponseFields {
///     /// PayPal transaction ID
///     pub paypal_transaction_id: String,
///     /// PayPal checkout URL for user to complete payment
///     pub paypal_checkout_url: String,
/// }
///
/// impl MintQuoteResponseFields for PayPalResponseFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_transaction_id.is_empty() {
///             return Err("PayPal transaction ID cannot be empty".to_string());
///         }
///         if self.paypal_checkout_url.is_empty() {
///             return Err("PayPal checkout URL cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "quote": "abc123",
///   "request": "paypal://checkout/xyz",
///   "unit": "sat",
///   "state": "UNPAID",
///   "expiry": 1234567890,
///   "paypal_transaction_id": "xyz789",
///   "paypal_checkout_url": "https://paypal.com/checkout/xyz"
/// }
/// ```
pub trait MintQuoteResponseFields: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific response fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for payment method-specific fields in melt quote requests
///
/// Implement this trait to add custom fields to melt quote requests for a specific payment method.
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::MeltQuoteMethodFields;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalFields {
///     /// PayPal payer email address
///     pub paypal_payer_email: String,
///     /// PayPal memo for the payment
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub paypal_memo: Option<String>,
/// }
///
/// impl MeltQuoteMethodFields for PayPalFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_payer_email.is_empty() {
///             return Err("PayPal payer email cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "request": "user@example.com",
///   "unit": "sat",
///   "paypal_payer_email": "user@example.com",
///   "paypal_memo": "Payment for services"
/// }
/// ```
pub trait MeltQuoteMethodFields: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for payment method-specific fields in melt quote responses
///
/// Implement this trait to add custom fields to melt quote responses for a specific payment method.
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::MeltQuoteResponseFields;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalResponseFields {
///     /// PayPal transaction ID
///     pub paypal_transaction_id: String,
///     /// PayPal payment status
///     pub paypal_status: String,
/// }
///
/// impl MeltQuoteResponseFields for PayPalResponseFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_transaction_id.is_empty() {
///             return Err("PayPal transaction ID cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "quote": "def456",
///   "amount": 950,
///   "unit": "sat",
///   "state": "PENDING",
///   "expiry": 1234567890,
///   "paypal_transaction_id": "abc123",
///   "paypal_status": "pending"
/// }
/// ```
pub trait MeltQuoteResponseFields: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific response fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for payment method-specific fields in incoming payment options
///
/// Implement this trait to add custom fields to incoming payment options for a specific payment method.
/// These are used when creating payment requests (e.g., invoices, payment URLs).
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::IncomingPaymentMethodData;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalIncomingFields {
///     /// PayPal return URL after payment
///     pub paypal_return_url: String,
///     /// PayPal webhook URL for payment notifications
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub paypal_webhook_url: Option<String>,
/// }
///
/// impl IncomingPaymentMethodData for PayPalIncomingFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_return_url.is_empty() {
///             return Err("PayPal return URL cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "method": "paypal",
///   "description": "Payment for services",
///   "amount": 1000,
///   "paypal_return_url": "https://example.com/return",
///   "paypal_webhook_url": "https://example.com/webhook"
/// }
/// ```
pub trait IncomingPaymentMethodData: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for payment method-specific fields in outgoing payment options
///
/// Implement this trait to add custom fields to outgoing payment options for a specific payment method.
/// These are used when making payments (e.g., paying invoices, sending to payment URLs).
/// Fields are serialized at the top level of the JSON object using `#[serde(flatten)]`.
///
/// ## Example Implementation
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use crate::nuts::payment_method::OutgoingPaymentMethodData;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct PayPalOutgoingFields {
///     /// PayPal payer email address
///     pub paypal_payer_email: String,
///     /// PayPal payment memo
///     #[serde(skip_serializing_if = "Option::is_none")]
///     pub paypal_memo: Option<String>,
/// }
///
/// impl OutgoingPaymentMethodData for PayPalOutgoingFields {
///     fn validate(&self) -> Result<(), String> {
///         if self.paypal_payer_email.is_empty() {
///             return Err("PayPal payer email cannot be empty".to_string());
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## JSON Serialization
///
/// With `#[serde(flatten)]`, the fields appear at the top level:
/// ```json
/// {
///   "method": "paypal",
///   "request": "user@example.com",
///   "max_fee_amount": 50,
///   "paypal_payer_email": "sender@example.com",
///   "paypal_memo": "Payment for invoice #123"
/// }
/// ```
pub trait OutgoingPaymentMethodData: Serialize + DeserializeOwned + Clone + Send + Sync {
    /// Validate the method-specific fields
    ///
    /// This method is called after deserialization to ensure all method-specific
    /// requirements are met. Return `Ok(())` if valid, or `Err(String)` with a
    /// descriptive error message if validation fails.
    ///
    /// ## Default Implementation
    ///
    /// The default implementation accepts all values. Override this method to add
    /// custom validation logic for your payment method.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}