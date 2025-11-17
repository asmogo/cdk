//! Custom Payment Methods
//!
//! This module defines types for custom payment methods that don't require additional fields
//! beyond the standard NUT-04 fields.
//!
//! ## Recommended Approach (Trait-Based)
//!
//! For custom payment methods, implement the [`MintQuoteMethodFields`] and
//! [`MintQuoteResponseFields`] traits on your own types, then use the generic
//! [`MintQuoteRequest`](crate::nuts::nut04::MintQuoteRequest) and [`MintQuoteResponse`](crate::nuts::nut04::MintQuoteResponse) types from [`crate::nuts::nut04`].
//!
//! For custom methods that don't need additional fields, pass an empty map for the method data.
//!
//! ## Field Naming Conventions
//!
//! Custom payment methods should namespace their fields to avoid collisions with
//! standard fields. For example:
//! - PayPal fields: `paypal_transaction_id`, `paypal_payer_email`, etc.
//! - Venmo fields: `venmo_user_id`, `venmo_payment_url`, etc.

use serde_json::{Map as JsonMap, Value as JsonValue};

use super::payment_method::{
    MeltQuoteMethodFields, MeltQuoteResponseFields, MintQuoteMethodFields, MintQuoteResponseFields,
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

impl MintQuoteMethodFields for JsonMap<String, JsonValue> {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MintQuoteResponseFields for JsonMap<String, JsonValue> {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MeltQuoteMethodFields for JsonMap<String, JsonValue> {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

impl MeltQuoteResponseFields for JsonMap<String, JsonValue> {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Type alias for generic custom mint quote requests with arbitrary flattened fields
///
/// This is a convenience type alias for [`MintQuoteRequest`](crate::nuts::nut04::MintQuoteRequest) with a [`serde_json::Map`]
/// providing method-specific data that is flattened into the top-level JSON object.
///
/// ## Example
///
/// ```rust,ignore
/// use crate::nuts::nut04::MintQuoteRequest;
/// use crate::nuts::nut_custom::GenericMintQuoteRequest;
/// use crate::nuts::CurrencyUnit;
/// use crate::Amount;
/// use serde_json::json;
///
/// let mut data = serde_json::Map::new();
/// data.insert("description".to_string(), json!("Payment"));
/// let request = GenericMintQuoteRequest::new(
///     Amount::from(1000),
///     CurrencyUnit::Sat,
///     data,
/// );
/// ```
pub type GenericMintQuoteRequest = crate::nuts::nut04::MintQuoteRequest<JsonMap<String, JsonValue>>;

/// Type alias for generic custom mint quote responses with arbitrary flattened fields
///
/// This is a convenience type alias for [`MintQuoteResponse`](crate::nuts::nut04::MintQuoteResponse) with a [`serde_json::Map`]
/// providing method-specific data that is flattened into the top-level JSON object.
pub type GenericMintQuoteResponse<Q> =
    crate::nuts::nut04::MintQuoteResponse<Q, JsonMap<String, JsonValue>>;

/// Type alias for generic custom melt quote requests with arbitrary flattened fields
///
/// This is a convenience type alias for [`MeltQuoteRequest`](crate::nuts::nut05::MeltQuoteRequest) with a [`serde_json::Map`]
/// providing method-specific data that is flattened into the top-level JSON object.
pub type GenericMeltQuoteRequest = crate::nuts::nut05::MeltQuoteRequest<JsonMap<String, JsonValue>>;

/// Type alias for generic custom melt quote responses with arbitrary flattened fields
///
/// This is a convenience type alias for [`MeltQuoteResponse`](crate::nuts::nut05::MeltQuoteResponse) with a [`serde_json::Map`]
/// providing method-specific data that is flattened into the top-level JSON object.
pub type GenericMeltQuoteResponse<Q> =
    crate::nuts::nut05::MeltQuoteResponse<Q, JsonMap<String, JsonValue>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nuts::CurrencyUnit;
    use crate::Amount;

    #[test]
    fn test_generic_mint_quote_request() {
        let mut data = JsonMap::new();
        data.insert(
            "description".to_string(),
            JsonValue::String("Test".to_string()),
        );
        let request = GenericMintQuoteRequest::new(Amount::from(1000), CurrencyUnit::Sat, data);
        assert_eq!(request.amount, Amount::from(1000));
        assert_eq!(request.unit, CurrencyUnit::Sat);
        assert!(request.method_fields.contains_key("description"));
    }

    #[test]
    fn test_generic_melt_quote_request() {
        let mut data = JsonMap::new();
        data.insert("memo".to_string(), JsonValue::String("Test".to_string()));
        let request = GenericMeltQuoteRequest::new(
            "custom://payment/123".to_string(),
            CurrencyUnit::Sat,
            data,
        );
        assert_eq!(request.request, "custom://payment/123");
        assert_eq!(request.unit, CurrencyUnit::Sat);
        assert!(request.method_fields.contains_key("memo"));
    }

    #[test]
    fn test_no_additional_fields_validation() {
        let fields = NoAdditionalFields;
        assert!(MintQuoteMethodFields::validate(&fields).is_ok());
    }
}
