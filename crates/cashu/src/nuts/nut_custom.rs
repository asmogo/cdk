//! Custom Payment Methods
//!
//! This module defines types for custom payment methods that don't require additional fields
//! beyond the standard NUT-04 fields.
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
//! ## Field Naming Conventions
//!
//! Custom payment methods should namespace their fields to avoid collisions with
//! standard fields. For example:
//! - PayPal fields: `paypal_transaction_id`, `paypal_payer_email`, etc.
//! - Venmo fields: `venmo_user_id`, `venmo_payment_url`, etc.

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

/// Type alias for simple custom mint quote requests with no additional fields
///
/// This is a convenience type alias for [`MintQuoteRequest`] with [`NoAdditionalFields`].
/// Use this for custom payment methods that don't require any method-specific fields
/// beyond the standard NUT-04 fields.
///
/// ## Example
///
/// ```rust,ignore
/// use crate::nuts::nut04::MintQuoteRequest;
/// use crate::nuts::nut_custom::SimpleMintQuoteRequest;
/// use crate::nuts::CurrencyUnit;
/// use crate::Amount;
///
/// let request = SimpleMintQuoteRequest::new(
///     Amount::from(1000),
///     CurrencyUnit::Sat,
///     NoAdditionalFields,
/// );
/// ```
pub type SimpleMintQuoteRequest = crate::nuts::nut04::MintQuoteRequest<NoAdditionalFields>;

/// Type alias for simple custom mint quote responses with no additional fields
///
/// This is a convenience type alias for [`MintQuoteResponse`] with [`NoAdditionalFields`].
/// Use this for custom payment methods that don't require any method-specific fields
/// beyond the standard NUT-04 fields.
pub type SimpleMintQuoteResponse<Q> = crate::nuts::nut04::MintQuoteResponse<Q, NoAdditionalFields>;

/// Type alias for simple custom melt quote requests with no additional fields
///
/// This is a convenience type alias for [`MeltQuoteRequest`] with [`NoAdditionalFields`].
/// Use this for custom payment methods that don't require any method-specific fields
/// beyond the standard NUT-05 fields.
pub type SimpleMeltQuoteRequest = crate::nuts::nut05::MeltQuoteRequest<NoAdditionalFields>;

/// Type alias for simple custom melt quote responses with no additional fields
///
/// This is a convenience type alias for [`MeltQuoteResponse`] with [`NoAdditionalFields`].
/// Use this for custom payment methods that don't require any method-specific fields
/// beyond the standard NUT-05 fields.
pub type SimpleMeltQuoteResponse<Q> = crate::nuts::nut05::MeltQuoteResponse<Q, NoAdditionalFields>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nuts::CurrencyUnit;
    use crate::Amount;

    #[test]
    fn test_simple_mint_quote_request() {
        let request =
            SimpleMintQuoteRequest::new(Amount::from(1000), CurrencyUnit::Sat, NoAdditionalFields);
        assert_eq!(request.amount, Amount::from(1000));
        assert_eq!(request.unit, CurrencyUnit::Sat);
    }

    #[test]
    fn test_simple_melt_quote_request() {
        let request = SimpleMeltQuoteRequest::new(
            "custom://payment/123".to_string(),
            CurrencyUnit::Sat,
            NoAdditionalFields,
        );
        assert_eq!(request.request, "custom://payment/123");
        assert_eq!(request.unit, CurrencyUnit::Sat);
    }

    #[test]
    fn test_no_additional_fields_validation() {
        let fields = NoAdditionalFields;
        assert!(MintQuoteMethodFields::validate(&fields).is_ok());
    }
}
